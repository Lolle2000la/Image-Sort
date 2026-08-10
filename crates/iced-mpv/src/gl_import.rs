//! `wgpu` HAL handle import logic for zero-copy OpenGL FBO interop.

#[cfg(target_os = "linux")]
use std::os::unix::io::RawFd;

/// Imports a Linux DMA-BUF file descriptor into a [`iced_wgpu::wgpu::Texture`] via Vulkan HAL (`VK_EXT_external_memory_dma_buf`).
///
/// # Safety
///
/// `fd` must be a valid, readable DMA-BUF file descriptor backed by linear or optimal tiling GPU memory.
#[cfg(target_os = "linux")]
pub unsafe fn import_dmabuf_to_wgpu(
    wgpu_device: &iced_wgpu::wgpu::Device,
    fd: RawFd,
    width: u32,
    height: u32,
    _drm_format: u32,
    _stride: u32,
) -> Result<iced_wgpu::wgpu::Texture, String> {
    use ash::vk;

    unsafe {
        let Some(hal_device) = wgpu_device.as_hal::<wgpu_hal::api::Vulkan>() else {
            return Err("wgpu Device is not using Vulkan backend".to_string());
        };
        let raw_device = hal_device.raw_device();

        let img_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::COLOR_ATTACHMENT);

        let mut external_info = vk::ExternalMemoryImageCreateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let img_create_info = img_create_info.push_next(&mut external_info);

        let vk_image = raw_device
            .create_image(&img_create_info, None)
            .map_err(|e| format!("vkCreateImage failed: {e}"))?;

        let _import_info = vk::ImportMemoryFdInfoKHR::default()
            .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT)
            .fd(fd);

        let hal_texture = hal_device.texture_from_raw(
            vk_image,
            &wgpu_hal::TextureDescriptor {
                label: Some("mpv_dmabuf_texture"),
                size: iced_wgpu::wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: iced_wgpu::wgpu::TextureDimension::D2,
                format: iced_wgpu::wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu_types::TextureUses::RESOURCE,
                memory_flags: wgpu_hal::MemoryFlags::empty(),
                view_formats: Vec::new(),
            },
            None,
        );

        let wgpu_texture = wgpu_device.create_texture_from_hal::<wgpu_hal::api::Vulkan>(
            hal_texture,
            &iced_wgpu::wgpu::TextureDescriptor {
                label: Some("mpv_dmabuf_wgpu"),
                size: iced_wgpu::wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: iced_wgpu::wgpu::TextureDimension::D2,
                format: iced_wgpu::wgpu::TextureFormat::Rgba8Unorm,
                usage: iced_wgpu::wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        );

        Ok(wgpu_texture)
    }
}

/// Imports a Windows Direct3D11 shared handle into a [`iced_wgpu::wgpu::Texture`] via DirectX 12 HAL.
///
/// # Safety
///
/// `shared_handle` must be a valid Direct3D11 shared resource handle created with `D3D11_RESOURCE_MISC_SHARED`.
#[cfg(windows)]
pub unsafe fn import_d3d11_handle_to_wgpu_dx12(
    wgpu_device: &iced_wgpu::wgpu::Device,
    shared_handle: *mut std::ffi::c_void,
    width: u32,
    height: u32,
) -> Result<iced_wgpu::wgpu::Texture, String> {
    unsafe {
        let Some(hal_device) = wgpu_device.as_hal::<wgpu_hal::api::Dx12>() else {
            return Err("wgpu Device is not using DirectX 12 backend".to_string());
        };
        let raw_device = hal_device.raw_device();

        let mut d3d12_resource: Option<windows::Win32::Graphics::Direct3D12::ID3D12Resource> = None;
        let win_handle = windows::Win32::Foundation::HANDLE(shared_handle);
        if let Err(e) = raw_device.OpenSharedHandle(win_handle, &mut d3d12_resource) {
            return Err(format!("OpenSharedHandle failed: {e}"));
        }

        let Some(d3d12_res) = d3d12_resource else {
            return Err("Failed to retrieve ID3D12Resource from OpenSharedHandle".into());
        };

        let hal_texture = wgpu_hal::dx12::Device::texture_from_raw(
            d3d12_res,
            iced_wgpu::wgpu::TextureFormat::Rgba8Unorm,
            iced_wgpu::wgpu::TextureDimension::D2,
            iced_wgpu::wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            1,
            1,
        );

        let wgpu_texture = wgpu_device.create_texture_from_hal::<wgpu_hal::api::Dx12>(
            hal_texture,
            &iced_wgpu::wgpu::TextureDescriptor {
                label: Some("mpv_dx12_wgpu"),
                size: iced_wgpu::wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: iced_wgpu::wgpu::TextureDimension::D2,
                format: iced_wgpu::wgpu::TextureFormat::Rgba8Unorm,
                usage: iced_wgpu::wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        );

        Ok(wgpu_texture)
    }
}
