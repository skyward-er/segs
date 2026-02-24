mod colors;
mod stack;

use colors::*;
use egui::{Color32, Shadow, Stroke, Theme, ThemePreference};
use segs_assets::{Font, fonts::Figtree};
pub use stack::{AppStyle, CtxStyleExt, UiStyleExt};

/// Setup egui styles to match SEGS UI design.
pub fn setup_style(ctx: &egui::Context) {
    // Override egui styles
    ctx.all_styles_mut(override_egui_styles);
    ctx.style_mut_of(Theme::Dark, override_dark_style);
    ctx.style_mut_of(Theme::Light, override_light_style);
    ctx.set_theme(ThemePreference::System);

    // Initialize global styles
    AppStyle::setup(Style::dark(), Style::light());
}

// ---- Custom UI styles -------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub is_dark: bool,
    // ---- Main View Visuals -------------------
    pub main_view_fill: Color32,
    pub main_panels_fill: Color32,
    pub main_view_stroke: Stroke,
    // ------------------------------------------
    pub left_bar: LeftBarMenuStyle,
    pub widgets: WidgetStyles,
    pub text_edit: TextEditStyle,
    /// Shadow color (e.g. hover effects)
    pub shadow_fill: Color32,
    /// Color used for accents/highlights
    pub accent_fill: Color32,
    /// A good color for confirmation states
    pub confirmation_fill: Color32,
    // ---- Stack-related states ----------------
    // These fields are relative to the position
    // in the ui traversal stack
    // ------------------------------------------
    pub current_background_fill: Color32,
}

impl Style {
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            main_view_fill: MAIN_VIEW_DARK,
            main_panels_fill: PANEL_DARK,
            main_view_stroke: Stroke::new(1., MAIN_VIEW_STROKE_DARK),
            left_bar: LeftBarMenuStyle::dark(),
            widgets: WidgetStyles::dark(),
            text_edit: TextEditStyle::dark(),
            shadow_fill: SHADOW_STRONG_ON_BACKGROUND_DARK,
            accent_fill: ACCENT_FILL_DARK,
            confirmation_fill: CONFIRMATION_FILL_DARK,
            current_background_fill: BACKGROUND_DARK,
        }
    }

    pub fn light() -> Self {
        Self {
            is_dark: false,
            main_view_fill: MAIN_VIEW_LIGHT,
            main_panels_fill: PANEL_LIGHT,
            main_view_stroke: Stroke::new(1., MAIN_VIEW_STROKE_LIGHT),
            left_bar: LeftBarMenuStyle::light(),
            widgets: WidgetStyles::light(),
            text_edit: TextEditStyle::light(),
            shadow_fill: SHADOW_STRONG_ON_BACKGROUND_LIGHT,
            accent_fill: ACCENT_FILL_LIGHT,
            confirmation_fill: CONFIRMATION_FILL_LIGHT,
            current_background_fill: BACKGROUND_LIGHT,
        }
    }

    pub fn base_font_of(&self, size: f32) -> egui::FontId {
        Figtree::medium().sized(size)
    }

    pub fn bold_font_of(&self, size: f32) -> egui::FontId {
        Figtree::extra_bold().sized(size)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LeftBarMenuStyle {
    pub icon_inactive_color: Color32,
    pub icon_active_color: Color32,
    pub shadow_color_hover: Color32,
    pub shadow_color_active: Color32,
}

impl LeftBarMenuStyle {
    fn dark() -> Self {
        Self {
            icon_inactive_color: ICON_INACTIVE_ON_BACKGROUND_DARK,
            icon_active_color: ICON_ACTIVE_ON_BACKGROUND_DARK,
            shadow_color_hover: SHADOW_LIGHT_ON_BACKGROUND_DARK,
            shadow_color_active: SHADOW_MEDIUM_ON_BACKGROUND_DARK,
        }
    }

    fn light() -> Self {
        Self {
            icon_inactive_color: ICON_INACTIVE_ON_BACKGROUND_LIGHT,
            icon_active_color: ICON_ACTIVE_ON_BACKGROUND_LIGHT,
            shadow_color_hover: SHADOW_LIGHT_ON_BACKGROUND_LIGHT,
            shadow_color_active: SHADOW_MEDIUM_ON_BACKGROUND_LIGHT,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WidgetStyles {
    pub noninteractive: InteractionStyle,
    pub inactive: InteractionStyle,
    pub hovered: InteractionStyle,
    pub active: InteractionStyle,
}

impl WidgetStyles {
    fn dark() -> Self {
        Self {
            noninteractive: InteractionStyle {
                bg_fill: NONINTERACTIVE_WIDGET_BG_FILL_DARK,
                bg_stroke_color: NONINTERACTIVE_WIDGET_BG_STROKE_DARK,
                fg_stroke_color: NONINTERACTIVE_WIDGET_FG_STROKE_DARK,
            },
            inactive: InteractionStyle {
                bg_fill: INACTIVE_WIDGET_BG_FILL_DARK,
                bg_stroke_color: INACTIVE_WIDGET_BG_STROKE_DARK,
                fg_stroke_color: INACTIVE_WIDGET_FG_STROKE_DARK,
            },
            hovered: InteractionStyle {
                bg_fill: HOVERED_WIDGET_BG_FILL_DARK,
                bg_stroke_color: HOVERED_WIDGET_BG_STROKE_DARK,
                fg_stroke_color: HOVERED_WIDGET_FG_STROKE_DARK,
            },
            active: InteractionStyle {
                bg_fill: ACTIVE_WIDGET_BG_FILL_DARK,
                bg_stroke_color: ACTIVE_WIDGET_BG_STROKE_DARK,
                fg_stroke_color: ACTIVE_WIDGET_FG_STROKE_DARK,
            },
        }
    }

    fn light() -> Self {
        Self {
            noninteractive: InteractionStyle {
                bg_fill: NONINTERACTIVE_WIDGET_BG_FILL_LIGHT,
                bg_stroke_color: NONINTERACTIVE_WIDGET_BG_STROKE_LIGHT,
                fg_stroke_color: NONINTERACTIVE_WIDGET_FG_STROKE_LIGHT,
            },
            inactive: InteractionStyle {
                bg_fill: INACTIVE_WIDGET_BG_FILL_LIGHT,
                bg_stroke_color: INACTIVE_WIDGET_BG_STROKE_LIGHT,
                fg_stroke_color: INACTIVE_WIDGET_FG_STROKE_LIGHT,
            },
            hovered: InteractionStyle {
                bg_fill: HOVERED_WIDGET_BG_FILL_LIGHT,
                bg_stroke_color: HOVERED_WIDGET_BG_STROKE_LIGHT,
                fg_stroke_color: HOVERED_WIDGET_FG_STROKE_LIGHT,
            },
            active: InteractionStyle {
                bg_fill: ACTIVE_WIDGET_BG_FILL_LIGHT,
                bg_stroke_color: ACTIVE_WIDGET_BG_STROKE_LIGHT,
                fg_stroke_color: ACTIVE_WIDGET_FG_STROKE_LIGHT,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InteractionStyle {
    pub bg_fill: Color32,
    pub bg_stroke_color: Color32,
    pub fg_stroke_color: Color32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextEditStyle {
    pub inactive_fill: Color32,
    pub hover_fill: Color32,
    pub active_fill: Color32,
}

impl TextEditStyle {
    fn dark() -> Self {
        Self {
            inactive_fill: TEXT_EDIT_INACTIVE_FILL_DARK,
            hover_fill: TEXT_EDIT_HOVER_FILL_DARK,
            active_fill: TEXT_EDIT_ACTIVE_FILL_DARK,
        }
    }

    fn light() -> Self {
        Self {
            inactive_fill: TEXT_EDIT_INACTIVE_FILL_LIGHT,
            hover_fill: TEXT_EDIT_HOVER_FILL_LIGHT,
            active_fill: TEXT_EDIT_ACTIVE_FILL_LIGHT,
        }
    }
}

// ---- Custom override functions for panel levels -----------------------------

pub mod presets {
    use super::*;

    pub fn popup_style(style: &mut Style) {
        if style.is_dark {
            style.widgets.noninteractive.bg_stroke_color = POPUP_STROKE_DARK;
        } else {
            style.widgets.noninteractive.bg_stroke_color = POPUP_STROKE_LIGHT;
        }
    }
}

// ---- Override default egui styles -------------------------------------------

/// Override default egui styles to match SEGS UI design.
fn override_egui_styles(style: &mut egui::Style) {
    // Animations
    style.animation_time = 0.1;

    // Text
    style.visuals.weak_text_alpha = 0.5;

    // Widget styles
    let inactive = &mut style.visuals.widgets.inactive;
    let active = &mut style.visuals.widgets.active;

    inactive.fg_stroke.color = active.fg_stroke.color;
    inactive.fg_stroke.width = 1.5;
}

/// Override dark theme styles.
fn override_dark_style(style: &mut egui::Style) {
    // General visuals
    style.visuals.panel_fill = BACKGROUND_DARK;

    // Customizing popup frames
    style.visuals.window_stroke = Stroke::new(1., POPUP_STROKE_DARK);
    style.visuals.window_fill = FOREGROUND_DARK;
    style.visuals.popup_shadow = Shadow {
        offset: [1, 2],
        blur: 3,
        spread: 0,
        color: POPUP_SHADOW_DARK,
    };

    // Override text color to improve contrast in dark mode
    style.visuals.widgets.noninteractive.fg_stroke.color = TEXT_DARK;
}

/// Override light theme styles.
fn override_light_style(style: &mut egui::Style) {
    // General visuals
    style.visuals.panel_fill = BACKGROUND_LIGHT;

    // Customizing popup frames
    style.visuals.window_stroke = Stroke::new(1., POPUP_STROKE_LIGHT);
    style.visuals.window_fill = FOREGROUND_LIGHT;
    style.visuals.popup_shadow = Shadow {
        offset: [1, 2],
        blur: 3,
        spread: 0,
        color: POPUP_SHADOW_LIGHT,
    };

    // Override text color to improve contrast in dark mode
    style.visuals.widgets.noninteractive.fg_stroke.color = TEXT_LIGHT;
}
