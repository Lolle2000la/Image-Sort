use std::collections::BTreeMap;
use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("extraction failed")]
    ExtractionFailed,
    #[error("metadata parse error: {0}")]
    ParseError(String),
}

impl From<id3::Error> for MetadataError {
    fn from(e: id3::Error) -> Self {
        MetadataError::ParseError(e.to_string())
    }
}

impl From<metaflac::Error> for MetadataError {
    fn from(e: metaflac::Error) -> Self {
        MetadataError::ParseError(e.to_string())
    }
}

impl From<mp4ameta::Error> for MetadataError {
    fn from(e: mp4ameta::Error) -> Self {
        MetadataError::ParseError(e.to_string())
    }
}

pub fn extract_image_metadata(
    path: &Path,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, MetadataError> {
    use exif::Reader;

    // Validate that the file exists and is accessible
    let meta = std::fs::metadata(path)?;

    let mut dirs: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

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

    if let Ok((w, h)) = crate::media::image_decoder::decode_image_dimensions(path) {
        file_sec.insert("Dimensions".into(), format!("{} x {}", w, h));
    }

    dirs.insert("File".into(), file_sec);

    // 2. Try to load EXIF
    if let Ok(file) = std::fs::File::open(path) {
        let mut buf_reader = std::io::BufReader::new(&file);
        if let Ok(exif) = Reader::new().read_from_container(&mut buf_reader) {
            for field in exif.fields() {
                let ifd_name = format!("EXIF IFD {}", field.ifd_num);
                let tag_name = field.tag.to_string();
                let value = field.display_value().to_string();
                dirs.entry(ifd_name).or_default().insert(tag_name, value);
            }
        }
    }

    Ok(dirs)
}
