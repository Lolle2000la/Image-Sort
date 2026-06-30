use std::collections::BTreeMap;
use std::path::Path;

use super::image_meta::MetadataError;

pub fn extract_audio_metadata(
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

    // 2. Read specific audio tags
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let specific_tags = match ext.to_lowercase().as_str() {
        "mp3" | "aiff" | "wav" => extract_id3_metadata(path),
        "flac" => extract_flac_metadata(path),
        "ogg" | "opus" => extract_vorbis_comment_metadata(path),
        "m4a" | "aac" => super::video_meta::extract_video_metadata(path),
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

fn extract_id3_metadata(
    path: &Path,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    use id3::{Tag, TagLike};

    let tag = Tag::read_from_path(path)?;
    let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut id3_section: BTreeMap<String, String> = BTreeMap::new();

    if let Some(title) = tag.title() {
        id3_section.insert("Title".into(), title.to_string());
    }
    if let Some(artist) = tag.artist() {
        id3_section.insert("Artist".into(), artist.to_string());
    }
    if let Some(album) = tag.album() {
        id3_section.insert("Album".into(), album.to_string());
    }
    if let Some(year) = tag.year() {
        id3_section.insert("Year".into(), year.to_string());
    }
    if let Some(genre) = tag.genre() {
        id3_section.insert("Genre".into(), genre.to_string());
    }
    if let Some(track) = tag.track() {
        id3_section.insert("Track".into(), track.to_string());
    }

    if !id3_section.is_empty() {
        sections.insert("ID3 Metadata".into(), id3_section);
    }

    Ok(sections)
}

fn extract_flac_metadata(
    path: &Path,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    use metaflac::Tag;

    let tag = Tag::read_from_path(path)?;
    let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut flac_section: BTreeMap<String, String> = BTreeMap::new();

    if let Some(vc) = tag.vorbis_comments() {
        for (key, values) in &vc.comments {
            if let Some(value) = values.first() {
                flac_section.insert(key.clone(), value.clone());
            }
        }
    }

    if !flac_section.is_empty() {
        sections.insert("FLAC Vorbis Comment".into(), flac_section);
    }

    Ok(sections)
}

fn extract_vorbis_comment_metadata(
    path: &Path,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    super::video_meta::extract_generic_container_metadata(path)
}

fn extract_generic_container_metadata(
    path: &Path,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    super::video_meta::extract_generic_container_metadata(path)
}
