use crate::mpv_context::MpvContext;
use crate::rotation::Rotation;
use libmpv_sys::*;
use std::ffi::c_char;
use std::os::raw::c_void;
use std::path::PathBuf;

/// Configuration for the background playback worker ([`start_video_worker`]).
#[derive(Debug, Clone)]
pub struct PlayerConfig {
    /// Maximum rendered frame width in pixels. Larger videos are scaled down
    /// to fit this box; the render size is the video's aspect-fitted size.
    pub max_frame_width: u32,
    /// Maximum rendered frame height in pixels.
    pub max_frame_height: u32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            max_frame_width: 960,
            max_frame_height: 540,
        }
    }
}

/// Commands sent to the background video worker.
#[derive(Debug, Clone)]
pub enum VideoCommand {
    /// Load a file into the player and start playback.
    Load(PathBuf),
    Play,
    Pause,
    TogglePause,
    /// Relative seek (seconds, may be negative).
    Seek(f64),
    /// Absolute seek (seconds from the start).
    SeekAbsolute(f64),
    SetMute(bool),
    SetVolume(f64),
    Stop,
    /// Pause and release the current file handle so OS-level operations
    /// (rename/move/delete) are not blocked.
    Deactivate,
}

/// Events emitted by the background video worker.
#[derive(Debug, Clone)]
pub enum VideoEvent {
    /// A freshly rendered frame.
    ///
    /// The pixel data is **always unrotated**: mpv renders with
    /// `video-rotate=no`, and the video's rotation is reported separately in
    /// `rotation`. The crate's iced shader applies `rotation` automatically
    /// at draw time; consumers that want the raw pixels simply ignore the
    /// field.
    FrameReady {
        path: PathBuf,
        width: u32,
        height: u32,
        rotation: Rotation,
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
        path: PathBuf,
        error: String,
    },
}

/// Rotates raw RGBA bytes by the given [`Rotation`].
///
/// Returns `(new_width, new_height, new_rgba_bytes)`.
pub fn rotate_rgba(src_w: u32, src_h: u32, src: &[u8], rotation: Rotation) -> (u32, u32, Vec<u8>) {
    use rayon::prelude::*;
    match rotation {
        Rotation::R90 => {
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
        Rotation::R180 => {
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
        Rotation::R270 => {
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
        Rotation::R0 => (src_w, src_h, src.to_vec()),
    }
}

/// Spawns the background video worker on the current tokio runtime with the
/// default [`PlayerConfig`].
///
/// The worker owns an [`MpvContext`], applies incoming [`VideoCommand`]s and
/// emits [`VideoEvent`]s (rendered frames, playback progress, ...) on
/// `event_tx`. The worker exits when the command channel is closed.
pub fn start_video_worker(
    cmd_rx: tokio::sync::mpsc::Receiver<VideoCommand>,
    event_tx: tokio::sync::mpsc::Sender<VideoEvent>,
) {
    start_video_worker_with(cmd_rx, event_tx, PlayerConfig::default());
}

/// Like [`start_video_worker`], with a custom [`PlayerConfig`].
pub fn start_video_worker_with(
    cmd_rx: tokio::sync::mpsc::Receiver<VideoCommand>,
    event_tx: tokio::sync::mpsc::Sender<VideoEvent>,
    config: PlayerConfig,
) {
    tokio::spawn(run_video_worker(cmd_rx, event_tx, config));
}

/// The worker loop. See [`start_video_worker`] for the public entry point.
async fn run_video_worker(
    mut cmd_rx: tokio::sync::mpsc::Receiver<VideoCommand>,
    event_tx: tokio::sync::mpsc::Sender<VideoEvent>,
    config: PlayerConfig,
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

    // Frame buffers are pooled and reused while iced still holds them: a pool
    // slot is reusable only when the worker is its sole owner (strong_count ==
    // 1). The pool payload stays `Arc<Vec<u8>>` rather than `Arc<[u8]>`
    // precisely because the pool must resize in place via `Arc::get_mut`; an
    // immutable slice payload would force a fresh allocation per frame.
    let max_buffer_size = (config.max_frame_width * config.max_frame_height * 4) as usize;
    let mut pool = [
        std::sync::Arc::new(vec![0u8; max_buffer_size]),
        std::sync::Arc::new(vec![0u8; max_buffer_size]),
        std::sync::Arc::new(vec![0u8; max_buffer_size]),
    ];

    let mut current_video_path = PathBuf::new();
    let mut canonical_video_path: Option<PathBuf> = None;
    let mut cached_video_params: Option<(i32, i32, Rotation)> = None;
    let mut last_position = -1.0;
    let mut last_muted = false;
    let mut last_volume = -1.0;
    let mut last_paused = false;
    let mut is_active = false;

    let mut progress_interval = tokio::time::interval(std::time::Duration::from_millis(100));

    loop {
        if event_tx.is_closed() && is_active {
            player.stop();
            is_active = false;
        }

        tokio::select! {
            cmd_opt = cmd_rx.recv() => {
                let Some(mut cmd) = cmd_opt else {
                    player.stop();
                    break;
                };
                while let Ok(newer_cmd) = cmd_rx.try_recv() {
                    cmd = newer_cmd;
                }
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
                                        error: err.to_string(),
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
                            let current_p = PathBuf::from(current_p_str);
                            let paths_match = current_p == current_video_path
                                || canonical_video_path.as_ref().is_some_and(|cp| current_p == *cp || current_p.canonicalize().ok().as_ref() == Some(cp));

                            if paths_match && player.is_video_ready() {
                                let (w, h) = player.get_video_size();
                                if w > 0 && h > 0 {
                                    let rotation = player.get_video_rotation();
                                    let (eff_w, eff_h) = if rotation.is_swapped() {
                                        (h, w)
                                    } else {
                                        (w, h)
                                    };

                                    let scale = (config.max_frame_width as f64 / eff_w as f64)
                                        .min(config.max_frame_height as f64 / eff_h as f64)
                                        .min(1.0);
                                    let render_unrot_w = ((w as f64 * scale) as i32) & !1;
                                    let render_unrot_h = ((h as f64 * scale) as i32) & !1;

                                    if render_unrot_w > 0 && render_unrot_h > 0 {
                                        cached_video_params = Some((render_unrot_w, render_unrot_h, rotation));
                                    }
                                }
                            }
                        }

                        // Rotation is detected once per load above: a file's
                        // rotation cannot change during playback, and the
                        // per-second recheck used to re-parse the container
                        // (read_mp4_tkhd_rotation) for every second of video,
                        // turning crafted files into per-second CPU burn.

                        if let Some((render_unrot_w, render_unrot_h, rotation)) = cached_video_params {
                            let unrot_size = (render_unrot_w * render_unrot_h * 4) as usize;

                            // Find a free buffer in the pool (where we are the sole owner)
                            let free_buffer = pool
                                .iter_mut()
                                .find(|buf| std::sync::Arc::strong_count(buf) == 1);

                            if let Some(arc_buf) = free_buffer
                                && let Some(target_vec) = std::sync::Arc::get_mut(arc_buf)
                            {
                                target_vec.resize(unrot_size, 0);
                                if player.render_frame(render_unrot_w, render_unrot_h, target_vec).is_ok() {
                                    let _ = event_tx.try_send(VideoEvent::FrameReady {
                                        path: current_video_path.clone(),
                                        width: render_unrot_w as u32,
                                        height: render_unrot_h as u32,
                                        rotation,
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
                    unsafe {
                        mpv_get_property(
                            player.handle,
                            c"time-pos".as_ptr(),
                            mpv_format_MPV_FORMAT_DOUBLE,
                            &mut pos as *mut _ as *mut c_void,
                        );
                    }
                    let dur = player.get_duration();
                    if pos != last_position {
                        // Stateful/periodic events are dropped under
                        // backpressure (like FrameReady): the worker must
                        // never stall command handling and frame pumping on a
                        // slow consumer, and the next tick resends the value.
                        let _ = event_tx.try_send(VideoEvent::PlaybackProgress {
                            position: pos,
                            duration: dur,
                        });
                        last_position = pos;
                    }

                    let mute = player.get_mute();
                    if mute != last_muted {
                        let _ = event_tx.try_send(VideoEvent::Muted(mute));
                        last_muted = mute;
                    }
                    let vol = player.get_volume();
                    if vol != last_volume {
                        let _ = event_tx.try_send(VideoEvent::Volume(vol));
                        last_volume = vol;
                    }
                    let paused = !player.is_playing();
                    if paused != last_paused {
                        let _ = event_tx.try_send(VideoEvent::Paused(paused));
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
        let (w, h, dst) = rotate_rgba(2, 1, &src, Rotation::R0);
        assert_eq!((w, h), (2, 1));
        assert_eq!(dst, src);
    }

    #[test]
    fn test_rotate_rgba_90() {
        let src = vec![255, 0, 0, 255, 0, 255, 0, 255]; // 2x1: Red, Green
        let (w, h, dst) = rotate_rgba(2, 1, &src, Rotation::R90);
        assert_eq!((w, h), (1, 2));
        // 90 deg CW: (0,0) Red -> (0,0); (1,0) Green -> (0,1)
        assert_eq!(&dst[0..4], &[255, 0, 0, 255]);
        assert_eq!(&dst[4..8], &[0, 255, 0, 255]);
    }

    #[test]
    fn test_rotate_rgba_180() {
        let src = vec![255, 0, 0, 255, 0, 255, 0, 255]; // 2x1: Red, Green
        let (w, h, dst) = rotate_rgba(2, 1, &src, Rotation::R180);
        assert_eq!((w, h), (2, 1));
        // 180 deg: Green, Red
        assert_eq!(&dst[0..4], &[0, 255, 0, 255]);
        assert_eq!(&dst[4..8], &[255, 0, 0, 255]);
    }

    #[test]
    fn test_rotate_rgba_270() {
        let src = vec![255, 0, 0, 255, 0, 255, 0, 255]; // 2x1: Red, Green
        let (w, h, dst) = rotate_rgba(2, 1, &src, Rotation::R270);
        assert_eq!((w, h), (1, 2));
        // 270 deg CW: (0,0) Red -> (0,1); (1,0) Green -> (0,0)
        assert_eq!(&dst[0..4], &[0, 255, 0, 255]);
        assert_eq!(&dst[4..8], &[255, 0, 0, 255]);
    }
}
