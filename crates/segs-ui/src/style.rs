use std::{
    ops::Deref,
    sync::{Arc, OnceLock},
};

use arc_swap::{ArcSwap, Guard};
use egui::{Color32, Shadow, Stroke, Style, Theme, ThemePreference, Visuals, style::WidgetVisuals};
use segs_assets::{Font, fonts::Figtree};

pub static DARK: OnceLock<Arc<AppStyle>> = OnceLock::new();
pub static LIGHT: OnceLock<Arc<AppStyle>> = OnceLock::new();

/// Panel fill, color of the background.
const LEVEL_0_COLOR_DARK: Color32 = Color32::from_rgb(9, 9, 9);
const LEVEL_0_COLOR_LIGHT: Color32 = Color32::from_rgb(240, 240, 240);
/// Color or collapsed panels, in the middle.
const LEVEL_1_COLOR_DARK: Color32 = Color32::from_rgb(16, 16, 17);
const LEVEL_1_COLOR_LIGHT: Color32 = Color32::from_rgb(249, 249, 249);
/// Color of the main view, the layer above panels.
const LEVEL_2_COLOR_DARK: Color32 = Color32::from_rgb(18, 18, 19);
const LEVEL_2_COLOR_LIGHT: Color32 = Color32::from_rgb(252, 252, 252);
/// Popup fill, color of the uppermost layer.
const LEVEL_3_COLOR_DARK: Color32 = Color32::from_rgb(28, 29, 31);
const LEVEL_3_COLOR_LIGHT: Color32 = Color32::WHITE;

#[derive(Debug)]
pub struct AppStyle {
    /// Egui orginal style struct
    egui: EguiStyleRef,

    /// Custom visual definitions
    pub visuals: AppVisuals,
}

impl AppStyle {
    pub fn dark(egui: Arc<Style>) -> Self {
        let egui = EguiStyleRef::new(egui);
        Self {
            visuals: AppVisuals::dark(egui.clone()),
            egui,
        }
    }

    pub fn light(egui: Arc<Style>) -> Self {
        let egui = EguiStyleRef::new(egui);
        Self {
            visuals: AppVisuals::light(egui.clone()),
            egui,
        }
    }

    pub(crate) fn set_egui_style(&self, style: Arc<Style>) {
        self.egui.set(style);
    }

    /// Get egui style reference.
    pub fn egui(&self) -> Arc<Style> {
        self.egui.get().clone()
    }

    pub fn interact_active(&self, active_flag: bool) -> &AppWidgetVisuals {
        if active_flag {
            &self.visuals.widget_style.active
        } else {
            &self.visuals.widget_style.inactive
        }
    }

    pub fn base_font_of(&self, size: f32) -> egui::FontId {
        Figtree::medium().sized(size)
    }

    pub fn bold_font_of(&self, size: f32) -> egui::FontId {
        Figtree::extra_bold().sized(size)
    }
}

#[derive(Debug)]
pub struct AppVisuals {
    /// Egui orginal style struct
    egui: EguiStyleRef,

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

    /// Widget style definitions
    pub widget_style: WidgetStyle,

    /// Shadow color (e.g. for hover effects)
    pub shadow_color: Color32,

    /// Color used for accents/highlights
    pub accent_color: Color32,

    /// A good color for enabled states
    pub enabled_color: Color32,
}

impl AppVisuals {
    fn dark(egui: EguiStyleRef) -> Self {
        Self {
            widget_style: WidgetStyle::dark(egui.clone()),
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
            egui,
        }
    }

    fn light(egui: EguiStyleRef) -> Self {
        Self {
            widget_style: WidgetStyle::light(egui.clone()),
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
            egui,
        }
    }

    /// Get egui visuals reference.
    pub fn egui(&self) -> VisualsRef {
        VisualsRef(self.egui.guard())
    }

    /// Get shadow color based on current visual mode.
    /// `a` is a multiplier for the alpha channel.
    pub fn shadow_color_lerp(&self, a: f32) -> Color32 {
        Color32::TRANSPARENT.lerp_to_gamma(self.shadow_color, a)
    }
}

#[derive(Debug)]
pub struct WidgetStyle {
    pub active: AppWidgetVisuals,
    pub hover: AppWidgetVisuals,
    pub inactive: AppWidgetVisuals,
}

impl WidgetStyle {
    fn dark(egui: EguiStyleRef) -> Self {
        Self {
            active: AppWidgetVisuals {
                egui: egui.clone(),
                kind: WidgetVisualsKind::Active,
            },
            hover: AppWidgetVisuals {
                egui: egui.clone(),
                kind: WidgetVisualsKind::Hover,
            },
            inactive: AppWidgetVisuals {
                egui,
                kind: WidgetVisualsKind::Inactive,
            },
        }
    }

    fn light(egui: EguiStyleRef) -> Self {
        Self {
            active: AppWidgetVisuals {
                egui: egui.clone(),
                kind: WidgetVisualsKind::Active,
            },
            hover: AppWidgetVisuals {
                egui: egui.clone(),
                kind: WidgetVisualsKind::Hover,
            },
            inactive: AppWidgetVisuals {
                egui,
                kind: WidgetVisualsKind::Inactive,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppWidgetVisuals {
    /// Egui original style struct
    egui: EguiStyleRef,

    /// Internal ref for retrieving egui widget visuals
    kind: WidgetVisualsKind,
}

impl AppWidgetVisuals {
    /// Get egui widget visuals reference.
    pub fn egui(&self) -> WidgetVisualsRef {
        WidgetVisualsRef {
            guard: self.egui.guard(),
            kind: self.kind,
        }
    }
}

/// Setup egui styles to match SEGS UI design.
pub fn setup_style(ctx: &egui::Context) {
    // Override egui styles
    ctx.all_styles_mut(override_egui_styles);
    ctx.style_mut_of(Theme::Dark, override_dark_style);
    ctx.style_mut_of(Theme::Light, override_light_style);
    ctx.set_theme(ThemePreference::System);

    // Initialize global styles
    DARK.set(Arc::new(AppStyle::dark(ctx.style()))).unwrap();
    LIGHT.set(Arc::new(AppStyle::light(ctx.style()))).unwrap();
}

/// Override default egui styles to match SEGS UI design.
fn override_egui_styles(style: &mut Style) {
    // Animations
    style.animation_time = 0.05;

    // Widget styles
    let inactive = &mut style.visuals.widgets.inactive;
    let active = &mut style.visuals.widgets.active;

    inactive.fg_stroke.color = active.fg_stroke.color;
    inactive.fg_stroke.width = 1.5;
}

/// Override dark theme styles.
fn override_dark_style(style: &mut Style) {
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
fn override_light_style(style: &mut Style) {
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

#[derive(Debug, Clone)]
struct EguiStyleRef(Arc<ArcSwap<Style>>);

impl EguiStyleRef {
    fn new(style: Arc<Style>) -> Self {
        Self(Arc::new(ArcSwap::from(style)))
    }

    fn get(&self) -> Arc<Style> {
        self.0.load_full()
    }

    fn guard(&self) -> Guard<Arc<Style>> {
        self.0.load()
    }

    fn set(&self, style: Arc<Style>) {
        self.0.store(style);
    }
}

pub struct VisualsRef(Guard<Arc<Style>>);

impl Deref for VisualsRef {
    type Target = Visuals;
    fn deref(&self) -> &Visuals {
        &self.0.visuals
    }
}

pub struct WidgetVisualsRef {
    guard: Guard<Arc<Style>>,
    kind: WidgetVisualsKind,
}

#[derive(Debug, Clone, Copy)]
enum WidgetVisualsKind {
    Active,
    Hover,
    Inactive,
}

impl Deref for WidgetVisualsRef {
    type Target = WidgetVisuals;
    fn deref(&self) -> &WidgetVisuals {
        match self.kind {
            WidgetVisualsKind::Active => &self.guard.visuals.widgets.active,
            WidgetVisualsKind::Hover => &self.guard.visuals.widgets.hovered,
            WidgetVisualsKind::Inactive => &self.guard.visuals.widgets.inactive,
        }
    }
}
