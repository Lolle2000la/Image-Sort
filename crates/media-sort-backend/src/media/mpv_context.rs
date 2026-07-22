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
            let hwdec = CString::new("auto-safe").expect("static string contains no null bytes");
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
                    for line in list_str.lines() {
                        for ext in line.trim().split(',') {
                            let clean_ext = ext.trim().to_lowercase();
                            if !clean_ext.is_empty() {
                                extensions.insert(clean_ext);
                            }
                        }
                    }
                    mpv_free(ptr as *mut c_void);
                }
            }
        }
        extensions
    }
}

/// Rotates raw RGBA byte buffer by specified degrees (0, 90, 180, 270).
/// Returns `(new_width, new_height, new_rgba_bytes)`.
pub fn rotate_rgba(src_w: u32, src_h: u32, src: &[u8], rotate: i64) -> (u32, u32, Vec<u8>) {
    use rayon::prelude::*;
    let norm_rotate = rotate.rem_euclid(360);
    match norm_rotate {
        90 => {
            let dst_w = src_h;
            let dst_h = src_w;
            let mut dst = vec![0u8; (dst_w * dst_h * 4) as usize];
            let dst_stride = (dst_w * 4) as usize;

            dst.par_chunks_exact_mut(dst_stride)
                .enumerate()
                .for_each(|(dst_y, row)| {
                    let src_x = dst_y as u32;
                    row.chunks_exact_mut(4)
                        .enumerate()
                        .for_each(|(dst_x, pixel)| {
                            let src_y = src_h - 1 - dst_x as u32;
                            let src_idx = ((src_y * src_w + src_x) * 4) as usize;
                            if src_idx + 4 <= src.len() {
                                pixel.copy_from_slice(&src[src_idx..src_idx + 4]);
                            }
                        });
                });

            (dst_w, dst_h, dst)
        }
        180 => {
            let dst_w = src_w;
            let dst_h = src_h;
            let mut dst = vec![0u8; (dst_w * dst_h * 4) as usize];
            let dst_stride = (dst_w * 4) as usize;

            dst.par_chunks_exact_mut(dst_stride)
                .enumerate()
                .for_each(|(dst_y, row)| {
                    let src_y = src_h - 1 - dst_y as u32;
                    row.chunks_exact_mut(4)
                        .enumerate()
                        .for_each(|(dst_x, pixel)| {
                            let src_x = src_w - 1 - dst_x as u32;
                            let src_idx = ((src_y * src_w + src_x) * 4) as usize;
                            if src_idx + 4 <= src.len() {
                                pixel.copy_from_slice(&src[src_idx..src_idx + 4]);
                            }
                        });
                });

            (dst_w, dst_h, dst)
        }
        270 => {
            let dst_w = src_h;
            let dst_h = src_w;
            let mut dst = vec![0u8; (dst_w * dst_h * 4) as usize];
            let dst_stride = (dst_w * 4) as usize;

            dst.par_chunks_exact_mut(dst_stride)
                .enumerate()
                .for_each(|(dst_y, row)| {
                    let src_x = src_w - 1 - dst_y as u32;
                    row.chunks_exact_mut(4)
                        .enumerate()
                        .for_each(|(dst_x, pixel)| {
                            let src_y = dst_x as u32;
                            let src_idx = ((src_y * src_w + src_x) * 4) as usize;
                            if src_idx + 4 <= src.len() {
                                pixel.copy_from_slice(&src[src_idx..src_idx + 4]);
                            }
                        });
                });

            (dst_w, dst_h, dst)
        }
        _ => (src_w, src_h, src.to_vec()),
    }
}

impl Drop for MpvContext {
    fn drop(&mut self) {
        unsafe {
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

#[derive(Debug, Clone)]
pub enum VideoCommand {
    Load(std::path::PathBuf),
    Play,
    Pause,
    TogglePause,
    Seek(f64),
    SeekAbsolute(f64),
    SetMute(bool),
    SetVolume(f64),
    Stop,
    Deactivate,
}

#[derive(Debug, Clone)]
pub enum VideoEvent {
    FrameReady {
        path: std::path::PathBuf,
        width: u32,
        height: u32,
        rotation: i64,
        rgba: std::sync::Arc<Vec<u8>>,
    },
    PlaybackProgress {
        position: f64,
        duration: f64,
    },
    Muted(bool),
    Volume(f64),
    Paused(bool),
    LoadFailed {
        path: std::path::PathBuf,
        error: String,
    },
}

// SAFETY: MpvContext owns pointers to `mpv_handle` and `mpv_render_context`.
// libmpv is thread-safe, and we can safely send the handle and render context
// to other threads as long as we properly manage callbacks and lifetimes.
unsafe impl Send for MpvContext {}

// SAFETY: Synchronization of libmpv functions is handled internally by
// the C library, allowing concurrent read/write calls from different threads.
unsafe impl Sync for MpvContext {}

pub fn start_video_worker(
    cmd_rx: tokio::sync::mpsc::Receiver<VideoCommand>,
    event_tx: tokio::sync::mpsc::Sender<VideoEvent>,
) {
    tokio::spawn(run_video_worker(cmd_rx, event_tx));
}

pub async fn run_video_worker(
    mut cmd_rx: tokio::sync::mpsc::Receiver<VideoCommand>,
    event_tx: tokio::sync::mpsc::Sender<VideoEvent>,
) {
    let mut player = match MpvContext::new() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to create MpvContext: {e}");
            return;
        }
    };

    let (wakeup_tx, mut wakeup_rx) = tokio::sync::mpsc::channel(64);
    // SAFETY: player is a newly created, valid MpvContext instance, and the
    // callback context is dropped when player is dropped at the end of this loop/worker.
    unsafe {
        player.register_callback(wakeup_tx);
    }

    let max_buffer_size = (960 * 540 * 4) as usize;
    let mut pool = vec![
        std::sync::Arc::new(vec![0u8; max_buffer_size]),
        std::sync::Arc::new(vec![0u8; max_buffer_size]),
        std::sync::Arc::new(vec![0u8; max_buffer_size]),
    ];

    let mut current_video_path = std::path::PathBuf::new();
    let mut canonical_video_path: Option<std::path::PathBuf> = None;
    let mut cached_video_params: Option<(i32, i32, i64)> = None;
    let mut last_position = -1.0;
    let mut last_muted = false;
    let mut last_volume = -1.0;
    let mut last_paused = false;
    let mut is_active = false;

    let mut progress_interval = tokio::time::interval(std::time::Duration::from_millis(100));

    loop {
        tokio::select! {
            cmd_opt = cmd_rx.recv() => {
                let Some(cmd) = cmd_opt else { break; };
                match cmd {
                    VideoCommand::Load(path) => {
                        player.stop();
                        match player.load_file(&path) {
                            Ok(()) => {
                                player.set_paused(false);
                                player.drain_render_context();
                                is_active = true;
                                canonical_video_path = path.canonicalize().ok();
                                current_video_path = path;
                                cached_video_params = None;
                            }
                            Err(err) => {
                                let _ = event_tx
                                    .send(VideoEvent::LoadFailed {
                                        path: path.clone(),
                                        error: err,
                                    })
                                    .await;
                            }
                        }
                    }
                    VideoCommand::Play => {
                        player.set_paused(false);
                    }
                    VideoCommand::Pause => {
                        player.set_paused(true);
                    }
                    VideoCommand::TogglePause => {
                        player.toggle_pause();
                    }
                    VideoCommand::Seek(sec) => {
                        player.seek(sec);
                    }
                    VideoCommand::SeekAbsolute(sec) => {
                        player.seek_absolute(sec);
                    }
                    VideoCommand::SetMute(m) => {
                        player.set_mute(m);
                    }
                    VideoCommand::SetVolume(v) => {
                        player.set_volume(v);
                    }
                    VideoCommand::Stop => {
                        player.set_paused(true);
                        player.seek_absolute(0.0);
                        cached_video_params = None;
                    }
                    VideoCommand::Deactivate => {
                        player.set_paused(true);
                        cached_video_params = None;
                        // SAFETY: send "stop" command to release the current file handle and
                        // flush internal mpv caches, preventing file locks that would block
                        // rename/move/delete operations on the last-played video.
                        unsafe {
                            let mut cmd: [*const c_char; 2] =
                                [c"stop".as_ptr(), std::ptr::null()];
                            mpv_command(player.handle, cmd.as_mut_ptr());
                        }
                        is_active = false;
                    }
                }
            }

            _ = wakeup_rx.recv() => {
                // Drain any extra wakeup notifications in the channel for this tick
                while wakeup_rx.try_recv().is_ok() {}

                if is_active {
                    loop {
                        let flags = unsafe {
                            mpv_render_context_update(player.render_ctx)
                        };

                        if (flags & mpv_render_update_flag_MPV_RENDER_UPDATE_FRAME as u64) == 0 {
                            break;
                        }

                        if cached_video_params.is_none()
                            && let Some(current_p_str) = player.get_current_path()
                        {
                            let current_p = std::path::PathBuf::from(current_p_str);
                            let paths_match = current_p == current_video_path
                                || canonical_video_path.as_ref().is_some_and(|cp| current_p == *cp || current_p.canonicalize().ok().as_ref() == Some(cp));

                            if paths_match && player.is_video_ready() {
                                let (w, h) = player.get_video_size();
                                if w > 0 && h > 0 {
                                    let rotate = player.get_video_rotation();
                                    let norm_rotate = rotate.rem_euclid(360);
                                    let (eff_w, eff_h) = if norm_rotate == 90 || norm_rotate == 270 {
                                        (h, w)
                                    } else {
                                        (w, h)
                                    };

                                    let max_w = 960.0;
                                    let max_h = 540.0;
                                    let scale = (max_w / eff_w as f64).min(max_h / eff_h as f64).min(1.0);
                                    let render_unrot_w = ((w as f64 * scale) as i32) & !1;
                                    let render_unrot_h = ((h as f64 * scale) as i32) & !1;

                                    if render_unrot_w > 0 && render_unrot_h > 0 {
                                        cached_video_params = Some((render_unrot_w, render_unrot_h, rotate));
                                    }
                                }
                            }
                        }

                        if let Some((_, _, cached_rot)) = cached_video_params {
                            let current_rot = player.get_video_rotation();
                            if current_rot != cached_rot {
                                let (w, h) = player.get_video_size();
                                if w > 0 && h > 0 {
                                    let norm_rotate = current_rot.rem_euclid(360);
                                    let (eff_w, eff_h) = if norm_rotate == 90 || norm_rotate == 270 {
                                        (h, w)
                                    } else {
                                        (w, h)
                                    };

                                    let max_w = 960.0;
                                    let max_h = 540.0;
                                    let scale = (max_w / eff_w as f64).min(max_h / eff_h as f64).min(1.0);
                                    let render_unrot_w = ((w as f64 * scale) as i32) & !1;
                                    let render_unrot_h = ((h as f64 * scale) as i32) & !1;

                                    if render_unrot_w > 0 && render_unrot_h > 0 {
                                        cached_video_params = Some((render_unrot_w, render_unrot_h, current_rot));
                                    }
                                }
                            }
                        }

                        if let Some((render_unrot_w, render_unrot_h, rotate)) = cached_video_params {
                            let unrot_size = (render_unrot_w * render_unrot_h * 4) as usize;

                            // Find a free buffer in the pool (where we are the sole owner)
                            let mut free_buffer = None;
                            for buf in &mut pool {
                                if std::sync::Arc::strong_count(buf) == 1 {
                                    free_buffer = Some(buf);
                                    break;
                                }
                            }

                            if let Some(arc_buf) = free_buffer
                                && let Some(target_vec) = std::sync::Arc::get_mut(arc_buf)
                            {
                                target_vec.resize(unrot_size, 0);
                                if player.render_frame(render_unrot_w, render_unrot_h, target_vec).is_ok() {
                                    let _ = event_tx.try_send(VideoEvent::FrameReady {
                                        path: current_video_path.clone(),
                                        width: render_unrot_w as u32,
                                        height: render_unrot_h as u32,
                                        rotation: rotate,
                                        rgba: arc_buf.clone(),
                                    });
                                }
                            } else {
                                // If buffer pool is currently fully occupied by iced view,
                                // we render into a temporary scratch buffer to advance mpv's state machine.
                                let mut dummy = vec![0u8; unrot_size];
                                let _ = player.render_frame(render_unrot_w, render_unrot_h, &mut dummy);
                            }
                        } else {
                            break;
                        }
                    }
                }
            }

            _ = progress_interval.tick() => {
                if is_active {
                    let mut pos: f64 = 0.0;
                    let mut dur: f64 = 0.0;
                    unsafe {
                        mpv_get_property(
                            player.handle,
                            c"time-pos".as_ptr(),
                            mpv_format_MPV_FORMAT_DOUBLE,
                            &mut pos as *mut _ as *mut c_void,
                        );
                        mpv_get_property(
                            player.handle,
                            c"duration".as_ptr(),
                            mpv_format_MPV_FORMAT_DOUBLE,
                            &mut dur as *mut _ as *mut c_void,
                        );
                    }
                    if pos != last_position {
                        let _ = event_tx.send(VideoEvent::PlaybackProgress {
                            position: pos,
                            duration: dur,
                        }).await;
                        last_position = pos;
                    }

                    let mute = player.get_mute();
                    if mute != last_muted {
                        let _ = event_tx.send(VideoEvent::Muted(mute)).await;
                        last_muted = mute;
                    }
                    let vol = player.get_volume();
                    if vol != last_volume {
                        let _ = event_tx.send(VideoEvent::Volume(vol)).await;
                        last_volume = vol;
                    }
                    let paused = !player.is_playing();
                    if paused != last_paused {
                        let _ = event_tx.send(VideoEvent::Paused(paused)).await;
                        last_paused = paused;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate_rgba_0() {
        let src = vec![255, 0, 0, 255, 0, 255, 0, 255]; // 2x1: Red, Green
        let (w, h, dst) = rotate_rgba(2, 1, &src, 0);
        assert_eq!((w, h), (2, 1));
        assert_eq!(dst, src);
    }

    #[test]
    fn test_rotate_rgba_90() {
        let src = vec![255, 0, 0, 255, 0, 255, 0, 255]; // 2x1: Red, Green
        let (w, h, dst) = rotate_rgba(2, 1, &src, 90);
        assert_eq!((w, h), (1, 2));
        // 90 deg CW: (0,0) Red -> (0,0); (1,0) Green -> (0,1)
        assert_eq!(&dst[0..4], &[255, 0, 0, 255]);
        assert_eq!(&dst[4..8], &[0, 255, 0, 255]);
    }

    #[test]
    fn test_rotate_rgba_180() {
        let src = vec![255, 0, 0, 255, 0, 255, 0, 255]; // 2x1: Red, Green
        let (w, h, dst) = rotate_rgba(2, 1, &src, 180);
        assert_eq!((w, h), (2, 1));
        // 180 deg: Green, Red
        assert_eq!(&dst[0..4], &[0, 255, 0, 255]);
        assert_eq!(&dst[4..8], &[255, 0, 0, 255]);
    }

    #[test]
    fn test_rotate_rgba_270() {
        let src = vec![255, 0, 0, 255, 0, 255, 0, 255]; // 2x1: Red, Green
        let (w, h, dst) = rotate_rgba(2, 1, &src, 270);
        assert_eq!((w, h), (1, 2));
        // 270 deg CW: (0,0) Red -> (0,1); (1,0) Green -> (0,0)
        assert_eq!(&dst[0..4], &[0, 255, 0, 255]);
        assert_eq!(&dst[4..8], &[255, 0, 0, 255]);
    }
}
