use iced::widget::{
    button as button_widget, container, pick_list as pick_list_widget, text_input as text_input_widget,
};
use iced::{Border, Color, Theme};

// Zinc Dark Palette
pub const BG: Color = Color::from_rgb(0.035, 0.035, 0.043);
pub const SURFACE: Color = Color::from_rgb(0.094, 0.094, 0.11);
pub const SURFACE_HIGHLIGHT: Color = Color::from_rgb(0.12, 0.12, 0.14);
pub const BORDER: Color = Color::from_rgb(0.153, 0.153, 0.161);
pub const ACCENT: Color = Color::from_rgb(0.063, 0.722, 0.514);
pub const ACCENT_MUTED: Color = Color::from_rgba(0.063, 0.722, 0.514, 0.08);

pub const TEXT_PRIMARY: Color = Color::from_rgb(0.98, 0.98, 0.99);
pub const TEXT_MUTED: Color = Color::from_rgb(0.63, 0.63, 0.66);
pub const ERROR: Color = Color::from_rgb(0.94, 0.27, 0.27);
pub const MODAL_BACKDROP: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.75);

pub fn button(_theme: &Theme, status: button_widget::Status) -> button_widget::Style {
    let base_border = Border {
        color: BORDER,
        width: 1.0,
        radius: 6.0.into(),
    };
    match status {
        button_widget::Status::Hovered => button_widget::Style {
            background: Some(SURFACE_HIGHLIGHT.into()),
            text_color: TEXT_PRIMARY,
            border: base_border,
            ..Default::default()
        },
        _ => button_widget::Style {
            background: Some(SURFACE.into()),
            text_color: TEXT_MUTED,
            border: base_border,
            ..Default::default()
        },
    }
}

pub fn text_input(theme: &Theme, status: text_input_widget::Status) -> text_input_widget::Style {
    let mut style = text_input_widget::default(theme, status);
    style.border.radius = 6.0.into();
    style.border.color = match status {
        text_input_widget::Status::Focused { .. } => ACCENT,
        text_input_widget::Status::Hovered => TEXT_MUTED,
        _ => BORDER,
    };
    style.background = BG.into();
    style
}

pub fn text_input_invalid(theme: &Theme, status: text_input_widget::Status) -> text_input_widget::Style {
    let mut style = text_input(theme, status);
    style.border.color = ERROR;
    style
}

pub fn pick_list(theme: &Theme, status: pick_list_widget::Status) -> pick_list_widget::Style {
    let mut style = pick_list_widget::default(theme, status);
    style.border.radius = 6.0.into();
    style.border.color = match status {
        pick_list_widget::Status::Opened { .. } | pick_list_widget::Status::Hovered => TEXT_MUTED,
        _ => BORDER,
    };
    style.background = BG.into();
    style
}

pub fn outline_card(_: &Theme) -> container::Style {
    container::Style {
        background: Some(BG.into()),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub fn surface_card(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE.into()),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 16.0.into(),
        },
        ..Default::default()
    }
}
