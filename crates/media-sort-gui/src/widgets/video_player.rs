use std::path::PathBuf;

use iced::widget::{button, column, container, row, slider, text};
use iced::{Alignment, Background, Border, Color, Element, Font, Length};

use crate::message::{Message, VideoMessage};
use crate::state::AppState;

pub fn video_player<'a>(
    path: PathBuf,
    state: &AppState,
    thumb_handle: Option<iced::widget::image::Handle>,
) -> Element<'a, Message> {
    let video_content: Element<'_, Message> = if state.video_rgba.is_some() {
        crate::widgets::video_shader::video_shader_view(
            state.video_width,
            state.video_height,
            state.video_rgba.clone(),
        )
    } else if let Some(handle) = thumb_handle {
        iced::widget::image(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        placeholder(path, &state.l10n)
    };

    let play_pause_btn = button(
        text(char::from(if state.video_paused {
            lucide_icons::Icon::Play
        } else {
            lucide_icons::Icon::Pause
        }))
        .font(Font::with_name("lucide"))
        .size(16),
    )
    .padding(8)
    .on_press(Message::Video(VideoMessage::PlayPause));

    let stop_btn = button(
        text(char::from(lucide_icons::Icon::Square))
            .font(Font::with_name("lucide"))
            .size(16),
    )
    .padding(8)
    .on_press(Message::Video(VideoMessage::Stop));

    let time_str = format!(
        "{} / {}",
        format_time(state.video_position),
        format_time(state.video_duration)
    );
    let time_label = text(time_str).size(13);

    let seek_max = if state.video_duration > 0.0 {
        state.video_duration
    } else {
        1.0
    };
    let seekbar = slider(0.0..=seek_max, state.video_position, |v| {
        Message::Video(VideoMessage::Seek(v))
    })
    .width(Length::Fill);

    let mute_btn = button(
        text(char::from(if state.video_muted {
            lucide_icons::Icon::VolumeX
        } else {
            lucide_icons::Icon::Volume2
        }))
        .font(Font::with_name("lucide"))
        .size(16),
    )
    .padding(8)
    .on_press(Message::Video(VideoMessage::Mute));

    let volume_slider = slider(0.0..=100.0, state.video_volume, |v| {
        Message::Video(VideoMessage::Volume(v))
    })
    .width(Length::Fixed(80.0));

    let controls_row = row![
        play_pause_btn,
        stop_btn,
        time_label,
        seekbar,
        mute_btn,
        volume_slider,
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding(8);

    column![
        container(video_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        controls_row
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn placeholder(
    path: PathBuf,
    l10n: &media_sort_core::l10n::Localization,
) -> Element<'static, Message> {
    container(
        column![
            text(l10n.tr("ui-video-playback-soon")).size(16),
            text(l10n.tr("ui-rendering-not-implemented")).size(12),
            button(
                row![
                    text(char::from(lucide_icons::Icon::ExternalLink))
                        .font(Font::with_name("lucide"))
                        .size(12),
                    text(format!(" {}", l10n.tr("ui-play-in-system-player")))
                ]
                .align_y(Alignment::Center)
            )
            .padding([8, 16])
            .on_press(Message::Video(VideoMessage::PlayExternally(path))),
        ]
        .spacing(12)
        .align_x(Alignment::Center),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.05, 0.05, 0.08))),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.2, 0.2, 0.25),
        },
        ..container::Style::default()
    })
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
