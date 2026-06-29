use iced::widget::{container, text};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;

pub fn media_preview_view(state: &AppState) -> Element<'_, Message> {
    let Some(index) = state.selected_index else {
        return container(text(state.l10n.tr("ui-select-file")).size(14))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    };

    let filtered = state.filtered_media_entries();
    let Some(entry) = filtered.get(index) else {
        return container(text(state.l10n.tr("ui-file-not-found")).size(14))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    };

    let preview_element: Element<'_, Message> = match entry.media_type {
        media_sort_core::media_type::MediaType::Image => {
            if let Some((ref path, ref handle)) = state.selected_image {
                if path == &entry.path {
                    iced::widget::image(handle.clone())
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                } else {
                    container(text(state.l10n.tr("ui-loading-image")).size(14))
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                }
            } else {
                container(text(state.l10n.tr("ui-loading-image")).size(14))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
        media_sort_core::media_type::MediaType::Video => {
            use iced::widget::{button, column, row, slider, text};

            let video_content: Element<'_, Message> = if let Some(ref handle) = state.video_frame {
                iced::widget::image(handle.clone())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                crate::widgets::video_canvas::video_canvas_view(entry.path.clone(), &state.l10n)
            };

            let play_pause_btn = button(
                text(char::from(if state.video_paused {
                    lucide_icons::Icon::Play
                } else {
                    lucide_icons::Icon::Pause
                }))
                .font(iced::Font::with_name("lucide"))
                .size(16)
            )
            .padding(8)
            .on_press(Message::VideoPlayPause);

            let stop_btn = button(
                text(char::from(lucide_icons::Icon::Square))
                    .font(iced::Font::with_name("lucide"))
                    .size(16)
            )
            .padding(8)
            .on_press(Message::VideoStop);

            let format_time = |secs: f64| {
                if secs.is_nan() || secs.is_infinite() || secs < 0.0 {
                    return "00:00".to_string();
                }
                let total_secs = secs.round() as i32;
                let minutes = total_secs / 60;
                let seconds = total_secs % 60;
                format!("{:02}:{:02}", minutes, seconds)
            };

            let time_str = format!(
                "{} / {}",
                format_time(state.video_position),
                format_time(state.video_duration)
            );
            let time_label = text(time_str).size(13);

            let seek_val = state.video_position;
            let seek_max = if state.video_duration > 0.0 { state.video_duration } else { 1.0 };
            let seekbar = slider(
                0.0..=seek_max,
                seek_val,
                Message::VideoSeek
            )
            .width(Length::Fill);

            let mute_btn = button(
                text(char::from(if state.video_muted {
                    lucide_icons::Icon::VolumeX
                } else {
                    lucide_icons::Icon::Volume2
                }))
                .font(iced::Font::with_name("lucide"))
                .size(16)
            )
            .padding(8)
            .on_press(Message::VideoMute);

            let volume_slider = slider(
                0.0..=100.0,
                state.video_volume,
                Message::VideoVolume
            )
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
            .align_y(iced::Alignment::Center)
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
        media_sort_core::media_type::MediaType::Audio => {
            crate::widgets::video_canvas::audio_controls_view()
        }
    };

    container(preview_element)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|theme: &iced::Theme| {
            let palette = theme.palette();
            let border_color = Color { a: 0.2, ..palette.text };
            iced::widget::container::Style {
                background: Some(iced::Background::Color(palette.background)),
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: border_color,
                },
                ..iced::widget::container::Style::default()
            }
        })
        .into()
}
