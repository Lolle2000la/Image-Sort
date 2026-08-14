use iced::{Color, Theme};

/// Represents a KDE Breeze color scheme section (Window, View, Header, Button, Selection).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezeColorGroup {
    pub background_normal: Color,
    pub background_alternate: Color,
    pub foreground_normal: Color,
    pub foreground_inactive: Color,
}

/// Represents the full, official KDE Plasma Breeze color palette definition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreezePalette {
    pub window: BreezeColorGroup,
    pub view: BreezeColorGroup,
    pub header: BreezeColorGroup,
    pub button: BreezeColorGroup,
    pub selection: BreezeColorGroup,
    pub decoration_focus: Color,
    pub decoration_hover: Color,
    pub foreground_link: Color,
    pub foreground_negative: Color,
    pub foreground_neutral: Color,
    pub foreground_positive: Color,
}

/// Official KDE Breeze Light palette constant struct.
pub static BREEZE_LIGHT_PALETTE: BreezePalette = BreezePalette {
    window: BreezeColorGroup {
        background_normal: Color::from_rgb8(239, 240, 241),
        background_alternate: Color::from_rgb8(227, 229, 231),
        foreground_normal: Color::from_rgb8(35, 38, 41),
        foreground_inactive: Color::from_rgb8(112, 125, 138),
    },
    view: BreezeColorGroup {
        background_normal: Color::from_rgb8(255, 255, 255),
        background_alternate: Color::from_rgb8(247, 247, 247),
        foreground_normal: Color::from_rgb8(35, 38, 41),
        foreground_inactive: Color::from_rgb8(112, 125, 138),
    },
    header: BreezeColorGroup {
        background_normal: Color::from_rgb8(222, 224, 226),
        background_alternate: Color::from_rgb8(239, 240, 241),
        foreground_normal: Color::from_rgb8(35, 38, 41),
        foreground_inactive: Color::from_rgb8(112, 125, 138),
    },
    button: BreezeColorGroup {
        background_normal: Color::from_rgb8(252, 252, 252),
        background_alternate: Color::from_rgb8(163, 212, 250),
        foreground_normal: Color::from_rgb8(35, 38, 41),
        foreground_inactive: Color::from_rgb8(112, 125, 138),
    },
    selection: BreezeColorGroup {
        background_normal: Color::from_rgb8(61, 174, 233),
        background_alternate: Color::from_rgb8(163, 212, 250),
        foreground_normal: Color::from_rgb8(255, 255, 255),
        foreground_inactive: Color::from_rgb8(112, 125, 138),
    },
    decoration_focus: Color::from_rgb8(61, 174, 233),
    decoration_hover: Color::from_rgb8(61, 174, 233),
    foreground_link: Color::from_rgb8(41, 128, 185),
    foreground_negative: Color::from_rgb8(218, 68, 83),
    foreground_neutral: Color::from_rgb8(246, 116, 0),
    foreground_positive: Color::from_rgb8(39, 174, 96),
};

/// Official KDE Breeze Dark palette constant struct.
pub static BREEZE_DARK_PALETTE: BreezePalette = BreezePalette {
    window: BreezeColorGroup {
        background_normal: Color::from_rgb8(32, 35, 38),
        background_alternate: Color::from_rgb8(41, 44, 48),
        foreground_normal: Color::from_rgb8(252, 252, 252),
        foreground_inactive: Color::from_rgb8(161, 169, 177),
    },
    view: BreezeColorGroup {
        background_normal: Color::from_rgb8(20, 22, 24),
        background_alternate: Color::from_rgb8(29, 31, 34),
        foreground_normal: Color::from_rgb8(252, 252, 252),
        foreground_inactive: Color::from_rgb8(161, 169, 177),
    },
    header: BreezeColorGroup {
        background_normal: Color::from_rgb8(41, 44, 48),
        background_alternate: Color::from_rgb8(32, 35, 38),
        foreground_normal: Color::from_rgb8(252, 252, 252),
        foreground_inactive: Color::from_rgb8(161, 169, 177),
    },
    button: BreezeColorGroup {
        background_normal: Color::from_rgb8(41, 44, 48),
        background_alternate: Color::from_rgb8(30, 87, 116),
        foreground_normal: Color::from_rgb8(252, 252, 252),
        foreground_inactive: Color::from_rgb8(161, 169, 177),
    },
    selection: BreezeColorGroup {
        background_normal: Color::from_rgb8(61, 174, 233),
        background_alternate: Color::from_rgb8(30, 87, 116),
        foreground_normal: Color::from_rgb8(252, 252, 252),
        foreground_inactive: Color::from_rgb8(161, 169, 177),
    },
    decoration_focus: Color::from_rgb8(61, 174, 233),
    decoration_hover: Color::from_rgb8(61, 174, 233),
    foreground_link: Color::from_rgb8(29, 153, 243),
    foreground_negative: Color::from_rgb8(218, 68, 83),
    foreground_neutral: Color::from_rgb8(246, 116, 0),
    foreground_positive: Color::from_rgb8(39, 174, 96),
};

/// Creates the KDE Breeze Light theme tailored for iced.
pub fn breeze_light() -> Theme {
    let p = &BREEZE_LIGHT_PALETTE;
    Theme::custom_with_fn(
        "Breeze Light",
        iced::theme::Palette {
            background: p.window.background_normal,
            text: p.window.foreground_normal,
            primary: p.decoration_focus,
            success: p.foreground_positive,
            danger: p.foreground_negative,
            warning: p.foreground_neutral,
        },
        |palette| {
            let mut extended = iced::theme::palette::Extended::generate(palette);

            // Breeze Light card/view surface (#FFFFFF)
            extended.background.weak.color = p.view.background_normal;
            extended.background.weak.text = p.view.foreground_normal;

            // Breeze Light header/panel surface (#DEE0E2)
            extended.background.strong.color = p.header.background_normal;
            extended.background.strong.text = p.header.foreground_normal;

            // Primary buttons/selection: #3DAEE9 background with selection foreground (#FFFFFF)
            extended.primary.base.color = p.selection.background_normal;
            extended.primary.base.text = p.selection.foreground_normal;
            extended.primary.weak.color = p.button.background_alternate;
            extended.primary.weak.text = p.button.foreground_normal;
            extended.primary.strong.color = p.foreground_link;
            extended.primary.strong.text = p.selection.foreground_normal;

            // Secondary buttons in Breeze Light: button background (#FCFCFC) with button text (#232629)
            extended.secondary.base.color = p.button.background_normal;
            extended.secondary.base.text = p.button.foreground_normal;
            extended.secondary.weak.color = p.window.background_normal;
            extended.secondary.weak.text = p.window.foreground_normal;
            extended.secondary.strong.color = p.header.background_normal;
            extended.secondary.strong.text = p.header.foreground_normal;

            // Action button text: selection foreground (#FFFFFF)
            extended.danger.base.text = p.selection.foreground_normal;
            extended.success.base.text = p.selection.foreground_normal;
            extended.warning.base.text = p.selection.foreground_normal;

            extended
        },
    )
}

/// Creates the KDE Breeze Dark theme tailored for iced.
pub fn breeze_dark() -> Theme {
    let p = &BREEZE_DARK_PALETTE;
    Theme::custom_with_fn(
        "Breeze Dark",
        iced::theme::Palette {
            background: p.window.background_normal,
            text: p.window.foreground_normal,
            primary: p.decoration_focus,
            success: p.foreground_positive,
            danger: p.foreground_negative,
            warning: p.foreground_neutral,
        },
        |palette| {
            let mut extended = iced::theme::palette::Extended::generate(palette);

            // Breeze Dark view/card surface (#141618)
            extended.background.weak.color = p.view.background_normal;
            extended.background.weak.text = p.view.foreground_normal;

            // Breeze Dark header/panel surface (#292C30)
            extended.background.strong.color = p.header.background_normal;
            extended.background.strong.text = p.header.foreground_normal;

            // Primary buttons/selection: #3DAEE9 background with selection foreground (#FCFCFC)
            extended.primary.base.color = p.selection.background_normal;
            extended.primary.base.text = p.selection.foreground_normal;
            extended.primary.weak.color = p.button.background_alternate;
            extended.primary.weak.text = p.selection.foreground_normal;
            extended.primary.strong.color = p.foreground_link;
            extended.primary.strong.text = p.selection.foreground_normal;

            // Secondary buttons in Breeze Dark: button background (#292C30) with button text (#FCFCFC)
            extended.secondary.base.color = p.button.background_normal;
            extended.secondary.base.text = p.button.foreground_normal;
            extended.secondary.weak.color = p.window.background_normal;
            extended.secondary.weak.text = p.window.foreground_normal;
            extended.secondary.strong.color = p.window.background_alternate;
            extended.secondary.strong.text = p.window.foreground_normal;

            // Action button text: selection foreground (#FCFCFC)
            extended.danger.base.text = p.selection.foreground_normal;
            extended.success.base.text = p.selection.foreground_normal;
            extended.warning.base.text = p.selection.foreground_normal;

            extended
        },
    )
}
