use std::fmt;

use media_sort_backend::media::audio_decoder::AudioPlayer;

pub struct AudioPlaybackState {
    pub player: Option<AudioPlayer>,
    pub playing: bool,
    pub position: f64,
    pub duration: f64,
    pub volume: f64,
    pub muted: bool,
    pub selected_cover: Option<iced::widget::image::Handle>,
}

// Manual Debug impl because AudioPlayer doesn't implement Debug.
impl fmt::Debug for AudioPlaybackState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioPlaybackState")
            .field("playing", &self.playing)
            .field("position", &self.position)
            .field("duration", &self.duration)
            .field("volume", &self.volume)
            .field("muted", &self.muted)
            .finish()
    }
}

impl AudioPlaybackState {
    pub fn new() -> Self {
        Self {
            player: AudioPlayer::new().ok(),
            playing: false,
            position: 0.0,
            duration: 0.0,
            volume: 100.0,
            muted: false,
            selected_cover: None,
        }
    }
}

impl Default for AudioPlaybackState {
    fn default() -> Self {
        Self::new()
    }
}
