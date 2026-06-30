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
        mp4_section.insert("Track".into(), track.0.to_string());
        mp4_section.insert("Total Tracks".into(), track.1.to_string());
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
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let mut sections = BTreeMap::new();

    if let Ok(file) = std::fs::File::open(path) {
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let meta_opts = MetadataOptions::default();
        let fmt_opts = FormatOptions::default();

        if let Ok(probed) =
            symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)
        {
            let mut format = probed.format;
            let mut metadata = format.metadata();
            let mut container_sec = BTreeMap::new();

            // Get all revisions
            while !metadata.is_latest() {
                if let Some(revision) = metadata.pop() {
                    for tag in revision.tags() {
                        let key = match tag.std_key {
                            Some(std_key) => format!("{:?}", std_key),
                            None => tag.key.clone(),
                        };
                        let val = tag.value.to_string();
                        if !val.trim().is_empty() {
                            container_sec.insert(key, val);
                        }
                    }
                }
            }

            // Also check the current/latest revision
            if let Some(revision) = metadata.current() {
                for tag in revision.tags() {
                    let key = match tag.std_key {
                        Some(std_key) => format!("{:?}", std_key),
                        None => tag.key.clone(),
                    };
                    let val = tag.value.to_string();
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
