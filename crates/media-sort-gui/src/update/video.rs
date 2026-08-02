use iced::Task;

use crate::message::{Message, VideoMessage};
use crate::state::AppState;

pub fn handle_video_message(state: &mut AppState, msg: VideoMessage) -> Task<Message> {
    match msg {
        VideoMessage::Player(player_msg) => {
            if let iced_mpv::PlayerMessage::Event(iced_mpv::VideoEvent::FrameReady {
                path, ..
            }) = &player_msg
            {
                state.cache.media_errors.remove(path);
            }
            if let Some(event) = state.video.update(player_msg) {
                match event {
                    iced_mpv::VideoPlayerEvent::LoadFailed { path, error } => {
                        tracing::error!("Video load failed for {}: {}", path.display(), error);
                        state.cache.media_errors.record(path, error);
                    }
                    iced_mpv::VideoPlayerEvent::PlayExternally(path) => {
                        super::tasks::open_externally(&path);
                    }
                }
            }
            Task::none()
        }
        VideoMessage::Seek(pos) => {
            state.video.seek(pos);
            Task::none()
        }
        VideoMessage::Volume(vol) => {
            state.video.set_volume(vol);
            Task::none()
        }
        VideoMessage::Mute => {
            state.video.toggle_mute();
            Task::none()
        }
        VideoMessage::PlayPause => {
            state.video.toggle_pause();
            Task::none()
        }
        VideoMessage::Stop => {
            state.video.stop();
            Task::none()
        }
        VideoMessage::PlayExternally(path) => {
            super::tasks::open_externally(&path);
            Task::none()
        }
    }
}
