use std::num::NonZeroUsize;
use std::path::PathBuf;

use lru::LruCache;

#[allow(dead_code)]
pub type ThumbnailCache = LruCache<PathBuf, Vec<u8>>;

#[allow(dead_code)]
pub fn new_thumbnail_cache(max_entries: usize) -> ThumbnailCache {
    let size = NonZeroUsize::new(max_entries.max(1)).unwrap();
    LruCache::new(size)
}
