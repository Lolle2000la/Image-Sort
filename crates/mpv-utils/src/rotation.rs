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
    while file.stream_position().unwrap_or(file_len) + 8 <= file_len {
        if file.read_exact(&mut buf).is_err() {
            break;
        }
        let size32 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as u64;
        let box_type = &buf[4..8];

        let content_start = file.stream_position().unwrap_or(0);

        let box_len = if size32 == 1 {
            let mut size64_buf = [0u8; 8];
            if file.read_exact(&mut size64_buf).is_err() {
                break;
            }
            u64::from_be_bytes(size64_buf)
        } else if size32 == 0 {
            file_len - content_start + 8
        } else {
            size32
        };

        if box_len < 8 {
            break;
        }

        let payload_len = box_len.saturating_sub(if size32 == 1 { 16 } else { 8 });

        if box_type == b"moov" || box_type == b"trak" {
            continue;
        } else if box_type == b"tkhd" {
            let mut tkhd_data = vec![0u8; (box_len as usize).min(256)];
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

        if file
            .seek(SeekFrom::Start(content_start + payload_len))
            .is_err()
        {
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
}
