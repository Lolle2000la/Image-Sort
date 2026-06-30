use iced::widget::{button, column, container, scrollable, text};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;

pub fn credits_dialog_view(state: &AppState) -> Element<'_, Message> {
    let title = text(state.l10n.tr("ui-credits-title")).size(20);
    let leading_text = text(state.l10n.tr("ui-credits-leading")).size(13);

    let libraries = vec![
        ("ash", "https://github.com/ash-rs/ash"),
        ("crossbeam", "https://github.com/crossbeam-rs/crossbeam"),
        ("dirs", "https://github.com/dirs-dev/dirs-rs"),
        ("env_logger", "https://github.com/rust-cli/env_logger"),
        ("Fluent", "https://github.com/projectfluent/fluent-rs"),
        ("Iced", "https://github.com/iced-rs/iced"),
        ("id3", "https://github.com/polyfloyd/rust-id3"),
        ("Image", "https://github.com/image-rs/image"),
        ("Kamadak EXIF", "https://github.com/kamadak/exif-rs"),
        ("libmpv", "https://github.com/mpv-player/mpv"),
        ("log", "https://github.com/rust-lang/log"),
        ("lru", "https://github.com/jeromefroe/lru-rs"),
        ("metaflac", "https://github.com/jameshurst/rust-metaflac"),
        ("mp4ameta", "https://github.com/Saecki/rust-mp4ameta"),
        ("Notify", "https://github.com/notify-rs/notify"),
        ("once_cell", "https://github.com/matklad/once_cell"),
        ("parking_lot", "https://github.com/Amanieu/parking_lot"),
        (
            "raw-window-handle",
            "https://github.com/rust-windowing/raw-window-handle",
        ),
        ("RFD (Rust File Dialogs)", "https://github.com/PolyhedralDev/rfd"),
        ("Rodio", "https://github.com/RustAudio/rodio"),
        ("Serde", "https://github.com/serde-rs/serde"),
        ("serde_json", "https://github.com/serde-rs/json"),
        ("smol_str", "https://github.com/rust-analyzer/smol_str"),
        ("strum", "https://github.com/Peternator7/strum"),
        ("Symphonia", "https://github.com/pdeljanov/Symphonia"),
        ("thiserror", "https://github.com/dtolnay/thiserror"),
        ("Tokio", "https://github.com/tokio-rs/tokio"),
        ("tracing", "https://github.com/tokio-rs/tracing"),
        ("Trash", "https://github.com/Byron/trash-rs"),
        ("unic-langid", "https://github.com/unicode-org/unic-locale"),
        ("walkdir", "https://github.com/BurntSushi/walkdir"),
        ("wgpu", "https://github.com/gfx-rs/wgpu"),
        ("winit", "https://github.com/rust-windowing/winit"),
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
