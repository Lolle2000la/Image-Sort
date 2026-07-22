use std::collections::BTreeMap;
use std::path::Path;

use super::image_meta::MetadataError;

pub fn extract_video_metadata(
    path: &Path,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    let mut dirs: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    // Validate that the file exists and is accessible
    let meta = std::fs::metadata(path)?;

    // 1. Add basic file metadata
    let mut file_sec: BTreeMap<String, String> = BTreeMap::new();
    if let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) {
        file_sec.insert("Name".into(), name);
    }
    let bytes = meta.len();
    let size_str = if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    };
    file_sec.insert("Size".into(), size_str);

    if let Ok(modified) = meta.modified() {
        let datetime: chrono::DateTime<chrono::Local> = modified.into();
        file_sec.insert(
            "Modified".into(),
            datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
        );
    }
    dirs.insert("File".into(), file_sec);

    // 2. Read specific video tags
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let specific_tags = match ext.to_lowercase().as_str() {
        "mp4" | "m4v" | "mov" => extract_mp4_metadata(path),
        "mkv" | "webm" => extract_matroska_metadata(path),
        _ => extract_generic_container_metadata(path),
    };

    if let Ok(tags) = specific_tags {
        for (sec_name, sec_data) in tags {
            if sec_name == "File" {
                dirs.entry("File".into()).or_default().extend(sec_data);
            } else {
                dirs.insert(sec_name, sec_data);
            }
        }
    }

    Ok(dirs)
}

fn extract_mp4_metadata(
    path: &Path,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    use mp4ameta::Tag;

    let mut file = std::fs::File::open(path)?;
    let tag = Tag::read_from(&mut file)?;

    let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut mp4_section: BTreeMap<String, String> = BTreeMap::new();

    if let Some(title) = tag.title() {
        mp4_section.insert("Title".into(), title.to_string());
    }
    if let Some(artist) = tag.artist() {
        mp4_section.insert("Artist".into(), artist.to_string());
    }
    if let Some(album) = tag.album() {
        mp4_section.insert("Album".into(), album.to_string());
    }
    if let Some(year) = tag.year() {
        mp4_section.insert("Year".into(), year.to_string());
    }
    if let Some(genre) = tag.genre() {
        mp4_section.insert("Genre".into(), genre.to_string());
    }
    if let Some(track) = tag.track_number() {
        mp4_section.insert("Track".into(), track.to_string());
    }
    if let Some(total) = tag.total_tracks() {
        mp4_section.insert("Total Tracks".into(), total.to_string());
    }

    if !mp4_section.is_empty() {
        sections.insert("MP4 Metadata".into(), mp4_section);
    }

    Ok(sections)
}

fn extract_matroska_metadata(
    path: &Path,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    extract_generic_container_metadata(path)
}

pub fn extract_generic_container_metadata(
    path: &Path,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::formats::probe::Hint;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;

    let mut sections = BTreeMap::new();

    if let Ok(file) = std::fs::File::open(path) {
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let meta_opts = MetadataOptions::default();
        let fmt_opts = FormatOptions::default();

        if let Ok(mut format) =
            symphonia::default::get_probe().probe(&hint, mss, fmt_opts, meta_opts)
        {
            let mut metadata = format.metadata();
            let mut container_sec = BTreeMap::new();

            // Get all revisions
            while !metadata.is_latest() {
                if let Some(revision) = metadata.pop() {
                    for tag in &revision.media.tags {
                        let key = match &tag.std {
                            Some(std_key) => format!("{:?}", std_key),
                            None => tag.raw.key.clone(),
                        };
                        let val = tag.raw.value.to_string();
                        if !val.trim().is_empty() {
                            container_sec.insert(key, val);
                        }
                    }
                }
            }

            // Also check the current/latest revision
            if let Some(revision) = metadata.current() {
                for tag in &revision.media.tags {
                    let key = match &tag.std {
                        Some(std_key) => format!("{:?}", std_key),
                        None => tag.raw.key.clone(),
                    };
                    let val = tag.raw.value.to_string();
                    if !val.trim().is_empty() {
                        container_sec.insert(key, val);
                    }
                }
            }

            if !container_sec.is_empty() {
                sections.insert("Container Metadata".into(), container_sec);
            }
        }
    }

    Ok(sections)
}

pub fn detect_video_rotation(path: &Path) -> Option<i64> {
    if let Some(rot) = read_mp4_tkhd_rotation(path)
        && rot != 0
    {
        return Some(rot);
    }
    if let Some(rot) = read_exif_video_rotation(path)
        && rot != 0
    {
        return Some(rot);
    }
    if let Some(rot) = read_mp4ameta_rotation(path)
        && rot != 0
    {
        return Some(rot);
    }
    None
}

fn read_mp4_tkhd_rotation(path: &Path) -> Option<i64> {
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
                        90
                    } else if a < -0.5 && b.abs() < 0.1 && c.abs() < 0.1 && d < -0.5 {
                        180
                    } else if a.abs() < 0.1 && b < -0.5 && c > 0.5 && d.abs() < 0.1 {
                        270
                    } else {
                        0
                    };

                    if rot != 0 {
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

fn read_exif_video_rotation(path: &Path) -> Option<i64> {
    let file = std::fs::File::open(path).ok()?;
    let mut bufreader = std::io::BufReader::new(file);
    let exif = exif::Reader::new()
        .read_from_container(&mut bufreader)
        .ok()?;
    let field = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;
    let orientation = field.value.get_uint(0)?;
    match orientation {
        6 => Some(90),
        3 => Some(180),
        8 => Some(270),
        1 => Some(0),
        _ => None,
    }
}

fn read_mp4ameta_rotation(path: &Path) -> Option<i64> {
    let mut file = std::fs::File::open(path).ok()?;
    let tag = mp4ameta::Tag::read_from(&mut file).ok()?;

    for (_ident, val) in tag.strings() {
        if let Ok(parsed) = val.trim().parse::<i64>() {
            let norm = parsed.rem_euclid(360);
            if norm != 0 {
                return Some(norm);
            }
        }
    }
    None
}
