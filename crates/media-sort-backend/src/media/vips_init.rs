use std::collections::HashSet;
use std::ffi::{CStr, c_char};
use std::sync::LazyLock;

static VIPS_APP: LazyLock<libvips::VipsApp> = LazyLock::new(|| {
    libvips::VipsApp::new("MediaSort", false).expect("Failed to initialize libvips")
});

pub fn ensure_init() {
    LazyLock::force(&VIPS_APP);
}

unsafe extern "C" {
    fn g_strfreev(str_array: *mut *mut c_char);
}

/// Query the running libvips for all file suffixes it can decode natively.
pub fn get_supported_suffixes() -> Vec<String> {
    ensure_init();
    let mut exts = Vec::new();

    unsafe {
        let array_ptr = libvips::bindings::vips_foreign_get_suffixes();
        if !array_ptr.is_null() {
            let mut cur = array_ptr;
            while !(*cur).is_null() {
                if let Ok(s) = CStr::from_ptr(*cur).to_str() {
                    exts.push(s.to_string());
                }
                cur = cur.add(1);
            }
            g_strfreev(array_ptr);
        }
    }

    exts.sort();
    exts
}

/// Returns format names (without leading dot) deduplicated, e.g. "jpeg" instead of
/// both ".jpg" and ".jpeg".
pub fn get_supported_format_names() -> Vec<String> {
    get_supported_suffixes()
        .into_iter()
        .map(|s| s.trim_start_matches('.').to_lowercase())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}
