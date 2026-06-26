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

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroUsize;

    #[test]
    fn test_new_thumbnail_cache_normal() {
        let cache = new_thumbnail_cache(10);
        assert_eq!(cache.cap(), NonZeroUsize::new(10).unwrap());
    }

    #[test]
    fn test_new_thumbnail_cache_zero_capacity() {
        let cache = new_thumbnail_cache(0);
        assert_eq!(cache.cap(), NonZeroUsize::new(1).unwrap());
    }
}
