use iced::Task;

use crate::message::{Message, VideoMessage};
use crate::state::AppState;

pub fn handle_video_message(state: &mut AppState, msg: VideoMessage) -> Task<Message> {
    match msg {
        VideoMessage::PlayerReady(sender) => {
            state.video_sender = Some(sender);
            Task::none()
        }
        VideoMessage::Event(event) => {
            match event {
                media_sort_backend::media::mpv_context::VideoEvent::FrameReady {
                    path,
                    width,
                    height,
                    rgba,
                } => {
                    let current_path = state.selected_index.and_then(|idx| {
                        state
                            .filtered_media_entries()
                            .get(idx)
                            .map(|e| e.path.clone())
                    });
                    if Some(path) == current_path && state.video_ready {
                        state.video_rgba = Some(rgba);
                        state.video_width = width;
                        state.video_height = height;
                        state.video_frame =
                            Some(iced::widget::image::Handle::from_rgba(1, 1, vec![0]));
                    }
                }
                media_sort_backend::media::mpv_context::VideoEvent::PlaybackProgress {
                    position,
                    duration,
                } => {
                    state.video_position = position;
                    state.video_duration = duration;
                    state.video_ready = true;
                }
                media_sort_backend::media::mpv_context::VideoEvent::Muted(muted) => {
                    state.video_muted = muted;
                }
                media_sort_backend::media::mpv_context::VideoEvent::Volume(vol) => {
                    state.video_volume = vol;
                }
                media_sort_backend::media::mpv_context::VideoEvent::Paused(paused) => {
                    state.video_paused = paused;
                }
            }
            Task::none()
        }
        VideoMessage::Seek(pos) => {
            state.video_seek_position = Some(pos);
            let should_seek = state
                .video_last_seek_time
                .is_none_or(|t| t.elapsed() >= std::time::Duration::from_millis(333));
            if should_seek {
                if let Some(ref sender) = state.video_sender {
                    let _ = sender.try_send(
                        media_sort_backend::media::mpv_context::VideoCommand::SeekAbsolute(pos),
                    );
                }
                state.video_last_seek_time = Some(std::time::Instant::now());
            }
            Task::none()
        }
        VideoMessage::Volume(vol) => {
            if let Some(ref sender) = state.video_sender {
                let _ = sender
                    .try_send(media_sort_backend::media::mpv_context::VideoCommand::SetVolume(vol));
            }
            Task::none()
        }
        VideoMessage::Mute => {
            if let Some(ref sender) = state.video_sender {
                let _ = sender.try_send(
                    media_sort_backend::media::mpv_context::VideoCommand::SetMute(
                        !state.video_muted,
                    ),
                );
            }
            Task::none()
        }
        VideoMessage::PlayPause => {
            if let Some(ref sender) = state.video_sender {
                let _ = sender
                    .try_send(media_sort_backend::media::mpv_context::VideoCommand::TogglePause);
            }
            Task::none()
        }
        VideoMessage::Stop => {
            if let Some(ref sender) = state.video_sender {
                let _ = sender.try_send(media_sort_backend::media::mpv_context::VideoCommand::Stop);
            }
            Task::none()
        }
        VideoMessage::PlayExternally(path) => {
            super::tasks::open_externally(&path);
            Task::none()
        }
    }
}
