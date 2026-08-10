//! OpenGL-to-`wgpu` zero-copy interop context setup and GPU handle export for `libmpv`.
//!
//! Provides [`create_gl_render_context`], offscreen OpenGL context management,
//! and OS-level memory handle export ([`SharedGpuFrameHandle`]) for zero-copy
//! video frame sharing with `wgpu`.

use libmpv_sys::*;
use std::ffi::{CString, c_char, c_void};

/// An OS-level GPU memory handle backing an OpenGL Framebuffer Object (FBO)
/// rendered by `libmpv`.
#[derive(Debug)]
pub enum SharedGpuFrameHandle {
    /// Linux DMA-BUF exported file descriptor (`VK_EXT_external_memory_dma_buf`).
    #[cfg(target_os = "linux")]
    DmaBuf {
        fd: std::os::unix::io::RawFd,
        width: u32,
        height: u32,
        drm_format: u32,
        stride: u32,
        offset: u32,
    },
    /// Windows Direct3D11 shared handle exported via `WGL_NV_DX_interop`.
    #[cfg(windows)]
    D3D11 {
        handle: *mut c_void,
        width: u32,
        height: u32,
    },
    /// Fallback indicator when hardware zero-copy interop is unavailable.
    SoftwareFallback,
}

unsafe impl Send for SharedGpuFrameHandle {}
unsafe impl Sync for SharedGpuFrameHandle {}

/// Creates a `libmpv` render context initialized for OpenGL rendering (`MPV_RENDER_API_TYPE_OPENGL`).
///
/// # Safety
///
/// `mpv` must be a valid, initialized `mpv_handle`. `get_proc_addr` must be a valid C function
/// pointer for resolving OpenGL extension function addresses in the current OpenGL context.
pub unsafe fn create_gl_render_context(
    mpv: *mut mpv_handle,
    get_proc_addr: unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void,
    get_proc_addr_ctx: *mut c_void,
) -> Result<*mut mpv_render_context, crate::MpvError> {
    if mpv.is_null() {
        return Err(crate::MpvError::CreateFailed);
    }
    let mut gl_init_params = mpv_opengl_init_params {
        get_proc_address: Some(get_proc_addr),
        get_proc_address_ctx: get_proc_addr_ctx,
        extra_exts: std::ptr::null(),
    };

    let api_type = CString::new("opengl").map_err(|_| crate::MpvError::InvalidCString)?;
    let mut params = [
        mpv_render_param {
            type_: mpv_render_param_type_MPV_RENDER_PARAM_API_TYPE,
            data: api_type.as_ptr() as *mut _,
        },
        mpv_render_param {
            type_: mpv_render_param_type_MPV_RENDER_PARAM_OPENGL_INIT_PARAMS,
            data: &mut gl_init_params as *mut _ as *mut _,
        },
        mpv_render_param {
            type_: 0,
            data: std::ptr::null_mut(),
        },
    ];

    let mut render_ctx: *mut mpv_render_context = std::ptr::null_mut();
    let err = unsafe { mpv_render_context_create(&mut render_ctx, mpv, params.as_mut_ptr()) };
    if err < 0 {
        Err(crate::MpvError::RenderContextCreate(err))
    } else {
        Ok(render_ctx)
    }
}

/// Offscreen OpenGL FBO render configuration for `libmpv`.
#[derive(Debug, Clone, Copy)]
pub struct GlFboTarget {
    pub fbo_id: u32,
    pub width: u32,
    pub height: u32,
    pub internal_format: u32,
}

/// Renders a single frame from `libmpv` into an offscreen OpenGL FBO target.
///
/// # Safety
///
/// `render_ctx` must be a valid `mpv_render_context` created with OpenGL API type.
/// The calling thread must have the target OpenGL context bound as current.
pub unsafe fn render_gl_fbo(
    render_ctx: *mut mpv_render_context,
    target: GlFboTarget,
) -> Result<(), crate::MpvError> {
    if render_ctx.is_null() {
        return Err(crate::MpvError::RenderContextCreate(-1));
    }
    let mut fbo = mpv_opengl_fbo {
        fbo: target.fbo_id as i32,
        w: target.width as i32,
        h: target.height as i32,
        internal_format: target.internal_format as i32,
    };

    let mut params = [
        mpv_render_param {
            type_: mpv_render_param_type_MPV_RENDER_PARAM_OPENGL_FBO,
            data: &mut fbo as *mut _ as *mut c_void,
        },
        mpv_render_param {
            type_: 0,
            data: std::ptr::null_mut(),
        },
    ];

    let err = unsafe { mpv_render_context_render(render_ctx, params.as_mut_ptr()) };
    if err < 0 {
        Err(crate::MpvError::RenderFrame(err))
    } else {
        Ok(())
    }
}

/// EGL FFI bindings and DMA-BUF export definitions for Linux platforms.
#[cfg(target_os = "linux")]
pub mod egl_export {
    use super::*;

    pub type EGLDisplay = *mut c_void;
    pub type EGLContext = *mut c_void;
    pub type EGLImageKHR = *mut c_void;
    pub type EGLClientBuffer = *mut c_void;
    pub type EGLenum = u32;
    pub type EGLBoolean = u32;
    pub type EGLint = i32;

    pub const EGL_GL_TEXTURE_2D_KHR: EGLenum = 0x30B1;
    pub const EGL_TRUE: EGLBoolean = 1;

    pub type PfnEglCreateImageKHR = unsafe extern "C" fn(
        dpy: EGLDisplay,
        ctx: EGLContext,
        target: EGLenum,
        buffer: EGLClientBuffer,
        attrib_list: *const EGLint,
    ) -> EGLImageKHR;

    pub type PfnEglExportDMABUFImageMESA = unsafe extern "C" fn(
        dpy: EGLDisplay,
        image: EGLImageKHR,
        fds: *mut i32,
        strides: *mut i32,
        offsets: *mut i32,
    ) -> EGLBoolean;

    pub type PfnEglDestroyImageKHR =
        unsafe extern "C" fn(dpy: EGLDisplay, image: EGLImageKHR) -> EGLBoolean;

    /// Exports an OpenGL texture backed by EGL to a DMA-BUF file descriptor.
    ///
    /// # Safety
    ///
    /// Requires valid EGL display/context pointers and EGL_MESA_image_dma_buf_export extension support.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn export_gl_texture_to_dmabuf(
        create_image: PfnEglCreateImageKHR,
        export_dmabuf: PfnEglExportDMABUFImageMESA,
        destroy_image: PfnEglDestroyImageKHR,
        display: EGLDisplay,
        context: EGLContext,
        gl_texture_id: u32,
        width: u32,
        height: u32,
        drm_format: u32,
    ) -> Result<SharedGpuFrameHandle, crate::MpvError> {
        let egl_image = unsafe {
            create_image(
                display,
                context,
                EGL_GL_TEXTURE_2D_KHR,
                gl_texture_id as usize as EGLClientBuffer,
                std::ptr::null(),
            )
        };

        if egl_image.is_null() {
            return Err(crate::MpvError::Other("eglCreateImageKHR failed".into()));
        }

        let mut fd: i32 = -1;
        let mut stride: i32 = 0;
        let mut offset: i32 = 0;

        let res = unsafe { export_dmabuf(display, egl_image, &mut fd, &mut stride, &mut offset) };
        unsafe { destroy_image(display, egl_image) };

        if res != EGL_TRUE || fd < 0 {
            return Err(crate::MpvError::Other(
                "eglExportDMABUFImageMESA failed".into(),
            ));
        }

        Ok(SharedGpuFrameHandle::DmaBuf {
            fd,
            width,
            height,
            drm_format,
            stride: stride as u32,
            offset: offset as u32,
        })
    }
}

/// WGL & Direct3D11 interop definitions for Windows platforms.
#[cfg(windows)]
pub mod wgl_export {
    use super::*;

    pub const WGL_ACCESS_WRITE_DISCARD_NV: u32 = 0x0002;
    pub const GL_RENDERBUFFER: u32 = 0x8D41;

    /// Shared handle container for Direct3D11 interop texture.
    #[derive(Debug)]
    pub struct D3D11GlSharedResource {
        pub d3d11_device: *mut c_void,
        pub d3d11_texture: *mut c_void,
        pub shared_handle: *mut c_void,
        pub wgl_device: *mut c_void,
        pub wgl_object: *mut c_void,
    }

    unsafe impl Send for D3D11GlSharedResource {}
    unsafe impl Sync for D3D11GlSharedResource {}
}
