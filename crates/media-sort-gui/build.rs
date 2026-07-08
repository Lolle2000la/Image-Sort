fn main() {
    #[cfg(target_os = "windows")]
    {
        embed_resource::compile("media-sort-gui.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
