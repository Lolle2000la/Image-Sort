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
        file_sec.insert("Modified".into(), datetime.format("%Y-%m-%d %H:%M:%S").to_string());
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

fn extract_generic_container_metadata(
    _path: &Path,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    Ok(BTreeMap::new())
}
