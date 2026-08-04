use std::path::Path;

/// A video rotation in degrees (clockwise), restricted to the four values
/// that video containers and mpv actually produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rotation {
    #[default]
    R0,
    R90,
    R180,
    R270,
}

impl Rotation {
    /// Normalizes an arbitrary degree value (which may be negative or larger
    /// than 360) into the nearest supported [`Rotation`]. Any value that does
    /// not round to 90/180/270 is treated as no rotation.
    pub fn from_degrees(degrees: i64) -> Self {
        match degrees.rem_euclid(360) {
            90 => Self::R90,
            180 => Self::R180,
            270 => Self::R270,
            _ => Self::R0,
        }
    }

    /// The rotation in degrees (0, 90, 180 or 270).
    pub fn as_degrees(self) -> i64 {
        match self {
            Self::R0 => 0,
            Self::R90 => 90,
            Self::R180 => 180,
            Self::R270 => 270,
        }
    }

    /// Whether the rotation swaps the effective width and height of the video
    /// (90° and 270°).
    pub fn is_swapped(self) -> bool {
        matches!(self, Self::R90 | Self::R270)
    }
}

/// Detects the rotation of a video file by parsing its container metadata:
/// the MP4 `tkhd` matrix, EXIF orientation, and mp4ameta tags — in that
/// order. Returns `None` when the file cannot be read or carries no rotation
/// information.
pub fn detect_video_rotation(path: &Path) -> Option<Rotation> {
    if let Some(rot) = read_mp4_tkhd_rotation(path)
        && rot != Rotation::R0
    {
        return Some(rot);
    }
    if let Some(rot) = read_exif_video_rotation(path)
        && rot != Rotation::R0
    {
        return Some(rot);
    }
    if let Some(rot) = read_mp4ameta_rotation(path)
        && rot != Rotation::R0
    {
        return Some(rot);
    }
    None
}

fn read_mp4_tkhd_rotation(path: &Path) -> Option<Rotation> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path).ok()?;
    let file_len = file.metadata().ok()?.len();

    let mut buf = [0u8; 8];
    let mut boxes_visited: u32 = 0;
    while file.stream_position().unwrap_or(file_len) + 8 <= file_len {
        boxes_visited += 1;
        if boxes_visited > 10_000 {
            // A hostile file can declare an unbounded number of tiny boxes
            // (16 bytes each); cap the walk so it cannot spin on CPU/IO.
            break;
        }
        if file.read_exact(&mut buf).is_err() {
            break;
        }
        let size32 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as u64;
        let box_type = &buf[4..8];

        let box_len = if size32 == 1 {
            let mut size64_buf = [0u8; 8];
            if file.read_exact(&mut size64_buf).is_err() {
                break;
            }
            u64::from_be_bytes(size64_buf)
        } else if size32 == 0 {
            let header_end = file.stream_position().unwrap_or(8);
            file_len - header_end + 8
        } else {
            size32
        };

        if box_len < 8 {
            break;
        }

        // Payload start: after the 8-byte header plus the size64 field of
        // extended-size boxes. Recording it after the size64 read keeps the
        // tkhd seek and the `content_start + payload_len` advance aligned —
        // otherwise a 64-bit box (e.g. a large `mdat`) lands 8 bytes early
        // and desyncs the box walk.
        let content_start = file.stream_position().unwrap_or(0);

        let payload_len = box_len.saturating_sub(if size32 == 1 { 16 } else { 8 });

        if box_type == b"moov" || box_type == b"trak" {
            continue;
        } else if box_type == b"tkhd" {
            // Read only the payload: `box_len` includes the 8-byte header, so
            // reading that many bytes from `content_start` would over-read
            // into the next box (or past EOF when tkhd is the last box).
            let mut tkhd_data = vec![0u8; (payload_len as usize).min(256)];
            file.seek(SeekFrom::Start(content_start)).ok()?;
            if file.read_exact(&mut tkhd_data).is_ok() && tkhd_data.len() >= 70 {
                let version = tkhd_data[0];
                let matrix_offset = if version == 1 { 52 } else { 40 };

                if tkhd_data.len() >= matrix_offset + 36 {
                    let m = &tkhd_data[matrix_offset..matrix_offset + 36];
                    let a = i32::from_be_bytes([m[0], m[1], m[2], m[3]]) as f64 / 65536.0;
                    let b = i32::from_be_bytes([m[4], m[5], m[6], m[7]]) as f64 / 65536.0;
                    let c = i32::from_be_bytes([m[12], m[13], m[14], m[15]]) as f64 / 65536.0;
                    let d = i32::from_be_bytes([m[16], m[17], m[18], m[19]]) as f64 / 65536.0;

                    let rot = if a.abs() < 0.1 && b > 0.5 && c < -0.5 && d.abs() < 0.1 {
                        Rotation::R90
                    } else if a < -0.5 && b.abs() < 0.1 && c.abs() < 0.1 && d < -0.5 {
                        Rotation::R180
                    } else if a.abs() < 0.1 && b < -0.5 && c > 0.5 && d.abs() < 0.1 {
                        Rotation::R270
                    } else {
                        Rotation::R0
                    };

                    if rot != Rotation::R0 {
                        return Some(rot);
                    }
                }
            }
        }

        let Some(next) = content_start.checked_add(payload_len) else {
            // u64 overflow (e.g. an extended-size box declaring a length of
            // 2^64 - k wraps the walk position) - never trust it.
            break;
        };
        // A declared box must lie within the file; anything beyond EOF is
        // invalid input, not a sparse-file feature worth following.
        if next > file_len {
            break;
        }
        if file.seek(SeekFrom::Start(next)).is_err() {
            break;
        }
    }
    None
}

fn read_exif_video_rotation(path: &Path) -> Option<Rotation> {
    let file = std::fs::File::open(path).ok()?;
    let mut bufreader = std::io::BufReader::new(file);
    let exif = exif::Reader::new()
        .read_from_container(&mut bufreader)
        .ok()?;
    let field = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;
    let orientation = field.value.get_uint(0)?;
    match orientation {
        6 => Some(Rotation::R90),
        3 => Some(Rotation::R180),
        8 => Some(Rotation::R270),
        1 => Some(Rotation::R0),
        _ => None,
    }
}

fn read_mp4ameta_rotation(path: &Path) -> Option<Rotation> {
    let mut file = std::fs::File::open(path).ok()?;
    let tag = mp4ameta::Tag::read_from(&mut file).ok()?;

    for (_ident, val) in tag.strings() {
        if let Ok(parsed) = val.trim().parse::<i64>() {
            let rotation = Rotation::from_degrees(parsed);
            if rotation != Rotation::R0 {
                return Some(rotation);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_video_rotation_nonexistent() {
        assert_eq!(
            detect_video_rotation(Path::new("/nonexistent/video.mp4")),
            None
        );
    }

    #[test]
    fn test_rotation_from_degrees() {
        assert_eq!(Rotation::from_degrees(0), Rotation::R0);
        assert_eq!(Rotation::from_degrees(90), Rotation::R90);
        assert_eq!(Rotation::from_degrees(180), Rotation::R180);
        assert_eq!(Rotation::from_degrees(270), Rotation::R270);
        assert_eq!(Rotation::from_degrees(360), Rotation::R0);
        assert_eq!(Rotation::from_degrees(-90), Rotation::R270);
        assert_eq!(Rotation::from_degrees(450), Rotation::R90);
        assert_eq!(Rotation::from_degrees(5), Rotation::R0);
    }

    #[test]
    fn test_rotation_is_swapped() {
        assert!(!Rotation::R0.is_swapped());
        assert!(Rotation::R90.is_swapped());
        assert!(!Rotation::R180.is_swapped());
        assert!(Rotation::R270.is_swapped());
    }

    #[test]
    fn test_rotation_degrees_roundtrip() {
        for rot in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
            assert_eq!(Rotation::from_degrees(rot.as_degrees()), rot);
        }
    }

    /// Builds a minimal MP4: an `ftyp`, a *64-bit-sized* `mdat` (extended-size
    /// boxes are what previously desynced the box walk by 8 bytes), and a
    /// `moov`/`trak`/`tkhd` whose matrix encodes a 90° rotation.
    #[test]
    fn test_mp4_with_size64_mdat_rotation_detected() {
        let mut bytes = Vec::new();

        let ftyp: [u8; 24] = [
            0, 0, 0, 24, b'f', b't', b'y', b'p', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        bytes.extend_from_slice(&ftyp);

        // mdat with a 64-bit size: 16-byte header, size32 == 1 marker.
        let mdat_payload: [u8; 4] = [0, 0, 0, 0];
        let mdat_len = 16u64 + mdat_payload.len() as u64;
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.extend_from_slice(b"mdat");
        bytes.extend_from_slice(&mdat_len.to_be_bytes());
        bytes.extend_from_slice(&mdat_payload);

        // tkhd payload (version 0): 40 bytes before the 36-byte matrix.
        let mut tkhd = Vec::new();
        tkhd.extend_from_slice(&[0u8; 40]);
        let unit = 65536.0f64; // 16.16 fixed point
        let (a, b, c, d) = (0.0, 1.0, -1.0, 0.0); // 90° clockwise
        for v in [a, b, 0.0, c, d, 0.0, 0.0, 0.0, 1.0] {
            tkhd.extend_from_slice(&((v * unit) as i32).to_be_bytes());
        }
        let tkhd_box: Vec<u8> = {
            let mut box_bytes = Vec::new();
            box_bytes.extend_from_slice(&((8 + tkhd.len()) as u32).to_be_bytes());
            box_bytes.extend_from_slice(b"tkhd");
            box_bytes.extend_from_slice(&tkhd);
            box_bytes
        };

        // trak wraps tkhd, moov wraps trak (both 32-bit sizes).
        let trak_box: Vec<u8> = {
            let mut box_bytes = Vec::new();
            box_bytes.extend_from_slice(&((8 + tkhd_box.len()) as u32).to_be_bytes());
            box_bytes.extend_from_slice(b"trak");
            box_bytes.extend_from_slice(&tkhd_box);
            box_bytes
        };
        let moov_box: Vec<u8> = {
            let mut box_bytes = Vec::new();
            box_bytes.extend_from_slice(&((8 + trak_box.len()) as u32).to_be_bytes());
            box_bytes.extend_from_slice(b"moov");
            box_bytes.extend_from_slice(&trak_box);
            box_bytes
        };
        bytes.extend_from_slice(&moov_box);

        let dir =
            std::env::temp_dir().join(format!("mpv_utils_rotation_size64_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("rotated.mp4");
        std::fs::write(&path, &bytes).unwrap();

        assert_eq!(
            detect_video_rotation(&path),
            Some(Rotation::R90),
            "the size64 mdat must not desync the box walk"
        );

        std::fs::remove_dir_all(&dir).ok();
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    /// Two-box construction that wraps the u64 next-position arithmetic:
    /// box A (size 32) walks 0 -> 32, then an extended-size box with
    /// size64 = 2^64 - 32 maps 32 -> 2^64, which wraps to 0 in release
    /// (infinite loop) or overflows in debug. Must terminate with None.
    #[test]
    fn test_mp4_box_walker_overflow_terminates() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&32u32.to_be_bytes());
        bytes.extend_from_slice(b"free");
        bytes.extend_from_slice(&[0u8; 24]);
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.extend_from_slice(b"tkhd");
        bytes.extend_from_slice(&(u64::MAX - 31).to_be_bytes()); // 2^64 - 32

        let dir = std::env::temp_dir().join(format!(
            "mpv_utils_rotation_overflow_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("overflow.mp4");
        std::fs::write(&path, &bytes).unwrap();

        let started = std::time::Instant::now();
        let result = detect_video_rotation(&path);
        let elapsed = started.elapsed();

        assert_eq!(result, None, "overflow payload must not loop");
        assert!(
            elapsed < std::time::Duration::from_secs(1),
            "box walk took {elapsed:?}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }
}
