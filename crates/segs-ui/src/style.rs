mod colors;
mod stack;

use egui::{Color32, Shadow, Stroke, Theme, ThemePreference};
use segs_assets::{Font, fonts::Figtree};
pub use stack::{AppStyle, CtxStyleExt, UiStyleExt};

use colors::*;

/// Setup egui styles to match SEGS UI design.
pub fn setup_style(ctx: &egui::Context) {
    // Override egui styles
    ctx.all_styles_mut(override_egui_styles);
    ctx.style_mut_of(Theme::Dark, override_dark_style);
    ctx.style_mut_of(Theme::Light, override_light_style);
    // ctx.set_theme(ThemePreference::System); // FIXME
    ctx.set_theme(ThemePreference::Dark);

    // Initialize global styles
    AppStyle::setup(Style::dark(), Style::light());
}

// ---- Custom UI styles -------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    // ~ Main View Visuals ~
    pub main_view_fill: Color32,
    pub main_panels_fill: Color32,
    pub main_view_stroke: Stroke,

    // These two controls how menu icons are tinted in active/inactive states.
    pub menu_icon_inactive_color: Color32,
    pub menu_icon_active_color: Color32,
    pub menu_icon_shadow_color_hover: Color32,
    pub menu_icon_shadow_color_active: Color32,

    pub button_hover_shadow_color: Color32,

    /// Shadow color (e.g. for hover effects)
    pub shadow_color: Color32,

    /// Color used for accents/highlights
    pub accent_color: Color32,

    /// A good color for enabled states
    pub enabled_color: Color32,
}

impl Style {
    pub fn dark() -> Self {
        Self {
            main_view_fill: LEVEL_2_COLOR_DARK,
            main_panels_fill: LEVEL_1_COLOR_DARK,
            main_view_stroke: Stroke::new(1., Color32::from_rgb(39, 40, 45)),
            menu_icon_inactive_color: Color32::from_rgb(149, 149, 151),
            menu_icon_active_color: Color32::WHITE,
            menu_icon_shadow_color_hover: Color32::from_rgb(21, 22, 25),
            menu_icon_shadow_color_active: Color32::from_rgb(30, 31, 34),
            button_hover_shadow_color: Color32::from_rgb(42, 43, 48),
            shadow_color: Color32::from_white_alpha(40),
            accent_color: Color32::from_rgb(0, 132, 255),
            enabled_color: Color32::from_rgb(23, 150, 87),
        }
    }

    pub fn light() -> Self {
        Self {
            main_view_fill: LEVEL_2_COLOR_LIGHT,
            main_panels_fill: LEVEL_1_COLOR_LIGHT,
            main_view_stroke: Stroke::new(1., Color32::from_rgb(216, 216, 216)),
            menu_icon_inactive_color: Color32::from_rgb(89, 90, 91),
            menu_icon_active_color: Color32::from_rgb(26, 26, 26),
            menu_icon_shadow_color_hover: Color32::from_rgb(232, 232, 232),
            menu_icon_shadow_color_active: Color32::from_rgb(226, 226, 226),
            button_hover_shadow_color: Color32::from_rgb(232, 232, 232),
            shadow_color: Color32::from_black_alpha(20),
            accent_color: Color32::from_rgb(232, 157, 86),
            enabled_color: Color32::from_rgb(88, 232, 160),
        }
    }

    pub fn base_font_of(&self, size: f32) -> egui::FontId {
        Figtree::medium().sized(size)
    }

    pub fn bold_font_of(&self, size: f32) -> egui::FontId {
        Figtree::extra_bold().sized(size)
    }

    /// Get shadow color based on current visual mode.
    /// `a` is a multiplier for the alpha channel.
    pub fn shadow_color_lerp(&self, a: f32) -> Color32 {
        Color32::TRANSPARENT.lerp_to_gamma(self.shadow_color, a)
    }
}

// ---- Override default egui styles -------------------------------------------

/// Override default egui styles to match SEGS UI design.
fn override_egui_styles(style: &mut egui::Style) {
    // Animations
    style.animation_time = 0.05;

    // Widget styles
    let inactive = &mut style.visuals.widgets.inactive;
    let active = &mut style.visuals.widgets.active;

    inactive.fg_stroke.color = active.fg_stroke.color;
    inactive.fg_stroke.width = 1.5;
}

/// Override dark theme styles.
fn override_dark_style(style: &mut egui::Style) {
    // General visuals
    style.visuals.panel_fill = LEVEL_0_COLOR_DARK;

    // Customizing popup frames
    style.visuals.window_stroke = Stroke::new(1., Color32::from_rgb(57, 59, 66));
    // This is the color of the Separator widget
    style.visuals.widgets.noninteractive.bg_stroke.color = Color32::from_rgb(57, 59, 66);
    style.visuals.window_fill = LEVEL_3_COLOR_DARK;
    style.visuals.popup_shadow = Shadow {
        offset: [1, 2],
        blur: 3,
        spread: 0,
        color: Color32::from_rgb(21, 22, 25),
    };

    // Override text color to improve contrast in dark mode
    style.visuals.override_text_color = Some(Color32::from_rgb(227, 228, 229));
}

/// Override light theme styles.
fn override_light_style(style: &mut egui::Style) {
    // General visuals
    style.visuals.panel_fill = LEVEL_0_COLOR_LIGHT;

    // Customizing popup frames
    style.visuals.window_stroke = Stroke::new(1., Color32::from_rgb(216, 216, 216));
    style.visuals.window_fill = LEVEL_3_COLOR_LIGHT;
    style.visuals.popup_shadow = Shadow {
        offset: [1, 2],
        blur: 3,
        spread: 0,
        color: Color32::from_rgb(232, 232, 232),
    };

    // Widget styles
    let active = &mut style.visuals.widgets.active;
    let inactive = &mut style.visuals.widgets.inactive;
    let hover = &mut style.visuals.widgets.hovered;

    active.fg_stroke.color = Color32::from_rgb(27, 27, 27);
    active.bg_fill = Color32::from_rgb(237, 237, 237);

    inactive.bg_stroke.color = Color32::from_rgb(216, 216, 216);
    inactive.bg_stroke.width = 0.5;

    inactive.fg_stroke.color = Color32::from_rgb(92, 92, 92);

    hover.bg_fill = active.bg_fill;
}
