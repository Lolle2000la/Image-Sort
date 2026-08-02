use libmpv_sys::*;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr;

const MPV_RENDER_PARAM_SW_SIZE: mpv_render_param_type = 17;
const MPV_RENDER_PARAM_SW_FORMAT: mpv_render_param_type = 18;
const MPV_RENDER_PARAM_SW_STRIDE: mpv_render_param_type = 19;
const MPV_RENDER_PARAM_SW_POINTER: mpv_render_param_type = 20;

pub struct MpvContext {
    pub handle: *mut mpv_handle,
    pub render_ctx: *mut mpv_render_context,
    callback_context_raw: *mut c_void,
}

impl MpvContext {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let handle = mpv_create();
            if handle.is_null() {
                return Err("Failed to create mpv instance".to_string());
            }

            // Set some default options
            let vo_name = CString::new("libmpv").expect("static string contains no null bytes");
            mpv_set_option_string(handle, c"vo".as_ptr(), vo_name.as_ptr());
            let keep_open = CString::new("yes").expect("static string contains no null bytes");
            mpv_set_option_string(handle, c"keep-open".as_ptr(), keep_open.as_ptr());
            let loop_file = CString::new("inf").expect("static string contains no null bytes");
            mpv_set_option_string(handle, c"loop-file".as_ptr(), loop_file.as_ptr());
            let hwdec = CString::new("auto-copy").expect("static string contains no null bytes");
            mpv_set_option_string(handle, c"hwdec".as_ptr(), hwdec.as_ptr());
            let no = CString::new("no").expect("static string contains no null bytes");
            mpv_set_option_string(handle, c"sub-auto".as_ptr(), no.as_ptr());
            mpv_set_option_string(handle, c"audio-file-auto".as_ptr(), no.as_ptr());
            mpv_set_option_string(handle, c"cache".as_ptr(), no.as_ptr());

            mpv_set_option_string(handle, c"video-rotate".as_ptr(), no.as_ptr());

            let vo_framedrop = CString::new("vo").expect("static string contains no null bytes");
            mpv_set_option_string(handle, c"framedrop".as_ptr(), vo_framedrop.as_ptr());
            let video_sync = CString::new("audio").expect("static string contains no null bytes");
            mpv_set_option_string(handle, c"video-sync".as_ptr(), video_sync.as_ptr());
            let video_timing_offset =
                CString::new("0").expect("static string contains no null bytes");
            mpv_set_option_string(
                handle,
                c"video-timing-offset".as_ptr(),
                video_timing_offset.as_ptr(),
            );
            mpv_set_option_string(handle, c"force-window".as_ptr(), no.as_ptr());
            mpv_set_option_string(handle, c"input-default-bindings".as_ptr(), no.as_ptr());

            let err = mpv_initialize(handle);
            if err < 0 {
                mpv_terminate_destroy(handle);
                return Err(format!("Failed to initialize mpv: {err}"));
            }

            // Create software render context
            let api_type = CString::new("sw").expect("static string contains no null bytes");
            let mut params = [
                mpv_render_param {
                    type_: mpv_render_param_type_MPV_RENDER_PARAM_API_TYPE,
                    data: api_type.as_ptr() as *mut c_void,
                },
                mpv_render_param {
                    type_: 0,
                    data: ptr::null_mut(),
                },
            ];

            let mut render_ctx: *mut mpv_render_context = ptr::null_mut();
            let err = mpv_render_context_create(&mut render_ctx, handle, params.as_mut_ptr());
            if err < 0 {
                mpv_terminate_destroy(handle);
                return Err(format!("Failed to create render context: {err}"));
            }

            Ok(Self {
                handle,
                render_ctx,
                callback_context_raw: ptr::null_mut(),
            })
        }
    }

    pub fn new_thumbnail_player() -> Result<Self, String> {
        unsafe {
            let handle = mpv_create();
            if handle.is_null() {
                return Err("Failed to create mpv instance".to_string());
            }

            let vo_name = CString::new("libmpv").expect("static string contains no null bytes");
            mpv_set_option_string(handle, c"vo".as_ptr(), vo_name.as_ptr());
            let keep_open = CString::new("yes").expect("static string contains no null bytes");
            mpv_set_option_string(handle, c"keep-open".as_ptr(), keep_open.as_ptr());
            let hwdec = CString::new("auto-copy").expect("static string contains no null bytes");
            mpv_set_option_string(handle, c"hwdec".as_ptr(), hwdec.as_ptr());

            let no = CString::new("no").expect("static string contains no null bytes");
            mpv_set_option_string(handle, c"aid".as_ptr(), no.as_ptr());
            mpv_set_option_string(handle, c"sid".as_ptr(), no.as_ptr());
            mpv_set_option_string(handle, c"sub-auto".as_ptr(), no.as_ptr());
            mpv_set_option_string(handle, c"audio-file-auto".as_ptr(), no.as_ptr());
            mpv_set_option_string(handle, c"cache".as_ptr(), no.as_ptr());
            mpv_set_option_string(handle, c"video-rotate".as_ptr(), no.as_ptr());
            mpv_set_option_string(handle, c"hr-seek".as_ptr(), no.as_ptr());
            mpv_set_option_string(handle, c"force-window".as_ptr(), no.as_ptr());
            mpv_set_option_string(handle, c"input-default-bindings".as_ptr(), no.as_ptr());

            let desync = CString::new("desync").expect("static string contains no null bytes");
            mpv_set_option_string(handle, c"video-sync".as_ptr(), desync.as_ptr());

            let err = mpv_initialize(handle);
            if err < 0 {
                mpv_terminate_destroy(handle);
                return Err(format!("Failed to initialize mpv: {err}"));
            }

            let api_type = CString::new("sw").expect("static string contains no null bytes");
            let mut params = [
                mpv_render_param {
                    type_: mpv_render_param_type_MPV_RENDER_PARAM_API_TYPE,
                    data: api_type.as_ptr() as *mut c_void,
                },
                mpv_render_param {
                    type_: 0,
                    data: ptr::null_mut(),
                },
            ];

            let mut render_ctx: *mut mpv_render_context = ptr::null_mut();
            let err = mpv_render_context_create(&mut render_ctx, handle, params.as_mut_ptr());
            if err < 0 {
                mpv_terminate_destroy(handle);
                return Err(format!("Failed to create render context: {err}"));
            }

            Ok(Self {
                handle,
                render_ctx,
                callback_context_raw: ptr::null_mut(),
            })
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that `self.render_ctx` is a valid render context and that
    /// the sender remains usable for the lifetime of the context.
    pub unsafe fn register_callback(&mut self, sender: tokio::sync::mpsc::Sender<()>) {
        let sender_box = Box::new(sender);
        self.callback_context_raw = Box::into_raw(sender_box) as *mut c_void;

        unsafe {
            mpv_render_context_set_update_callback(
                self.render_ctx,
                Some(mpv_wakeup_callback),
                self.callback_context_raw,
            );
        }
    }

    pub fn has_frame_ready(&self) -> bool {
        let flags = unsafe { mpv_render_context_update(self.render_ctx) };
        (flags & mpv_render_update_flag_MPV_RENDER_UPDATE_FRAME as u64) != 0
    }

    pub fn get_current_path(&self) -> Option<String> {
        unsafe {
            let mut path_ptr: *mut c_char = ptr::null_mut();
            let err = mpv_get_property(
                self.handle,
                c"path".as_ptr(),
                mpv_format_MPV_FORMAT_STRING,
                &mut path_ptr as *mut _ as *mut c_void,
            );
            if err >= 0 && !path_ptr.is_null() {
                let s = CStr::from_ptr(path_ptr).to_string_lossy().into_owned();
                mpv_free(path_ptr as *mut c_void);
                Some(s)
            } else {
                None
            }
        }
    }

    pub fn load_file(&mut self, path: &Path) -> Result<(), String> {
        unsafe {
            let path_str =
                CString::new(path.to_str().ok_or("Invalid path")?).map_err(|e| e.to_string())?;
            let mut cmd: [*const c_char; 3] =
                [c"loadfile".as_ptr(), path_str.as_ptr(), ptr::null()];
            let err = mpv_command(self.handle, cmd.as_mut_ptr());
            if err < 0 {
                return Err(format!("Failed to load file: {err}"));
            }
            Ok(())
        }
    }

    /// Send a `stop` command to release the current file and flush internal caches.
    /// Must be called before loading a new file to prevent mpv from entering an
    /// inconsistent state when files are switched rapidly.
    pub fn stop(&mut self) {
        unsafe {
            let mut cmd: [*const c_char; 2] = [c"stop".as_ptr(), ptr::null()];
            mpv_command(self.handle, cmd.as_mut_ptr());
            self.drain_render_context();
        }
    }

    pub fn drain_render_context(&self) {
        unsafe {
            for _ in 0..128 {
                if mpv_render_context_update(self.render_ctx) == 0 {
                    break;
                }
            }
        }
    }

    /// Returns true when the video output chain is fully initialized and ready to
    /// produce frames. Must be checked before calling `render_frame` to avoid
    /// `mp_image_crop` assertions during the transient initialization window
    /// between `load_file` and the first fully-formed frame.
    pub fn is_video_ready(&self) -> bool {
        unsafe {
            let mut ptr: *mut c_char = ptr::null_mut();
            let err = mpv_get_property(
                self.handle,
                c"video-out-params".as_ptr(),
                mpv_format_MPV_FORMAT_STRING,
                &mut ptr as *mut _ as *mut c_void,
            );
            if err >= 0 && !ptr.is_null() {
                mpv_free(ptr as *mut c_void);
                true
            } else {
                false
            }
        }
    }

    pub fn get_video_size(&self) -> (i64, i64) {
        unsafe {
            let mut width: i64 = 0;
            let mut height: i64 = 0;
            mpv_get_property(
                self.handle,
                c"video-params/w".as_ptr(),
                mpv_format_MPV_FORMAT_INT64,
                &mut width as *mut _ as *mut c_void,
            );
            mpv_get_property(
                self.handle,
                c"video-params/h".as_ptr(),
                mpv_format_MPV_FORMAT_INT64,
                &mut height as *mut _ as *mut c_void,
            );
            (width, height)
        }
    }

    pub fn get_video_rotation(&self) -> i64 {
        if let Some(path_str) = self.get_current_path()
            && let Some(rot) = crate::engine::rotation::detect_video_rotation(Path::new(&path_str))
            && rot != 0
        {
            return rot.rem_euclid(360);
        }

        unsafe {
            let mut rotate: i64 = 0;
            // 1. Check video-params/rotate
            let mut err = mpv_get_property(
                self.handle,
                c"video-params/rotate".as_ptr(),
                mpv_format_MPV_FORMAT_INT64,
                &mut rotate as *mut _ as *mut c_void,
            );
            if err >= 0 && rotate != 0 {
                return rotate.rem_euclid(360);
            }

            // 2. Check video-out-params/rotate
            err = mpv_get_property(
                self.handle,
                c"video-out-params/rotate".as_ptr(),
                mpv_format_MPV_FORMAT_INT64,
                &mut rotate as *mut _ as *mut c_void,
            );
            if err >= 0 && rotate != 0 {
                return rotate.rem_euclid(360);
            }

            // 3. Check track-list/0/demux-rotation
            err = mpv_get_property(
                self.handle,
                c"track-list/0/demux-rotation".as_ptr(),
                mpv_format_MPV_FORMAT_INT64,
                &mut rotate as *mut _ as *mut c_void,
            );
            if err >= 0 && rotate != 0 {
                return rotate.rem_euclid(360);
            }

            // 4. Check track-list/0/user-rotation
            err = mpv_get_property(
                self.handle,
                c"track-list/0/user-rotation".as_ptr(),
                mpv_format_MPV_FORMAT_INT64,
                &mut rotate as *mut _ as *mut c_void,
            );
            if err >= 0 && rotate != 0 {
                return rotate.rem_euclid(360);
            }

            // 5. Check metadata string tags
            let meta_keys = [
                c"metadata/by-key/rotate".as_ptr(),
                c"metadata/by-key/ROTATE".as_ptr(),
                c"metadata/by-key/orientation".as_ptr(),
                c"metadata/by-key/ORIENTATION".as_ptr(),
                c"metadata/by-key/com.apple.quicktime.orientation".as_ptr(),
            ];

            for key in meta_keys {
                let mut str_ptr: *mut c_char = ptr::null_mut();
                err = mpv_get_property(
                    self.handle,
                    key,
                    mpv_format_MPV_FORMAT_STRING,
                    &mut str_ptr as *mut _ as *mut c_void,
                );
                if err >= 0 && !str_ptr.is_null() {
                    let s = CStr::from_ptr(str_ptr).to_string_lossy();
                    let parsed = s.trim().parse::<i64>().unwrap_or(0);
                    mpv_free(str_ptr as *mut c_void);
                    if parsed != 0 {
                        return parsed.rem_euclid(360);
                    }
                }
            }

            0
        }
    }

    pub fn render_frame(&self, width: i32, height: i32, buffer: &mut [u8]) -> Result<(), String> {
        let required = (width as usize) * (height as usize) * 4;
        if buffer.len() < required {
            return Err(format!(
                "Buffer too small: {} bytes, need {} for {}x{} RGBA",
                buffer.len(),
                required,
                width,
                height
            ));
        }
        unsafe {
            let format = CString::new("rgba").expect("static string contains no null bytes");
            let mut size: [c_int; 2] = [width, height];
            let mut stride = (width * 4) as usize;

            let mut params = [
                mpv_render_param {
                    type_: MPV_RENDER_PARAM_SW_SIZE,
                    data: size.as_mut_ptr() as *mut c_void,
                },
                mpv_render_param {
                    type_: MPV_RENDER_PARAM_SW_FORMAT,
                    data: format.as_ptr() as *mut c_void,
                },
                mpv_render_param {
                    type_: MPV_RENDER_PARAM_SW_STRIDE,
                    data: &mut stride as *mut _ as *mut c_void,
                },
                mpv_render_param {
                    type_: MPV_RENDER_PARAM_SW_POINTER,
                    data: buffer.as_mut_ptr() as *mut c_void,
                },
                mpv_render_param {
                    type_: 0,
                    data: ptr::null_mut(),
                },
            ];

            let err = mpv_render_context_render(self.render_ctx, params.as_mut_ptr());
            if err < 0 {
                return Err(format!("Failed to render frame: {err}"));
            }
            Ok(())
        }
    }

    pub fn is_playing(&self) -> bool {
        unsafe {
            let mut paused: c_int = 0;
            mpv_get_property(
                self.handle,
                c"pause".as_ptr(),
                mpv_format_MPV_FORMAT_FLAG,
                &mut paused as *mut _ as *mut c_void,
            );
            paused == 0
        }
    }

    pub fn set_paused(&mut self, paused: bool) {
        unsafe {
            let val: c_int = if paused { 1 } else { 0 };
            mpv_set_property(
                self.handle,
                c"pause".as_ptr(),
                mpv_format_MPV_FORMAT_FLAG,
                &val as *const _ as *mut c_void,
            );
        }
    }

    pub fn toggle_pause(&mut self) {
        unsafe {
            let mut cmd: [*const c_char; 3] = [c"cycle".as_ptr(), c"pause".as_ptr(), ptr::null()];
            mpv_command(self.handle, cmd.as_mut_ptr());
        }
    }

    pub fn seek(&mut self, seconds: f64) {
        unsafe {
            let sec_str = CString::new(seconds.to_string())
                .expect("floating point number string contains no null bytes");
            let mut cmd: [*const c_char; 4] = [
                c"seek".as_ptr(),
                sec_str.as_ptr(),
                c"relative".as_ptr(),
                ptr::null(),
            ];
            mpv_command(self.handle, cmd.as_mut_ptr());
        }
    }

    pub fn seek_absolute(&mut self, seconds: f64) {
        unsafe {
            let sec_str = CString::new(seconds.to_string())
                .expect("floating point number string contains no null bytes");
            let mut cmd: [*const c_char; 5] = [
                c"seek".as_ptr(),
                sec_str.as_ptr(),
                c"absolute".as_ptr(),
                c"exact".as_ptr(),
                ptr::null(),
            ];
            mpv_command(self.handle, cmd.as_mut_ptr());
        }
    }

    pub fn set_volume(&mut self, volume: f64) {
        unsafe {
            mpv_set_property(
                self.handle,
                c"volume".as_ptr(),
                mpv_format_MPV_FORMAT_DOUBLE,
                &volume as *const _ as *mut c_void,
            );
        }
    }

    pub fn set_mute(&mut self, mute: bool) {
        unsafe {
            let val: c_int = if mute { 1 } else { 0 };
            mpv_set_property(
                self.handle,
                c"mute".as_ptr(),
                mpv_format_MPV_FORMAT_FLAG,
                &val as *const _ as *mut c_void,
            );
        }
    }

    pub fn get_volume(&self) -> f64 {
        unsafe {
            let mut vol: f64 = 0.0;
            mpv_get_property(
                self.handle,
                c"volume".as_ptr(),
                mpv_format_MPV_FORMAT_DOUBLE,
                &mut vol as *mut _ as *mut c_void,
            );
            vol
        }
    }

    pub fn get_mute(&self) -> bool {
        unsafe {
            let mut mute: c_int = 0;
            mpv_get_property(
                self.handle,
                c"mute".as_ptr(),
                mpv_format_MPV_FORMAT_FLAG,
                &mut mute as *mut _ as *mut c_void,
            );
            mute != 0
        }
    }

    /// Query the set of file extensions compiled into the underlying FFmpeg layer of
    /// `libmpv`. Returns an empty set if the context cannot be created or the
    /// `demuxer-lavf-list` property cannot be read.
    pub fn query_supported_extensions() -> std::collections::HashSet<String> {
        let mut extensions = std::collections::HashSet::new();
        if let Ok(ctx) = Self::new() {
            unsafe {
                let mut ptr: *mut c_char = ptr::null_mut();
                let err = mpv_get_property(
                    ctx.handle,
                    c"demuxer-lavf-list".as_ptr(),
                    mpv_format_MPV_FORMAT_STRING,
                    &mut ptr as *mut _ as *mut c_void,
                );
                if err >= 0 && !ptr.is_null() {
                    let list_str = CStr::from_ptr(ptr).to_string_lossy();
                    extensions.extend(
                        list_str
                            .lines()
                            .flat_map(|line| line.trim().split(','))
                            .map(|ext| ext.trim().to_lowercase())
                            .filter(|ext| !ext.is_empty()),
                    );
                    mpv_free(ptr as *mut c_void);
                }
            }
        }
        extensions
    }
}

impl Drop for MpvContext {
    fn drop(&mut self) {
        unsafe {
            let mut cmd: [*const c_char; 2] = [c"stop".as_ptr(), ptr::null()];
            mpv_command(self.handle, cmd.as_mut_ptr());

            mpv_render_context_set_update_callback(self.render_ctx, None, ptr::null_mut());

            if !self.callback_context_raw.is_null() {
                let _sender_box =
                    Box::from_raw(self.callback_context_raw as *mut tokio::sync::mpsc::Sender<()>);
            }

            mpv_render_context_free(self.render_ctx);
            mpv_terminate_destroy(self.handle);
        }
    }
}

/// # Safety
///
/// `cb_ctx` must be a valid pointer to a `tokio::sync::mpsc::Sender<()>` that was
/// previously registered via `mpv_render_context_set_update_callback`.
pub unsafe extern "C" fn mpv_wakeup_callback(cb_ctx: *mut c_void) {
    let sender = cb_ctx as *const tokio::sync::mpsc::Sender<()>;
    if let Some(tx) = unsafe { sender.as_ref() } {
        let _ = tx.try_send(());
    }
}

// SAFETY: MpvContext owns pointers to `mpv_handle` and `mpv_render_context`.
// libmpv is thread-safe, and we can safely send the handle and render context
// to other threads as long as we properly manage callbacks and lifetimes.
unsafe impl Send for MpvContext {}

// SAFETY: Synchronization of libmpv functions is handled internally by
// the C library, allowing concurrent read/write calls from different threads.
unsafe impl Sync for MpvContext {}
