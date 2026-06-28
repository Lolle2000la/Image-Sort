use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, text};
use iced::{Color, Element, Length, Alignment};

use crate::message::Message;
use crate::state::AppState;
use crate::subscriptions::keyboard::format_keybinding;

const BOLD_FONT: iced::Font = iced::Font {
    weight: iced::font::Weight::Bold,
    ..iced::Font::DEFAULT
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocaleOption {
    code: String,
    display: String,
}

impl std::fmt::Display for LocaleOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display)
    }
}

fn locale_options() -> Vec<LocaleOption> {
    media_sort_core::l10n::AVAILABLE_LOCALES
        .iter()
        .map(|&code| LocaleOption {
            code: code.to_string(),
            display: media_sort_core::l10n::locale_display_name(code).to_string(),
        })
        .collect()
}

fn get_keybinding(state: &AppState, idx: usize) -> &media_sort_core::settings::keybindings::KeyBinding {
    let kb = &state.settings.keybindings;
    match idx {
        0 => &kb.move_to_folder,
        1 => &kb.delete,
        2 => &kb.rename,
        3 => &kb.go_left,
        4 => &kb.go_right,
        5 => &kb.create_folder,
        6 => &kb.folder_up,
        7 => &kb.folder_left,
        8 => &kb.folder_down,
        9 => &kb.folder_right,
        10 => &kb.undo,
        11 => &kb.redo,
        12 => &kb.open_folder,
        13 => &kb.open_selected_folder,
        14 => &kb.pin,
        15 => &kb.pin_selected,
        16 => &kb.unpin,
        17 => &kb.move_pinned_up,
        18 => &kb.move_pinned_down,
        19 => &kb.search_images,
        20 => &kb.toggle_metadata_panel,
        _ => panic!("Invalid keybinding index: {}", idx),
    }
}

fn keybinding_row<'a>(
    state: &'a AppState,
    idx: usize,
    label: &'static str,
) -> Element<'a, Message> {
    let binding = get_keybinding(state, idx);
    let is_editing = state.editing_keybinding == Some(idx);
    let shortcut_text = if is_editing {
        "Press a key...".to_string()
    } else {
        format_keybinding(binding)
    };

    let btn_label = if is_editing {
        text(shortcut_text).color(Color::from_rgb(1.0, 0.8, 0.0)).size(12)
    } else {
        text(shortcut_text).size(12)
    };

    row![
        text(label).size(12).width(Length::Fixed(240.0)),
        button(btn_label)
            .on_press(Message::EditKeyBinding(idx))
            .style(iced::widget::button::secondary)
            .width(Length::Fixed(120.0)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn keybinding_section<'a>(
    title: &'static str,
    items: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    column(
        std::iter::once(text(title).font(BOLD_FONT).size(14).into())
            .chain(items.into_iter())
    )
    .spacing(8)
    .into()
}

fn keybinding_subsection<'a>(
    title: &'static str,
    items: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    column(
        std::iter::once(text(title).size(12).into())
            .chain(items.into_iter())
    )
    .spacing(6)
    .padding(iced::Padding::new(0.0).left(12.0))
    .into()
}

pub fn settings_dialog_view(state: &AppState) -> Element<'_, Message> {
    let title = text("Settings").size(20).font(BOLD_FONT);

    // Tab bar
    let tab_bar = row![
        button(text("General settings").size(13))
            .on_press(Message::OpenSettings)
            .style(if !state.show_keybindings {
                iced::widget::button::primary
            } else {
                iced::widget::button::secondary
            }),
        button(text("Key bindings").size(13))
            .on_press(Message::OpenKeybindings)
            .style(if state.show_keybindings {
                iced::widget::button::primary
            } else {
                iced::widget::button::secondary
            }),
    ]
    .spacing(10);

    let tab_content: Element<'_, Message> = if !state.show_keybindings {
        // General settings tab
        let dark_mode_cb = checkbox("Dark Mode", state.settings.general.dark_mode)
            .on_toggle(|_| Message::ToggleDarkMode)
            .size(16);

        let animate_gifs_cb = checkbox("Play animated gifs", state.settings.general.animate_gifs)
            .on_toggle(|_| Message::ToggleAnimateGifs)
            .size(16);

        let animate_thumbs_cb = checkbox(
            "Preview animated gifs in thumbnails",
            state.settings.general.animate_gif_thumbnails,
        )
        .on_toggle(|_| Message::ToggleAnimateThumbnails)
        .size(16);

        let check_updates_cb = checkbox(
            "Check for updates on startup",
            state.settings.general.check_for_updates_on_startup,
        )
        .on_toggle(|_| Message::ToggleCheckForUpdates)
        .size(16);

        let install_prerelease_cb = checkbox(
            "Install prerelease builds",
            state.settings.general.install_prerelease_builds,
        )
        .on_toggle(|_| Message::ToggleInstallPrerelease)
        .size(16);

        let reopen_folder_cb = checkbox(
            "Reopen last opened folder on startup",
            state.settings.general.reopen_last_opened_folder,
        )
        .on_toggle(|_| Message::ToggleReopenFolder)
        .size(16);

        #[allow(unused_mut)]
        let mut settings_col = column![
            column![
                text("Appearance").font(BOLD_FONT).size(14),
                dark_mode_cb,
                reopen_folder_cb,
            ].spacing(8),
            column![
                text("Animated Gifs").font(BOLD_FONT).size(14),
                animate_gifs_cb,
                animate_thumbs_cb,
            ].spacing(8),
            column![
                text("Updates").font(BOLD_FONT).size(14),
                check_updates_cb,
                install_prerelease_cb,
            ].spacing(8),
            column![
                text(state.l10n.tr("settings-language")).font(BOLD_FONT).size(14),
                pick_list(
                    locale_options(),
                    Some(LocaleOption {
                        code: state.l10n.locale(),
                        display: media_sort_core::l10n::locale_display_name(&state.l10n.locale()).to_string(),
                    }),
                    |opt: LocaleOption| Message::ChangeLanguage(opt.code),
                ).width(Length::Fixed(200.0)),
            ].spacing(8),
        ]
        .spacing(16);

        #[cfg(target_os = "windows")]
        {
            let integration_with_windows_cb = checkbox(
                "Add a shortcut to open a folder in Image Sort directly from the context menu in Windows Explorer.",
                state.settings.general.integration_with_windows,
            )
            .on_toggle(|_| Message::ToggleIntegrationWithWindows)
            .size(16);

            settings_col = settings_col.push(
                column![
                    text("Integration with Windows").font(BOLD_FONT).size(14),
                    integration_with_windows_cb,
                ].spacing(8),
            );
        }

        scrollable(settings_col)
            .height(Length::Fill)
            .into()
    } else {
        // Key bindings tab
        let restore_btn = button(text("Restore default key bindings").size(12))
            .on_press(Message::RestoreDefaultKeyBindings)
            .style(iced::widget::button::secondary);

        // Images Section
        let images_management = keybinding_subsection("Management", vec![
            keybinding_row(state, 0, "Move"),
            keybinding_row(state, 1, "Delete"),
            keybinding_row(state, 2, "Rename"),
        ]);
        let images_selection = keybinding_subsection("Selection", vec![
            keybinding_row(state, 3, "Select image on the left"),
            keybinding_row(state, 4, "Select image on the right"),
        ]);
        let images_search = keybinding_subsection("Search", vec![
            keybinding_row(state, 19, "Search images... (press 'Tab' to leave)"),
        ]);
        let images_metadata = keybinding_subsection("Metadata", vec![
            keybinding_row(state, 20, "Open/Close metadata panel"),
        ]);
        let images_section = keybinding_section("Images", vec![
            images_management,
            images_selection,
            images_search,
            images_metadata,
        ]);

        // Folders Section
        let folders_management = keybinding_subsection("Management", vec![
            keybinding_row(state, 5, "Create Folder"),
        ]);
        let folders_open = keybinding_subsection("Open", vec![
            keybinding_row(state, 12, "Open folder"),
            keybinding_row(state, 13, "Open selected folder"),
        ]);
        let folders_pinned = keybinding_subsection("Pinned", vec![
            keybinding_row(state, 14, "Pin"),
            keybinding_row(state, 15, "Pin selected"),
            keybinding_row(state, 16, "Unpin"),
            keybinding_row(state, 17, "Move the selected pinned folder up"),
            keybinding_row(state, 18, "Move the selected pinned folder down"),
        ]);
        let folders_selection = keybinding_subsection("Selection", vec![
            keybinding_row(state, 6, "Select the folder above"),
            keybinding_row(state, 7, "Collapse folder"),
            keybinding_row(state, 8, "Select the folder below"),
            keybinding_row(state, 9, "Expand folder (list sub-folders)"),
        ]);
        let folders_section = keybinding_section("Folders", vec![
            folders_management,
            folders_open,
            folders_pinned,
            folders_selection,
        ]);

        // Other Section
        let other_history = keybinding_subsection("History", vec![
            keybinding_row(state, 10, "Undo"),
            keybinding_row(state, 11, "Redo"),
        ]);
        let other_section = keybinding_section("Other", vec![
            other_history,
        ]);

        let bindings_column = column![
            restore_btn,
            images_section,
            folders_section,
            other_section,
        ]
        .spacing(16);

        scrollable(bindings_column)
            .height(Length::Fill)
            .into()
    };

    let close_btn = button(text("Close"))
        .on_press(Message::CloseSettings)
        .style(iced::widget::button::primary);

    container(
        column![
            title,
            tab_bar,
            tab_content,
            close_btn,
        ]
        .spacing(16)
        .align_x(Alignment::Start),
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
    .width(Length::Fixed(440.0))
    .height(Length::Fixed(500.0))
    .into()
}
