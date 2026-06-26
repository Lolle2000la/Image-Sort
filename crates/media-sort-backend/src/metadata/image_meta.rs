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

    let file = std::fs::File::open(path)?;
    let mut buf_reader = std::io::BufReader::new(&file);
    let exif = Reader::new()
        .read_from_container(&mut buf_reader)
        .map_err(|_| MetadataError::ExtractionFailed)?;

    let mut dirs: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for field in exif.fields() {
        let ifd_name = field.ifd_num.to_string();
        let tag_name = field.tag.to_string();
        let value = field.display_value().to_string();

        dirs.entry(ifd_name).or_default().insert(tag_name, value);
    }

    Ok(dirs)
}
