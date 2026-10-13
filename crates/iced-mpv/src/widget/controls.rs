use crate::state::VideoState;
use iced::widget::{button, row, slider, text};
use iced::{Alignment, Element, Font, Length};

/// User actions produced by the transport-agnostic control bar
/// ([`media_controls_view`]). A subset of [`crate::VideoAction`] (no
/// play-externally), so the same bar can drive audio playback too.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaControl {
    /// Toggle between playing and paused.
    PlayPause,
    /// Stop playback.
    Stop,
    /// Seek to the given position in seconds.
    Seek(f64),
    /// Set the volume in percent (0–100).
    SetVolume(f64),
    /// Toggle mute.
    ToggleMute,
}

/// An immutable snapshot of transport state for [`media_controls_view`].
///
/// The bar is transport-agnostic: build this from [`VideoState`] (via
/// `From<&VideoState>` or field-by-field) or from your own audio player state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MediaControlsState {
    /// The playback position in seconds.
    pub position: f64,
    /// The media duration in seconds (0 until known).
    pub duration: f64,
    /// The volume in percent (0–100).
    pub volume: f64,
    /// Whether the output is muted.
    pub muted: bool,
    /// Whether playback is currently playing (not paused).
    pub playing: bool,
}

impl From<&VideoState> for MediaControlsState {
    fn from(state: &VideoState) -> Self {
        Self {
            position: state.position(),
            duration: state.duration(),
            volume: state.volume(),
            muted: state.muted(),
            playing: !state.paused(),
        }
    }
}

/// A transport-agnostic media control bar (play/pause, stop, seekbar, mute
/// and volume) that emits [`MediaControl`] messages. It only reads a
/// [`MediaControlsState`] snapshot, so it can render video playback state *or*
/// audio player state.
pub fn media_controls_view(state: MediaControlsState) -> Element<'static, MediaControl> {
    let play_pause_btn = button(
        text(char::from(if state.playing {
            lucide_icons::Icon::Pause
        } else {
            lucide_icons::Icon::Play
        }))
        .font(Font::with_name("lucide"))
        .size(16),
    )
    .padding(8)
    .on_press(MediaControl::PlayPause);

    let stop_btn = button(
        text(char::from(lucide_icons::Icon::Square))
            .font(Font::with_name("lucide"))
            .size(16),
    )
    .padding(8)
    .on_press(MediaControl::Stop);

    let time_str = format!(
        "{} / {}",
        format_time(state.position),
        format_time(state.duration)
    );
    let time_label = text(time_str).size(13);

    let seek_max = if state.duration > 0.0 {
        state.duration
    } else {
        1.0
    };
    let seekbar = slider(0.0..=seek_max, state.position, MediaControl::Seek).width(Length::Fill);

    let mute_btn = button(
        text(char::from(if state.muted {
            lucide_icons::Icon::VolumeX
        } else {
            lucide_icons::Icon::Volume2
        }))
        .font(Font::with_name("lucide"))
        .size(16),
    )
    .padding(8)
    .on_press(MediaControl::ToggleMute);

    let volume_slider =
        slider(0.0..=100.0, state.volume, MediaControl::SetVolume).width(Length::Fixed(80.0));

    row![
        play_pause_btn,
        stop_btn,
        time_label,
        seekbar,
        mute_btn,
        volume_slider,
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding(8)
    .into()
}

/// Formats seconds as `MM:SS`. Non-finite and negative values render as
/// `00:00`.
pub fn format_time(secs: f64) -> String {
    if secs.is_nan() || secs.is_infinite() || secs < 0.0 {
        return "00:00".to_string();
    }
    let total_secs = secs.round() as i32;
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time_zero() {
        assert_eq!(format_time(0.0), "00:00");
    }

    #[test]
    fn test_format_time_whole_minutes() {
        assert_eq!(format_time(60.0), "01:00");
        assert_eq!(format_time(120.0), "02:00");
        assert_eq!(format_time(59.0), "00:59");
        assert_eq!(format_time(119.0), "01:59");
    }

    #[test]
    fn test_format_time_with_seconds() {
        assert_eq!(format_time(90.0), "01:30");
        assert_eq!(format_time(3661.0), "61:01");
        assert_eq!(format_time(61.0), "01:01");
        assert_eq!(format_time(59.0), "00:59");
        assert_eq!(format_time(119.0), "01:59");
    }

    #[test]
    fn test_format_time_nan_and_infinite() {
        assert_eq!(format_time(f64::NAN), "00:00");
        assert_eq!(format_time(f64::INFINITY), "00:00");
        assert_eq!(format_time(f64::NEG_INFINITY), "00:00");
    }

    #[test]
    fn test_format_time_negative() {
        assert_eq!(format_time(-1.0), "00:00");
        assert_eq!(format_time(-100.0), "00:00");
    }
}
