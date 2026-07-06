use std::sync::LazyLock;

static VIPS_APP: LazyLock<libvips::VipsApp> = LazyLock::new(|| {
    libvips::VipsApp::new("MediaSort", false).expect("Failed to initialize libvips")
});

pub fn ensure_init() {
    LazyLock::force(&VIPS_APP);
}
