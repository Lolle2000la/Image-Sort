use crate::engine::mpv_context::MpvContext;
use libmpv_sys::*;
use std::ffi::c_char;
use std::os::raw::c_void;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum VideoCommand {
    Load(PathBuf),
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
        path: PathBuf,
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
        path: PathBuf,
        error: String,
    },
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
    let mut pool = [
        std::sync::Arc::new(vec![0u8; max_buffer_size]),
        std::sync::Arc::new(vec![0u8; max_buffer_size]),
        std::sync::Arc::new(vec![0u8; max_buffer_size]),
    ];

    let mut current_video_path = PathBuf::new();
    let mut canonical_video_path: Option<PathBuf> = None;
    let mut cached_video_params: Option<(i32, i32, i64)> = None;
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
                            let current_p = PathBuf::from(current_p_str);
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
