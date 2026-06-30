use iced::widget::{button, column, container, scrollable, text};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;

pub fn credits_dialog_view(state: &AppState) -> Element<'_, Message> {
    let title = text(state.l10n.tr("ui-credits-title")).size(20);
    let leading_text = text(state.l10n.tr("ui-credits-leading")).size(13);

    let libraries = vec![
        ("Iced", "https://github.com/iced-rs/iced"),
        ("Tokio", "https://github.com/tokio-rs/tokio"),
        ("Symphonia", "https://github.com/pdeljanov/Symphonia"),
        ("libmpv", "https://github.com/mpv-player/mpv"),
        ("Kamadak EXIF", "https://github.com/kamadak/exif-rs"),
        ("Fluent", "https://github.com/projectfluent/fluent-rs"),
        ("Image", "https://github.com/image-rs/image"),
        ("Trash", "https://github.com/Byron/trash-rs"),
        ("Notify", "https://github.com/notify-rs/notify"),
        ("RFD (Rust File Dialogs)", "https://github.com/PolyhedralDev/rfd"),
        ("Rodio", "https://github.com/RustAudio/rodio"),
        ("Serde", "https://github.com/serde-rs/serde"),
        ("thiserror", "https://github.com/dtolnay/thiserror"),
        ("tracing", "https://github.com/tokio-rs/tracing"),
        ("wgpu", "https://github.com/gfx-rs/wgpu"),
        ("winit", "https://github.com/rust-windowing/winit"),
        ("parking_lot", "https://github.com/Amanieu/parking_lot"),
        ("crossbeam", "https://github.com/crossbeam-rs/crossbeam"),
        ("walkdir", "https://github.com/BurntSushi/walkdir"),
        ("id3", "https://github.com/polyfloyd/rust-id3"),
        ("metaflac", "https://github.com/tazziden/metaflac-rs"),
        ("mp4ameta", "https://github.com/sgr4/mp4ameta"),
        ("ash", "https://github.com/ash-rs/ash"),
        ("lru", "https://github.com/jeromefroe/lru-rs"),
        ("log", "https://github.com/rust-lang/log"),
        ("dirs", "https://github.com/dirs-dev/dirs-rs"),
        ("strum", "https://github.com/Peternator7/strum"),
    ];

    let mut rows = Vec::with_capacity(libraries.len());
    for (name, url) in libraries {
        rows.push(
            column![text(name).size(13), text(url).size(11),]
                .spacing(2)
                .into(),
        );
    }

    let list = column(rows).spacing(10);

    let close_btn = button(text(state.l10n.tr("ui-close")))
        .on_press(Message::CloseCredits)
        .style(iced::widget::button::primary);

    container(
        column![
            title,
            leading_text,
            scrollable(list).height(Length::Fill),
            close_btn,
        ]
        .spacing(16)
        .align_x(iced::Alignment::Start),
    )
    .padding(24)
    .style(|theme: &iced::Theme| {
        let palette = theme.palette();
        let border_color = Color {
            a: 0.2,
            ..palette.text
        };
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
    .width(Length::Fixed(400.0))
    .height(Length::Fixed(450.0))
    .into()
}
