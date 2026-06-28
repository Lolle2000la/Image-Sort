use iced::widget::{button, column, container, scrollable, text};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;

pub fn credits_dialog_view(state: &AppState) -> Element<'_, Message> {
    let title = text(state.l10n.tr("ui-credits-title")).size(20);
    let leading_text = text(state.l10n.tr("ui-credits-leading")).size(13);

    let libraries = vec![
        ("AdonisUI", "https://github.com/benruehl/adonis-ui"),
        ("AdonisUI.ClassicTheme", "https://github.com/benruehl/adonis-ui"),
        ("coverlet.collector", "https://github.com/coverlet-coverage/coverlet"),
        ("DynamicData", "https://dynamic-data.org/"),
        ("FlaUI", "https://github.com/FlaUI/FlaUI"),
        ("GitVersion(Task)", "https://github.com/GitTools/GitVersion"),
        ("Lazy Cache", "https://github.com/alastairtree/LazyCache"),
        ("Microsoft.CodeAnalysis.FxCopAnalyzers", "https://github.com/dotnet/roslyn-analyzers"),
        ("Microsoft.Extensions.DependencyInjection", "https://asp.net"),
        ("Microsoft.NET.Test.Sdk", "https://github.com/microsoft/vstest/"),
        ("Moq", "https://github.com/moq/moq4"),
        (".NET Core", "https://dotnet.microsoft.com/"),
        ("Octokit", "https://github.com/octokit/octokit.net"),
        ("ReactiveUI", "https://www.reactiveui.net/"),
        ("Semver", "https://github.com/maxhauser/semver"),
        ("WIX TOOLSET", "https://wixtoolset.org/"),
        ("WPF Animated GIF", "https://github.com/XamlAnimatedGif/WpfAnimatedGif"),
        ("xunit", "https://github.com/xunit/xunit"),
        ("xunit.runner.visualstudio", "https://github.com/xunit/visualstudio.xunit"),
    ];

    let mut rows = Vec::with_capacity(libraries.len());
    for (name, url) in libraries {
        rows.push(
            column![
                text(name).size(13),
                text(url).size(11),
            ]
            .spacing(2)
            .into()
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
    .width(Length::Fixed(400.0))
    .height(Length::Fixed(450.0))
    .into()
}
