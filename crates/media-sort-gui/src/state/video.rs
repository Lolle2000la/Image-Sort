use std::time::Instant;

#[derive(Debug, Clone)]
pub struct VideoPlaybackState {
    pub sender:
        Option<tokio::sync::mpsc::Sender<media_sort_backend::media::mpv_context::VideoCommand>>,
    pub frame: Option<iced::widget::image::Handle>,
    pub rgba: Option<std::sync::Arc<Vec<u8>>>,
    pub width: u32,
    pub height: u32,
    pub position: f64,
    pub duration: f64,
    pub volume: f64,
    pub muted: bool,
    pub paused: bool,
    pub ready: bool,
    pub seek_position: Option<f64>,
    pub last_seek_time: Option<Instant>,
}

impl Default for VideoPlaybackState {
    fn default() -> Self {
        Self {
            sender: None,
            frame: None,
            rgba: None,
            width: 0,
            height: 0,
            position: 0.0,
            duration: 0.0,
            volume: 100.0,
            muted: false,
            paused: false,
            ready: false,
            seek_position: None,
            last_seek_time: None,
        }
    }
}
