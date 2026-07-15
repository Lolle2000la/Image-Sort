pub mod folder;
pub mod keyboard;
pub mod media;
pub mod settings;
pub mod tasks;
pub mod video;

#[cfg(test)]
mod tests;

use iced::Task;

use crate::message::Message;
use crate::state::AppState;

#[cfg(feature = "demo")]
use iced_automation::AutomationStateTrait;

pub fn update(state: &mut AppState, message: Message) -> Task<Message> {
    #[cfg(feature = "demo")]
    if let Some(task) =
        iced_automation::handle_automation_message(state, &message, Message::AutomationBounds)
    {
        return task;
    }

    match message {
        Message::Tick(_instant) => handle_tick(state, _instant),
        Message::Video(video_msg) => video::handle_video_message(state, video_msg),
        Message::Folder(folder_msg) => folder::handle_folder_message(state, folder_msg),
        Message::Media(media_msg) => media::handle_media_message(state, media_msg),
        Message::Settings(settings_msg) => settings::handle_settings_message(state, settings_msg),
        Message::KeyCaptured(key, ctrl, shift, alt) => {
            keyboard::handle_key_captured(state, key, ctrl, shift, alt)
        }
        Message::SettingsLoaded(result) => settings::handle_settings_loaded(state, result),
        Message::MediaScanCompleted(result) => media::handle_media_scan_completed(state, result),
        Message::Quit => {
            let _ = state.settings.save();
            state.should_exit = true;
            Task::none()
        }
        Message::EventOccurred(event) => handle_event_occurred(state, event),
        Message::OpenCredits => {
            state.show_credits = true;
            Task::none()
        }
        Message::CloseCredits => {
            state.show_credits = false;
            Task::none()
        }
        #[cfg(feature = "velopack")]
        Message::Update(update_msg) => handle_update_message(state, update_msg),
        #[cfg(feature = "demo")]
        Message::AutomationBounds(_) | Message::AutomationVirtualTick(_) => Task::none(),
    }
}

fn handle_tick(state: &mut AppState, _instant: std::time::Instant) -> Task<Message> {
    if state.should_exit {
        let _ = state.settings.save();
        return iced::window::latest().and_then(iced::window::close);
    }

    #[cfg(feature = "demo")]
    let automation_task =
        iced_automation::try_tick_state(state, _instant, update, Message::AutomationBounds);
    #[cfg(not(feature = "demo"))]
    let automation_task = Task::none();

    let bg_task = poll_background_channels(state);
    let refresh_thumbnails = state.thumbnail_tracker.tick();

    if let Some(ref player) = state.audio_player
        && state.audio_playing
    {
        if player.empty() {
            state.audio_playing = false;
            state.audio_position = 0.0;
        } else {
            state.audio_position = player.position();
            state.audio_duration = player.duration();
        }
    }

    if refresh_thumbnails {
        Task::batch(vec![
            tasks::load_visible_thumbnails(state),
            bg_task,
            automation_task,
        ])
    } else {
        Task::batch(vec![bg_task, automation_task])
    }
}

pub fn poll_background_channels(state: &mut AppState) -> Task<Message> {
    let mut bg_tasks = Vec::new();

    if let Some(ref rx) = state.folder_tree_receiver
        && let Ok(tree) = rx.try_recv()
    {
        state.folder_tree_receiver = None;
        state.folder_tree = tree;
        state.sync_selected_folder_idx();
    }

    let scan_finished = if let Some(ref rx) = state.scan_receiver {
        for path in rx.try_iter() {
            let media_type =
                crate::state::detect_media_type(&path, state.settings.general.animate_gifs);
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());
            state
                .media_entries
                .push(media_sort_core::models::MediaEntry {
                    path,
                    media_type,
                    file_name,
                });
        }
        matches!(
            rx.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Disconnected)
        )
    } else {
        false
    };

    if scan_finished {
        state.scan_receiver = None;
        state
            .media_entries
            .sort_by(|a, b| a.file_name.cmp(&b.file_name));
        let select_idx = state.pending_select_index.take().unwrap_or(0);
        state.thumbnail_tracker.cancel_debounce();
        bg_tasks.push(tasks::load_visible_thumbnails(state));
        bg_tasks.push(tasks::select_and_load_entry(state, select_idx));
    }

    Task::batch(bg_tasks)
}

#[cfg(feature = "demo")]
impl iced_automation::AutomationContext<Message> for AppState {
    fn poll_background(&mut self) -> Task<Message> {
        poll_background_channels(self)
    }
}

fn handle_event_occurred(state: &mut AppState, event: iced::Event) -> Task<Message> {
    match event {
        iced::Event::Window(iced::window::Event::CloseRequested) => {
            let _ = state.settings.save();
            iced::window::latest().and_then(iced::window::close)
        }
        iced::Event::Window(iced::window::Event::Resized(size)) => {
            state.settings.window_position.width = size.width.round() as u32;
            state.settings.window_position.height = size.height.round() as u32;
            #[cfg(feature = "demo")]
            if let Some(automation) = state.automation_mut() {
                automation.update_window_size(size.width, size.height);
            }

            state.thumbnail_tracker.handle_scroll();
            Task::none()
        }
        iced::Event::Window(iced::window::Event::Moved(point)) => {
            state.settings.window_position.left = point.x.round() as i32;
            state.settings.window_position.top = point.y.round() as i32;
            Task::none()
        }
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            if state.dragging_folder_divider {
                let new_width = position.x.round().clamp(100.0, 800.0) as u16;
                state.settings.general.folder_tree_width = new_width;
            }
            if state.dragging_metadata_divider {
                let window_width = state.settings.window_position.width as f32;
                let raw_width = window_width - position.x;
                let new_width = raw_width.round().clamp(100.0, 800.0) as u16;
                state.settings.metadata_panel.panel_width = new_width;
            }
            Task::none()
        }
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
            let mut saved = false;
            if state.dragging_folder_divider || state.dragging_metadata_divider {
                state.dragging_folder_divider = false;
                state.dragging_metadata_divider = false;
                let _ = state.settings.save();
                saved = true;
            }
            if state.dragging_pinned_folder.is_some() {
                state.dragging_pinned_folder = None;
                if !saved {
                    let _ = state.settings.save();
                }
            }
            Task::none()
        }
        _ => Task::none(),
    }
}

#[cfg(feature = "velopack")]
fn handle_update_message(
    state: &mut AppState,
    msg: crate::message::UpdateMessage,
) -> Task<Message> {
    use crate::message::UpdateMessage;
    match msg {
        UpdateMessage::CheckForUpdates => {
            let settings = state.settings.general.clone();
            Task::perform(
                async move { crate::check_for_update_async(&settings).await },
                |result| match result {
                    Ok(Some(info)) => {
                        Message::Update(UpdateMessage::UpdateAvailable(Box::new(info)))
                    }
                    Ok(None) => Message::Update(UpdateMessage::NoUpdateFound),
                    Err(e) => Message::Update(UpdateMessage::UpdateFailed(e)),
                },
            )
        }
        UpdateMessage::UpdateAvailable(info) => {
            state.show_update_prompt = true;
            state.pending_update = Some(*info);
            Task::none()
        }
        UpdateMessage::NoUpdateFound => {
            tracing::info!("Update check completed, no update available");
            Task::none()
        }
        UpdateMessage::UserConfirmedUpdate(info) => {
            state.show_update_prompt = false;
            state.pending_update = None;
            let allow_prerelease = state.settings.general.install_prerelease_builds
                || env!("CARGO_PKG_VERSION").contains('-');
            Task::perform(
                crate::download_and_apply_async(*info, allow_prerelease),
                |result| match result {
                    Ok(()) => Message::Update(UpdateMessage::UpdateFailed(
                        "Update applied, restarting...".into(),
                    )),
                    Err(e) => Message::Update(UpdateMessage::UpdateFailed(e)),
                },
            )
        }
        UpdateMessage::UpdateFailed(e) => {
            tracing::error!("Update failed: {e}");
            Task::none()
        }
        UpdateMessage::DismissUpdatePrompt => {
            state.show_update_prompt = false;
            state.pending_update = None;
            Task::none()
        }
    }
}
