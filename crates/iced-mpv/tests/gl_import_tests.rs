use iced_mpv::{Rotation, video_zero_copy_shader_view};

#[test]
fn test_video_zero_copy_shader_view_construction() {
    let _element = video_zero_copy_shader_view::<(), iced::Theme, iced::Renderer>(
        1920,
        1080,
        Rotation::R90,
        None,
    );
}

#[cfg(target_os = "linux")]
#[test]
fn test_import_dmabuf_invalid_fd_safety() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let instance = iced_wgpu::wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&iced_wgpu::wgpu::RequestAdapterOptions {
                power_preference: iced_wgpu::wgpu::PowerPreference::LowPower,
                compatible_surface: None,
                force_fallback_adapter: true,
            })
            .await;

        if let Ok(adapter) = adapter
            && let Ok((device, _queue)) = adapter
                .request_device(&iced_wgpu::wgpu::DeviceDescriptor::default())
                .await
        {
            let res = unsafe {
                iced_mpv::gl_import::import_dmabuf_to_wgpu(&device, -1, 100, 100, 0, 400)
            };
            assert!(res.is_err());
        }
    });
}
