use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Font, Length};

use crate::message::Message;
use crate::state::AppState;

mod contributors_codegen {
    include!(concat!(env!("OUT_DIR"), "/contributors_codegen.rs"));
}

pub use contributors_codegen::CONTRIBUTORS;

const BOLD_FONT: Font = Font {
    weight: iced::font::Weight::Bold,
    ..Font::DEFAULT
};

pub fn credits_dialog_view(state: &AppState) -> Element<'_, Message> {
    let title = text(state.l10n.tr("ui-about-title"))
        .size(20)
        .font(BOLD_FONT);

    let version = env!("CARGO_PKG_VERSION");

    let mut contributor_widgets = Vec::new();
    for (i, c) in CONTRIBUTORS.iter().enumerate() {
        if i > 0 {
            contributor_widgets.push(text(", ").size(12).into());
        }
        contributor_widgets.push(
            button(text(c.preferred_display_name).size(12))
                .on_press(Message::OpenUrl(c.link.to_string()))
                .style(iced::widget::button::text)
                .into(),
        );
    }
    let contributors_row = row(contributor_widgets)
        .spacing(2)
        .align_y(Alignment::Center);

    let about_info = column![
        row![
            text(state.l10n.tr("ui-about-version"))
                .size(12)
                .width(Length::Fixed(100.0))
                .font(BOLD_FONT),
            text(version).size(12),
        ]
        .spacing(8),
        row![
            text(state.l10n.tr("ui-about-license"))
                .size(12)
                .width(Length::Fixed(100.0))
                .font(BOLD_FONT),
            text("MIT License").size(12),
        ]
        .spacing(8),
        row![
            text(state.l10n.tr("ui-about-maintainer"))
                .size(12)
                .width(Length::Fixed(100.0))
                .font(BOLD_FONT),
            text("Luca Auer").size(12),
        ]
        .spacing(8),
        row![
            text(state.l10n.tr("ui-about-contributors"))
                .size(12)
                .width(Length::Fixed(100.0))
                .font(BOLD_FONT),
            contributors_row,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(6);

    let website_btn = button(text(state.l10n.tr("ui-about-website")).size(12))
        .on_press(Message::OpenUrl("https://mediasort.app".to_string()))
        .style(iced::widget::button::text);

    let repo_btn = button(text(state.l10n.tr("ui-about-repository")).size(12))
        .on_press(Message::OpenUrl(
            "https://github.com/Lolle2000la/Image-Sort".to_string(),
        ))
        .style(iced::widget::button::text);

    let links_row = row![website_btn, text(" | ").size(12), repo_btn]
        .spacing(4)
        .align_y(Alignment::Center);

    let divider = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
        .style(|theme: &iced::Theme| {
            let palette = theme.palette();
            iced::widget::container::Style {
                background: Some(iced::Background::Color(Color {
                    a: 0.15,
                    ..palette.text
                })),
                ..Default::default()
            }
        })
        .width(Length::Fill);

    let libraries_title = text(state.l10n.tr("ui-about-libraries"))
        .size(14)
        .font(BOLD_FONT);
    let leading_text = text(state.l10n.tr("ui-credits-leading")).size(12);

    let libraries = vec![
        ("dirs", "https://github.com/dirs-dev/dirs-rs"),
        ("Fluent", "https://github.com/projectfluent/fluent-rs"),
        ("Iced", "https://github.com/iced-rs/iced"),
        ("id3", "https://github.com/polyfloyd/rust-id3"),
        ("Image", "https://github.com/image-rs/image"),
        ("Kamadak EXIF", "https://github.com/kamadak/exif-rs"),
        ("libmpv", "https://github.com/mpv-player/mpv"),
        ("lru", "https://github.com/jeromefroe/lru-rs"),
        ("metaflac", "https://github.com/jameshurst/rust-metaflac"),
        ("mp4ameta", "https://github.com/Saecki/rust-mp4ameta"),
        ("Notify", "https://github.com/notify-rs/notify"),
        ("once_cell", "https://github.com/matklad/once_cell"),
        (
            "RFD (Rust File Dialogs)",
            "https://github.com/PolyMeilex/rfd",
        ),
        ("Rodio", "https://github.com/RustAudio/rodio"),
        ("Serde", "https://github.com/serde-rs/serde"),
        ("serde_json", "https://github.com/serde-rs/json"),
        ("strum", "https://github.com/Peternator7/strum"),
        ("Symphonia", "https://github.com/pdeljanov/Symphonia"),
        ("thiserror", "https://github.com/dtolnay/thiserror"),
        ("Tokio", "https://github.com/tokio-rs/tokio"),
        ("tracing", "https://github.com/tokio-rs/tracing"),
        ("tracing-subscriber", "https://github.com/tokio-rs/tracing"),
        ("Trash", "https://github.com/Byron/trash-rs"),
        ("unic-langid", "https://github.com/zbraniecki/unic-locale"),
        ("walkdir", "https://github.com/BurntSushi/walkdir"),
    ];

    let mut rows = Vec::with_capacity(libraries.len());
    for (name, url) in libraries {
        rows.push(
            column![
                text(name).size(12).font(BOLD_FONT).width(Length::Fill),
                button(text(url).size(10))
                    .on_press(Message::OpenUrl(url.to_string()))
                    .style(iced::widget::button::text),
            ]
            .spacing(2)
            .width(Length::Fill)
            .into(),
        );
    }

    let list = column(rows).spacing(10).width(Length::Fill);

    let close_btn = button(text(state.l10n.tr("ui-close")))
        .on_press(Message::CloseCredits)
        .style(iced::widget::button::primary);

    container(
        column![
            title,
            about_info,
            links_row,
            divider,
            libraries_title,
            leading_text,
            scrollable(list).width(Length::Fill).height(Length::Fill),
            close_btn,
        ]
        .spacing(12)
        .align_x(Alignment::Start),
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
    .width(Length::Fixed(500.0))
    .height(Length::Fixed(550.0))
    .into()
}
