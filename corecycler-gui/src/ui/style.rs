use iced::widget::{button, container};
use iced::{Border, Color, Theme};

// Modern monochrome palette with accent
pub const BG_DARK: Color = Color::from_rgb(0.06, 0.06, 0.06);
pub const BG_SURFACE: Color = Color::from_rgb(0.10, 0.10, 0.10);
pub const BG_CARD: Color = Color::from_rgb(0.13, 0.13, 0.13);
pub const BG_ELEVATED: Color = Color::from_rgb(0.16, 0.16, 0.16);
pub const BG_INPUT: Color = Color::from_rgb(0.09, 0.09, 0.09);

pub const TEXT_PRIMARY: Color = Color::from_rgb(0.95, 0.95, 0.95);
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.55, 0.55, 0.55);
pub const TEXT_MUTED: Color = Color::from_rgb(0.38, 0.38, 0.38);

pub const ACCENT: Color = Color::WHITE;
pub const BORDER: Color = Color::from_rgb(0.20, 0.20, 0.20);
pub const BORDER_LIGHT: Color = Color::from_rgb(0.25, 0.25, 0.25);

pub const SUCCESS: Color = Color::from_rgb(0.35, 0.85, 0.45);
pub const ERROR: Color = Color::from_rgb(0.95, 0.30, 0.30);
pub const WARNING: Color = Color::from_rgb(0.95, 0.75, 0.15);

pub const CORE_IDLE: Color = Color::from_rgb(0.15, 0.15, 0.15);
pub const CORE_TESTING: Color = Color::from_rgb(0.95, 0.95, 0.95);
pub const CORE_TESTING_TEXT: Color = Color::from_rgb(0.06, 0.06, 0.06);
pub const CORE_PASSED: Color = Color::from_rgb(0.18, 0.18, 0.18);
pub const CORE_ERROR: Color = Color::from_rgb(0.30, 0.08, 0.08);
pub const CORE_SKIPPED: Color = Color::from_rgb(0.12, 0.12, 0.12);

/// Container style for cards / sections
pub fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(BG_CARD)),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Container style for the main surface
pub fn surface_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(BG_SURFACE)),
        border: Border {
            color: BORDER,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Container style for core status boxes
pub fn core_box_style(bg: Color) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

// Tab colors
pub const TAB_ACTIVE_BG: Color = Color::from_rgb(0.22, 0.50, 0.95);
pub const TAB_INACTIVE_BG: Color = Color::from_rgb(0.14, 0.14, 0.14);
pub const TAB_INACTIVE_HOVER: Color = Color::from_rgb(0.20, 0.20, 0.20);

/// Active tab button style
pub fn tab_active(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(TAB_ACTIVE_BG)),
        text_color: Color::WHITE,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Inactive tab button style
pub fn tab_inactive(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => TAB_INACTIVE_HOVER,
        _ => TAB_INACTIVE_BG,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: TEXT_SECONDARY,
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}
