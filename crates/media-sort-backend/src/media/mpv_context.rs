use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr;
use libmpv_sys::*;

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
            let vo_name = CString::new("libmpv").unwrap();
            mpv_set_option_string(handle, b"vo\0".as_ptr() as *const c_char, vo_name.as_ptr());
            let keep_open = CString::new("yes").unwrap();
            mpv_set_option_string(handle, b"keep-open\0".as_ptr() as *const c_char, keep_open.as_ptr());
            let loop_file = CString::new("inf").unwrap();
            mpv_set_option_string(handle, b"loop-file\0".as_ptr() as *const c_char, loop_file.as_ptr());
            let hwdec = CString::new("auto").unwrap();
            mpv_set_option_string(handle, b"hwdec\0".as_ptr() as *const c_char, hwdec.as_ptr());

            let no = CString::new("no").unwrap();
            mpv_set_option_string(handle, b"sub-auto\0".as_ptr() as *const c_char, no.as_ptr());
            mpv_set_option_string(handle, b"audio-file-auto\0".as_ptr() as *const c_char, no.as_ptr());
            mpv_set_option_string(handle, b"cache\0".as_ptr() as *const c_char, no.as_ptr());

            let err = mpv_initialize(handle);
            if err < 0 {
                mpv_terminate_destroy(handle);
                return Err(format!("Failed to initialize mpv: {err}"));
            }

            // Create software render context
            let api_type = CString::new("sw").unwrap();
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

    pub unsafe fn register_callback(&mut self, sender: tokio::sync::mpsc::Sender<()>) {
        let sender_box = Box::new(sender);
        self.callback_context_raw = Box::into_raw(sender_box) as *mut c_void;

        mpv_render_context_set_update_callback(
            self.render_ctx,
            Some(mpv_wakeup_callback),
            self.callback_context_raw,
        );
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
                b"path\0".as_ptr() as *const c_char,
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
            let path_str = CString::new(path.to_str().ok_or("Invalid path")?).map_err(|e| e.to_string())?;
            let mut cmd: [*const c_char; 3] = [
                b"loadfile\0".as_ptr() as *const c_char,
                path_str.as_ptr(),
                ptr::null(),
            ];
            let err = mpv_command(self.handle, cmd.as_mut_ptr());
            if err < 0 {
                return Err(format!("Failed to load file: {err}"));
            }
            Ok(())
        }
    }

    pub fn get_video_size(&self) -> (i64, i64) {
        unsafe {
            let mut width: i64 = 0;
            let mut height: i64 = 0;
            mpv_get_property(
                self.handle,
                b"width\0".as_ptr() as *const c_char,
                mpv_format_MPV_FORMAT_INT64,
                &mut width as *mut _ as *mut c_void,
            );
            mpv_get_property(
                self.handle,
                b"height\0".as_ptr() as *const c_char,
                mpv_format_MPV_FORMAT_INT64,
                &mut height as *mut _ as *mut c_void,
            );
            (width, height)
        }
    }

    pub fn render_frame(&self, width: i32, height: i32, buffer: &mut [u8]) -> Result<(), String> {
        unsafe {
            let format = CString::new("rgba").unwrap();
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
                b"pause\0".as_ptr() as *const c_char,
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
                b"pause\0".as_ptr() as *const c_char,
                mpv_format_MPV_FORMAT_FLAG,
                &val as *const _ as *mut c_void,
            );
        }
    }

    pub fn toggle_pause(&mut self) {
        unsafe {
            let mut cmd: [*const c_char; 3] = [
                b"cycle\0".as_ptr() as *const c_char,
                b"pause\0".as_ptr() as *const c_char,
                ptr::null(),
            ];
            mpv_command(self.handle, cmd.as_mut_ptr());
        }
    }

    pub fn seek(&mut self, seconds: f64) {
        unsafe {
            let sec_str = CString::new(seconds.to_string()).unwrap();
            let mut cmd: [*const c_char; 4] = [
                b"seek\0".as_ptr() as *const c_char,
                sec_str.as_ptr(),
                b"relative\0".as_ptr() as *const c_char,
                ptr::null(),
            ];
            mpv_command(self.handle, cmd.as_mut_ptr());
        }
    }

    pub fn seek_absolute(&mut self, seconds: f64) {
        unsafe {
            let sec_str = CString::new(seconds.to_string()).unwrap();
            let mut cmd: [*const c_char; 5] = [
                b"seek\0".as_ptr() as *const c_char,
                sec_str.as_ptr(),
                b"absolute\0".as_ptr() as *const c_char,
                b"exact\0".as_ptr() as *const c_char,
                ptr::null(),
            ];
            mpv_command(self.handle, cmd.as_mut_ptr());
        }
    }

    pub fn set_volume(&mut self, volume: f64) {
        unsafe {
            mpv_set_property(
                self.handle,
                b"volume\0".as_ptr() as *const c_char,
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
                b"mute\0".as_ptr() as *const c_char,
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
                b"volume\0".as_ptr() as *const c_char,
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
                b"mute\0".as_ptr() as *const c_char,
                mpv_format_MPV_FORMAT_FLAG,
                &mut mute as *mut _ as *mut c_void,
            );
            mute != 0
        }
    }
}

impl Drop for MpvContext {
    fn drop(&mut self) {
        unsafe {
            mpv_render_context_set_update_callback(
                self.render_ctx,
                None,
                ptr::null_mut(),
            );

            if !self.callback_context_raw.is_null() {
                let _sender_box = Box::from_raw(
                    self.callback_context_raw as *mut tokio::sync::mpsc::Sender<()>,
                );
            }

            mpv_render_context_free(self.render_ctx);
            mpv_terminate_destroy(self.handle);
        }
    }
}

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
        rgba: Vec<u8>,
    },
    PlaybackProgress {
        position: f64,
        duration: f64,
    },
    Muted(bool),
    Volume(f64),
    Paused(bool),
}

unsafe impl Send for MpvContext {}
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
    unsafe {
        player.register_callback(wakeup_tx);
    }

    let mut buffer = Vec::new();
    let mut current_video_path = std::path::PathBuf::new();
    let mut last_position = -1.0;
    let mut last_muted = false;
    let mut last_volume = -1.0;
    let mut last_paused = false;
    let mut is_active = false;
    let mut is_loading = false;

    let mut progress_interval = tokio::time::interval(std::time::Duration::from_millis(100));

    loop {
        tokio::select! {
            cmd_opt = cmd_rx.recv() => {
                let Some(cmd) = cmd_opt else { break; };
                match cmd {
                    VideoCommand::Load(path) => {
                        if let Ok(()) = player.load_file(&path) {
                            player.set_paused(false);
                            is_active = true;
                            current_video_path = path;
                            is_loading = true;
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
                    }
                    VideoCommand::Deactivate => {
                        player.set_paused(true);
                        is_active = false;
                    }
                }
            }

            _ = wakeup_rx.recv() => {
                if is_active {
                    let mut should_render = true;
                    if is_loading {
                        if let Some(ref current_p) = player.get_current_path() {
                            let target_p = current_video_path.to_string_lossy();
                            if current_p == &target_p {
                                is_loading = false;
                            } else {
                                should_render = false;
                            }
                        } else {
                            should_render = false;
                        }
                    }

                    if should_render {
                        let flags = unsafe {
                            mpv_render_context_update(player.render_ctx)
                        };
                        if (flags & mpv_render_update_flag_MPV_RENDER_UPDATE_FRAME as u64) != 0 {
                            let (w, h) = player.get_video_size();
                            if w > 0 && h > 0 {
                                let max_w = 960.0;
                                let max_h = 540.0;
                                let scale = (max_w / w as f64).min(max_h / h as f64).min(1.0);
                                let render_w = (w as f64 * scale) as i32;
                                let render_h = (h as f64 * scale) as i32;

                                let size = (render_w * render_h * 4) as usize;
                                if buffer.len() != size {
                                    buffer.resize(size, 0);
                                }

                                if player.render_frame(render_w, render_h, &mut buffer).is_ok() {
                                    let _ = event_tx.send(VideoEvent::FrameReady {
                                        path: current_video_path.clone(),
                                        width: render_w as u32,
                                        height: render_h as u32,
                                        rgba: buffer.clone(),
                                    }).await;
                                }
                            }
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
                            b"time-pos\0".as_ptr() as *const c_char,
                            mpv_format_MPV_FORMAT_DOUBLE,
                            &mut pos as *mut _ as *mut c_void,
                        );
                        mpv_get_property(
                            player.handle,
                            b"duration\0".as_ptr() as *const c_char,
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
