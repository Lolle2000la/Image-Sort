use iced::Task;

use crate::message::{Message, SettingsMessage};
use crate::state::AppState;

pub fn handle_settings_message(state: &mut AppState, msg: SettingsMessage) -> Task<Message> {
    match msg {
        SettingsMessage::ToggleMetadataPanel => {
            state.metadata_panel_expanded = !state.metadata_panel_expanded;
            state.settings.metadata_panel.is_expanded = state.metadata_panel_expanded;
            let _ = state.settings.save();
            Task::none()
        }
        SettingsMessage::EditKeyBinding(index) => {
            state.editing_keybinding = Some(index);
            state.waiting_for_key = true;
            Task::none()
        }
        SettingsMessage::Open => {
            state.show_settings = true;
            state.show_keybindings = false;
            Task::none()
        }
        SettingsMessage::Close => {
            state.show_settings = false;
            state.show_keybindings = false;
            state.editing_keybinding = None;
            state.waiting_for_key = false;
            Task::done(Message::Settings(SettingsMessage::Save))
        }
        SettingsMessage::ChangeLanguage(locale) => {
            state.l10n.set_locale(&locale);
            state.settings.general.locale = Some(locale);
            let _ = state.settings.save();
            state.search_placeholder = state.l10n.tr("keybindings-search-images");
            state.rename_placeholder = state.l10n.tr("ui-enter-new-name");
            state.create_folder_placeholder = state.l10n.tr("ui-folder-name-placeholder");
            Task::none()
        }
        SettingsMessage::SetTheme(theme) => {
            state.settings.general.theme = theme;
            let _ = state.settings.save();
            Task::none()
        }
        SettingsMessage::ToggleReopenFolder => {
            state.settings.general.reopen_last_opened_folder =
                !state.settings.general.reopen_last_opened_folder;
            let _ = state.settings.save();
            Task::none()
        }
        SettingsMessage::StartDragFolderDivider => {
            state.dragging_folder_divider = true;
            Task::none()
        }
        SettingsMessage::StartDragMetadataDivider => {
            state.dragging_metadata_divider = true;
            Task::none()
        }
        SettingsMessage::ToggleAnimateGifs => {
            state.settings.general.animate_gifs = !state.settings.general.animate_gifs;
            let _ = state.settings.save();
            Task::none()
        }
        SettingsMessage::Save => {
            let _ = state.settings.save();
            Task::none()
        }
        SettingsMessage::OpenKeybindings => {
            state.show_settings = true;
            state.show_keybindings = true;
            Task::none()
        }
        SettingsMessage::RestoreDefaultKeyBindings => {
            state.settings.keybindings =
                media_sort_core::settings::keybindings::KeyBindings::default();
            let _ = state.settings.save();
            Task::none()
        }
        #[cfg(feature = "velopack")]
        SettingsMessage::ToggleCheckForUpdates => {
            state.settings.general.check_for_updates_on_startup =
                !state.settings.general.check_for_updates_on_startup;
            let _ = state.settings.save();
            Task::none()
        }
        #[cfg(feature = "velopack")]
        SettingsMessage::ToggleInstallPrerelease => {
            state.settings.general.install_prerelease_builds =
                !state.settings.general.install_prerelease_builds;
            let _ = state.settings.save();
            Task::none()
        }
        #[cfg(target_os = "windows")]
        SettingsMessage::ToggleIntegrationWithWindows => {
            state.settings.general.integration_with_windows =
                !state.settings.general.integration_with_windows;
            let enabled = state.settings.general.integration_with_windows;
            let _ = state.settings.save();

            if enabled {
                if let Ok(exe) = std::env::current_exe()
                    && let Some(exe_str) = exe.to_str()
                {
                    let _ = media_sort_backend::platform::windows_shell::register(exe_str);
                }
            } else {
                let _ = media_sort_backend::platform::windows_shell::unregister();
            }
            Task::none()
        }
    }
}

pub fn handle_settings_loaded(
    state: &mut AppState,
    result: Box<Result<media_sort_core::settings::store::SettingsStore, String>>,
) -> Task<Message> {
    match *result {
        Ok(settings) => {
            state.settings = settings;
            #[cfg(target_os = "windows")]
            {
                if state.settings.general.integration_with_windows {
                    if let Ok(exe) = std::env::current_exe()
                        && let Some(exe_str) = exe.to_str()
                    {
                        let _ = media_sort_backend::platform::windows_shell::register(exe_str);
                    }
                }
            }
            Task::none()
        }
        Err(err) => {
            tracing::error!("Failed to load settings: {err}");
            Task::none()
        }
    }
}
