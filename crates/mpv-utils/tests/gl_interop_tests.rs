use mpv_utils::{GlFboTarget, SharedGpuFrameHandle, create_gl_render_context, render_gl_fbo};
use std::ffi::c_void;

extern "C" fn mock_get_proc_addr(_ctx: *mut c_void, _name: *const std::ffi::c_char) -> *mut c_void {
    std::ptr::null_mut()
}

#[test]
fn test_gl_fbo_target_creation() {
    let target = GlFboTarget {
        fbo_id: 1,
        width: 1920,
        height: 1080,
        internal_format: 0x8058, // GL_RGBA8
    };

    assert_eq!(target.fbo_id, 1);
    assert_eq!(target.width, 1920);
    assert_eq!(target.height, 1080);
    assert_eq!(target.internal_format, 0x8058);
}

#[test]
fn test_shared_gpu_frame_handle_software_fallback() {
    let handle = SharedGpuFrameHandle::SoftwareFallback;
    assert!(matches!(handle, SharedGpuFrameHandle::SoftwareFallback));
}

#[cfg(target_os = "linux")]
#[test]
fn test_shared_gpu_frame_handle_dmabuf_variant() {
    let handle = SharedGpuFrameHandle::DmaBuf {
        fd: 42,
        width: 1280,
        height: 720,
        drm_format: 0x34325241,
        stride: 5120,
        offset: 0,
    };

    if let SharedGpuFrameHandle::DmaBuf {
        fd,
        width,
        height,
        drm_format,
        stride,
        offset,
    } = handle
    {
        assert_eq!(fd, 42);
        assert_eq!(width, 1280);
        assert_eq!(height, 720);
        assert_eq!(drm_format, 0x34325241);
        assert_eq!(stride, 5120);
        assert_eq!(offset, 0);
    } else {
        panic!("expected DmaBuf handle");
    }
}

#[test]
fn test_create_gl_render_context_null_handle_returns_error() {
    let result = unsafe {
        create_gl_render_context(
            std::ptr::null_mut(),
            mock_get_proc_addr,
            std::ptr::null_mut(),
        )
    };

    assert!(result.is_err());
}

#[test]
fn test_render_gl_fbo_null_context_returns_error() {
    let target = GlFboTarget {
        fbo_id: 0,
        width: 640,
        height: 480,
        internal_format: 0,
    };

    let result = unsafe { render_gl_fbo(std::ptr::null_mut(), target) };
    assert!(result.is_err());
}
