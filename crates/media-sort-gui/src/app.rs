use iced::{Element, Subscription, Task};

use crate::message::Message;
use crate::state::AppState;
use crate::view;

#[cfg(feature = "demo")]
use iced_automation::AutomationStateTrait;

pub fn update(state: &mut AppState, message: Message) -> Task<Message> {
    crate::update::update(state, message)
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    let base_view = view::main_layout::main_layout_view(state);
    #[cfg(feature = "demo")]
    if let Some(automation) = state.automation() {
        return iced_automation::wrap_view(base_view, automation);
    }
    base_view
}

pub fn theme(state: &AppState) -> iced::Theme {
    match state.settings.general.theme.as_str() {
        "Dark" => iced::Theme::Dark,
        "Dracula" => iced::Theme::Dracula,
        "Nord" => iced::Theme::Nord,
        "SolarizedLight" => iced::Theme::SolarizedLight,
        "SolarizedDark" => iced::Theme::SolarizedDark,
        "GruvboxLight" => iced::Theme::GruvboxLight,
        "GruvboxDark" => iced::Theme::GruvboxDark,
        "CatppuccinLatte" => iced::Theme::CatppuccinLatte,
        "CatppuccinFrappe" => iced::Theme::CatppuccinFrappe,
        "CatppuccinMacchiato" => iced::Theme::CatppuccinMacchiato,
        "CatppuccinMocha" => iced::Theme::CatppuccinMocha,
        "TokyoNight" => iced::Theme::TokyoNight,
        "TokyoNightStorm" => iced::Theme::TokyoNightStorm,
        "TokyoNightLight" => iced::Theme::TokyoNightLight,
        "KanagawaWave" => iced::Theme::KanagawaWave,
        "KanagawaDragon" => iced::Theme::KanagawaDragon,
        "KanagawaLotus" => iced::Theme::KanagawaLotus,
        "Moonfly" => iced::Theme::Moonfly,
        "Nightfly" => iced::Theme::Nightfly,
        "Oxocarbon" => iced::Theme::Oxocarbon,
        "Ferra" => iced::Theme::Ferra,
        _ => iced::Theme::Light,
    }
}

pub fn subscription(state: &AppState) -> Subscription<Message> {
    Subscription::batch([base_subscription(), video_subscription(state)])
}

fn base_subscription() -> Subscription<Message> {
    Subscription::batch([
        iced::time::every(std::time::Duration::from_millis(16)).map(Message::Tick),
        crate::subscriptions::keyboard::keyboard_subscription(),
        iced::event::listen().map(Message::EventOccurred),
    ])
}

fn video_subscription(state: &AppState) -> Subscription<Message> {
    let hwdec = state.settings.general.video_hardware_decoding;
    let zero_copy = state.settings.general.video_zero_copy;
    // When both zero-copy and hwdec are active, frames stay on the GPU and
    // never touch the CPU, so there is no benefit in capping the render size.
    let (max_w, max_h) = if hwdec && zero_copy {
        (None, None)
    } else {
        (Some(960), Some(540))
    };
    let config = iced_mpv::PlayerConfig {
        max_frame_width: max_w,
        max_frame_height: max_h,
        enable_hwdec: hwdec,
        enable_zero_copy: zero_copy,
    };
    iced_mpv::VideoPlayer::subscription_with_config_and_map(config, Message::Video)
}

/// Like [`subscription`], but without the video worker. The headless demo
/// export renders many app instances in parallel and never plays videos, so
/// skipping the worker avoids spawning one libmpv `MpvContext` per render.
#[cfg(feature = "demo")]
pub(crate) fn demo_subscription(_state: &AppState) -> Subscription<Message> {
    base_subscription()
}
