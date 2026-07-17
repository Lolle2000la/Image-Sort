use iced::Task;

use crate::message::{MediaMessage, Message};
use crate::state::AppState;
use media_sort_core::media_type::MediaType;

pub fn select_and_load_entry(state: &mut AppState, index: usize) -> Task<Message> {
    let filtered = state.filtered_media_entries();
    let filtered_len = filtered.len();
    if filtered_len > 0 {
        let index = index.min(filtered_len - 1);
        let entry = filtered[index];
        let path = entry.path.clone();
        let media_type = entry.media_type;

        let start = index.saturating_sub(5);
        let end = (index + 6).min(filtered_len);
        let mut thumbnail_paths = Vec::new();
        for entry in filtered.iter().take(end).skip(start) {
            thumbnail_paths.push(entry.path.clone());
        }

        let mut preload_tasks = Vec::new();
        if index + 1 < filtered_len {
            let next_entry = filtered[index + 1];
            if next_entry.media_type == media_sort_core::media_type::MediaType::Image
                && !state.image_cache.contains(&next_entry.path)
            {
                preload_tasks.push(load_full_image(
                    next_entry.path.clone(),
                    next_entry.media_type,
                ));
            }
        }
        if index > 0 {
            let prev_entry = filtered[index - 1];
            if prev_entry.media_type == media_sort_core::media_type::MediaType::Image
                && !state.image_cache.contains(&prev_entry.path)
            {
                preload_tasks.push(load_full_image(
                    prev_entry.path.clone(),
                    prev_entry.media_type,
                ));
            }
        }

        drop(filtered);

        state
            .thumbnail_tracker
            .retain_paths(thumbnail_paths.clone());

        state.selected_index = Some(index);
        state.current_metadata = None;

        state.settings.general.last_selected_media = Some(path.to_string_lossy().to_string());
        let _ = state.settings.save();

        if media_type == media_sort_core::media_type::MediaType::Video {
            if let Some(ref sender) = state.video_sender {
                let _ = sender.try_send(
                    media_sort_backend::media::mpv_context::VideoCommand::Load(path.clone()),
                );
            }
            state.video_frame = None;
            state.video_rgba = None;
            state.video_width = 0;
            state.video_height = 0;
            state.video_position = 0.0;
            state.video_duration = 0.0;
            state.video_ready = false;
            state.video_seek_position = None;
            state.video_last_seek_time = None;
            if let Some(ref mut ap) = state.audio_player {
                ap.stop();
            }
            state.audio_playing = false;
            state.audio_position = 0.0;
        } else if media_type == media_sort_core::media_type::MediaType::Audio {
            if let Some(ref sender) = state.video_sender {
                let _ = sender
                    .try_send(media_sort_backend::media::mpv_context::VideoCommand::Deactivate);
            }
            state.video_frame = None;
            state.video_rgba = None;
            state.video_width = 0;
            state.video_height = 0;
            state.video_ready = false;
            if state.audio_playing
                && let Some(ref player) = state.audio_player
            {
                player.stop();
                if let Err(e) = player.play(&path) {
                    tracing::error!("Audio play failed: {e}");
                    state.audio_playing = false;
                } else {
                    state.audio_duration = player.duration();
                }
            }
        } else {
            if let Some(ref sender) = state.video_sender {
                let _ = sender
                    .try_send(media_sort_backend::media::mpv_context::VideoCommand::Deactivate);
            }
            state.video_frame = None;
            state.video_rgba = None;
            state.video_width = 0;
            state.video_height = 0;
            state.video_ready = false;
            if state.audio_playing {
                if let Some(ref player) = state.audio_player {
                    player.stop();
                }
                state.audio_playing = false;
                state.audio_position = 0.0;
            }
        }

        let mut tasks = vec![load_metadata(state, index)];

        tasks.push(scroll_to_selected_entry(state, index));

        state.selected_audio_cover = None;
        if media_type == media_sort_core::media_type::MediaType::Audio
            && let Some(bytes) = media_sort_backend::media::thumbnail::extract_audio_cover(&path)
            && let Ok(img) = image::load_from_memory(&bytes)
        {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            state.selected_audio_cover = Some(iced::widget::image::Handle::from_rgba(
                w,
                h,
                rgba.into_raw(),
            ));
        }

        if let Some(handle) = state.image_cache.get(&path) {
            state.selected_image = Some((path, handle.clone()));
        } else {
            state.selected_image = None;
            tasks.push(load_full_image(path, media_type));
        }

        for t in preload_tasks {
            tasks.push(t);
        }

        for p in thumbnail_paths {
            if !state.thumbnail_cache.contains(&p) {
                tasks.push(load_thumbnail(p, state.thumbnail_tracker.clone_checker()));
            }
        }
        Task::batch(tasks)
    } else {
        state.selected_index = None;
        state.current_metadata = None;
        state.selected_image = None;

        state.settings.general.last_selected_media = None;
        let _ = state.settings.save();
        if let Some(ref sender) = state.video_sender {
            let _ =
                sender.try_send(media_sort_backend::media::mpv_context::VideoCommand::Deactivate);
        }
        state.video_frame = None;
        state.video_rgba = None;
        state.video_width = 0;
        state.video_height = 0;
        state.video_ready = false;
        Task::none()
    }
}

/// Build a [`Task`] that scrolls the media grid so that the entry at
/// `index` is clearly visible. We use a *relative* scroll position so we
/// don't have to depend on `state.media_grid_scroll.viewport_width` being
/// up to date — that snapshot can briefly lag behind the actual layout
/// (e.g. when the user has just resized the window, or the very first
/// `on_scroll` after opening a folder hasn't fired yet). If we used an
/// absolute pixel offset computed from a stale viewport, we could end up
/// "scrolling" to a position that doesn't actually bring the selected
/// card into view. The scrollable resolves the relative position using
/// its own, always-current, content and viewport widths.
///
/// The relative position is `index / (n - 1)`, so the selected card ends
/// up at the corresponding proportional position in the content, well
/// inside the viewport for any sane window size.
pub fn scroll_to_selected_entry(state: &AppState, index: usize) -> Task<Message> {
    use crate::view::media_grid::MEDIA_GRID_SCROLLABLE_ID;

    let n = state.filtered_media_entries().len();
    let Some(relative_x) = relative_position_for(index, n) else {
        return Task::none();
    };

    iced::widget::operation::snap_to(
        MEDIA_GRID_SCROLLABLE_ID.clone(),
        iced::widget::scrollable::RelativeOffset {
            x: Some(relative_x),
            y: None,
        },
    )
}

/// Compute the relative horizontal scroll position (in `[0.0, 1.0]`) that
/// corresponds to `index` in a list of `total` entries. Returns `None` when
/// the list has zero or one entries, in which case there is nothing to
/// scroll to.
pub fn relative_position_for(index: usize, total: usize) -> Option<f32> {
    if total <= 1 {
        return None;
    }
    let clamped_index = index.min(total - 1);
    Some(clamped_index as f32 / (total - 1) as f32)
}

pub fn scroll_to_selected_folder(state: &AppState) -> Task<Message> {
    use crate::view::folder_panel::FOLDER_TREE_SCROLLABLE_ID;

    let visible = state.collect_visible_folders();
    let Some(idx) = state.selected_folder_idx.filter(|i| *i < visible.len()) else {
        return Task::none();
    };
    let Some(relative_y) = relative_position_for(idx, visible.len()) else {
        return Task::none();
    };

    iced::widget::operation::snap_to(
        FOLDER_TREE_SCROLLABLE_ID.clone(),
        iced::widget::scrollable::RelativeOffset {
            x: None,
            y: Some(relative_y),
        },
    )
}

pub fn load_visible_thumbnails(state: &mut AppState) -> Task<Message> {
    let entry_paths: Vec<std::path::PathBuf> = state
        .filtered_media_entries()
        .iter()
        .map(|e| e.path.clone())
        .collect();
    let load_queue = state.thumbnail_tracker.update_viewport(
        &state.media_grid_scroll,
        &entry_paths,
        state.settings.window_position.width,
    );

    Task::batch(
        load_queue
            .into_iter()
            .filter(|path| {
                !state.thumbnail_cache.contains(path) && !state.unsupported_files.contains(path)
            })
            .map(|path| load_thumbnail(path, state.thumbnail_tracker.clone_checker())),
    )
}

pub fn load_thumbnail(
    path: std::path::PathBuf,
    visible_tracker: std::sync::Arc<
        std::sync::RwLock<std::collections::HashSet<std::path::PathBuf>>,
    >,
) -> Task<Message> {
    Task::perform(
        async move {
            let path_clone = path.clone();
            let tracker = visible_tracker.clone();
            let result = tokio::task::spawn_blocking(move || {
                if let Ok(guard) = tracker.read()
                    && !guard.contains(&path_clone)
                {
                    return Ok(None);
                }
                match crate::subscriptions::prefetch::generate_thumbnail(&path_clone) {
                    Ok((w, h, rgba)) => Ok(Some((w, h, rgba))),
                    Err(()) => Err(()),
                }
            })
            .await
            .unwrap_or(Ok(None));
            (path, result)
        },
        |(path, result)| {
            Message::Media(match result {
                Ok(Some((w, h, rgba))) => MediaMessage::ThumbnailReady(path, w, h, rgba),
                Ok(None) => MediaMessage::ThumbnailCancelled(path),
                Err(()) => MediaMessage::ThumbnailFailed(path),
            })
        },
    )
}

pub fn open_externally(path: &std::path::Path) {
    let res = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(path).spawn()
    } else {
        std::process::Command::new("xdg-open").arg(path).spawn()
    };
    if let Err(e) = res {
        tracing::error!("Failed to open file externally: {e}");
    }
}

pub fn reveal_in_file_manager(path: &std::path::Path) {
    let res = if cfg!(target_os = "windows") {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()
            .map(|_| ())
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .map(|_| ())
    } else {
        let mut uri = String::from("file://");
        for ch in path.to_string_lossy().chars() {
            match ch {
                ' ' => uri.push_str("%20"),
                '%' => uri.push_str("%25"),
                '#' => uri.push_str("%23"),
                '?' => uri.push_str("%3f"),
                _ => uri.push(ch),
            }
        }
        let mut success = false;
        if let Ok(output) = std::process::Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.FileManager1",
                "--type=method_call",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:{}", uri),
                "string:",
            ])
            .output()
        {
            success = output.status.success();
        }

        if success {
            Ok(())
        } else if let Some(parent) = path.parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map(|_| ())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No parent directory",
            ))
        }
    };
    if let Err(e) = res {
        tracing::error!("Failed to reveal file in file manager: {e}");
    }
}

pub fn load_full_image(path: std::path::PathBuf, media_type: MediaType) -> Task<Message> {
    if media_type != MediaType::Image {
        return Task::none();
    }
    Task::perform(
        async move {
            let path_clone = path.clone();
            let res = tokio::task::spawn_blocking(move || {
                media_sort_backend::media::image_decoder::load_image(&path_clone)
                    .map(|img| {
                        use image::GenericImageView;
                        let (w, h) = img.dimensions();
                        let rgba = img.to_rgba8().into_raw();
                        (w, h, rgba)
                    })
                    .map_err(|e| e.to_string())
            })
            .await
            .unwrap_or_else(|e| Err(format!("Join error: {e}")));
            (path, res)
        },
        |(path, res)| Message::Media(MediaMessage::ImageLoaded(path, res)),
    )
}

pub fn load_metadata(state: &AppState, index: usize) -> Task<Message> {
    let entries = state.filtered_media_entries();
    let Some(entry) = entries.get(index) else {
        return Task::none();
    };

    let path = entry.path.clone();
    let media_type = entry.media_type;

    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || match media_type {
                MediaType::Image => {
                    media_sort_backend::metadata::image_meta::extract_image_metadata(&path)
                        .map_err(|e| e.to_string())
                }
                MediaType::Audio => {
                    media_sort_backend::metadata::audio_meta::extract_audio_metadata(&path)
                        .map_err(|e| e.to_string())
                }
                MediaType::Video => {
                    media_sort_backend::metadata::video_meta::extract_video_metadata(&path)
                        .map_err(|e| e.to_string())
                }
            })
            .await
            .unwrap_or_else(|e| Err(format!("Join error: {e}")))
        },
        |result| Message::Media(MediaMessage::MetadataLoaded(result)),
    )
}
