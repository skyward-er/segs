use std::sync::{Arc, LazyLock};

use egui::{Color32, ThemePreference};
use segs_assets::{Font, fonts::Figtree};

pub static DARK: LazyLock<Arc<Style>> = LazyLock::new(|| Arc::new(Style::dark()));
pub static LIGHT: LazyLock<Arc<Style>> = LazyLock::new(|| Arc::new(Style::light()));

const DARK_SHADOW_ALPHA: u8 = 40;
const LIGHT_SHADOW_ALPHA: u8 = 20;

#[derive(Clone)]
pub struct Style {
    pub visuals: Arc<Visuals>,
}

impl Style {
    pub fn dark() -> Self {
        Style {
            visuals: Arc::new(Visuals::dark()),
        }
    }

    pub fn light() -> Self {
        Style {
            visuals: Arc::new(Visuals::light()),
        }
    }

    pub fn base_font_of(&self, size: f32) -> egui::FontId {
        Figtree::medium().sized(size)
    }

    pub fn bold_font_of(&self, size: f32) -> egui::FontId {
        Figtree::extra_bold().sized(size)
    }
}

pub struct Visuals {
    pub dark_mode: bool,
    pub text_color: Color32,
    pub shadow_color: Color32,
    pub icon_color: Color32,
    pub accent_color: Color32,
    pub enabled_color: Color32,
}

impl Visuals {
    pub fn dark() -> Self {
        Self {
            dark_mode: true,
            text_color: Color32::WHITE.lerp_to_gamma(Color32::BLACK, 0.2),
            shadow_color: Color32::from_white_alpha(DARK_SHADOW_ALPHA),
            icon_color: Color32::WHITE,
            accent_color: Color32::from_hex("#0084ff").unwrap(),
            enabled_color: Color32::from_hex("#179657").unwrap(),
        }
    }

    pub fn light() -> Self {
        Self {
            dark_mode: false,
            text_color: Color32::BLACK.lerp_to_gamma(Color32::WHITE, 0.1),
            shadow_color: Color32::from_black_alpha(LIGHT_SHADOW_ALPHA),
            icon_color: Color32::BLACK,
            accent_color: Color32::from_hex("#e89d56").unwrap(),
            enabled_color: Color32::from_hex("#58e8a0").unwrap(),
        }
    }

    /// Get shadow color based on current visual mode.
    /// `a` is a multiplier for the alpha channel.
    pub fn shadow_color_lerp(&self, a: f32) -> Color32 {
        if self.dark_mode {
            // White shadow for dark mode
            Color32::from_white_alpha((a * DARK_SHADOW_ALPHA as f32) as u8)
        } else {
            // Dark shadow for light mode
            Color32::from_black_alpha((a * LIGHT_SHADOW_ALPHA as f32) as u8)
        }
    }
}

/// Setup egui styles to match SEGS UI design.
pub fn setup_style(ctx: &egui::Context) {
    ctx.all_styles_mut(override_egui_styles);
    ctx.set_theme(ThemePreference::System);
}

/// Override default egui styles to match SEGS UI design.
fn override_egui_styles(style: &mut egui::Style) {
    // Animations
    style.animation_time = 0.05;

    // Widget styles
    let inactive = &mut style.visuals.widgets.inactive;
    let active = &mut style.visuals.widgets.active;

    inactive.fg_stroke.width = 1.5;
    inactive.fg_stroke.color = active.fg_stroke.color;
}
