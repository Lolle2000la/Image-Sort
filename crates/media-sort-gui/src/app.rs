use iced::window;
use iced::{Element, Subscription, Task};

use crate::message::Message;
use crate::state::AppState;

pub fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Tick(_instant) => {
            if state.should_exit {
                let _ = state.settings.save();
                return window::get_latest().and_then(window::close);
            }
            Task::none()
        }
        Message::SettingsLoaded(result) => match *result {
            Ok(settings) => {
                state.settings = settings;
                Task::none()
            }
            Err(err) => {
                log::error!("Failed to load settings: {err}");
                Task::none()
            }
        },
        Message::Quit => {
            let _ = state.settings.save();
            state.should_exit = true;
            Task::none()
        }
    }
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    use iced::widget::{column, text};

    let folder_text = state
        .current_folder
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "No folder selected".to_string());

    column![
        text("Media Sort v3.0.0").size(24),
        text(format!("Current folder: {folder_text}")),
        text("Use the folder panel to navigate."),
    ]
    .padding(20)
    .into()
}

pub fn theme(state: &AppState) -> iced::Theme {
    if state.settings.general.dark_mode {
        iced::Theme::Dark
    } else {
        iced::Theme::Light
    }
}

pub fn subscription(_state: &AppState) -> Subscription<Message> {
    iced::time::every(std::time::Duration::from_millis(16)).map(Message::Tick)
}
