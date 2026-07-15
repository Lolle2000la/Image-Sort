use iced::widget::{button, row, slider, text};
use iced::{Alignment, Element, Font, Length};

#[derive(Debug, Clone)]
pub enum MediaControlMessage {
    PlayPause,
    Stop,
    Seek(f64),
    SetVolume(f64),
    ToggleMute,
}

pub fn media_controls_view(
    position: f64,
    duration: f64,
    volume: f64,
    muted: bool,
    playing: bool,
) -> Element<'static, MediaControlMessage> {
    let play_pause_btn = button(
        text(char::from(if playing {
            lucide_icons::Icon::Pause
        } else {
            lucide_icons::Icon::Play
        }))
        .font(Font::with_name("lucide"))
        .size(16),
    )
    .padding(8)
    .on_press(MediaControlMessage::PlayPause);

    let stop_btn = button(
        text(char::from(lucide_icons::Icon::Square))
            .font(Font::with_name("lucide"))
            .size(16),
    )
    .padding(8)
    .on_press(MediaControlMessage::Stop);

    let time_str = format!("{} / {}", format_time(position), format_time(duration));
    let time_label = text(time_str).size(13);

    let seek_max = if duration > 0.0 { duration } else { 1.0 };
    let seekbar = slider(0.0..=seek_max, position, MediaControlMessage::Seek).width(Length::Fill);

    let mute_btn = button(
        text(char::from(if muted {
            lucide_icons::Icon::VolumeX
        } else {
            lucide_icons::Icon::Volume2
        }))
        .font(Font::with_name("lucide"))
        .size(16),
    )
    .padding(8)
    .on_press(MediaControlMessage::ToggleMute);

    let volume_slider =
        slider(0.0..=100.0, volume, MediaControlMessage::SetVolume).width(Length::Fixed(80.0));

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

fn format_time(secs: f64) -> String {
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
    }

    #[test]
    fn test_format_time_with_seconds() {
        assert_eq!(format_time(90.0), "01:30");
        assert_eq!(format_time(3661.0), "61:01");
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
