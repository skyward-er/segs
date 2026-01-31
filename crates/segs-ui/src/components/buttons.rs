use egui::{Response, Theme, Ui};
use segs_assets::icons;

use crate::widgets::buttons::IconToggle;

pub fn lock_mode_toggle(ui: &mut Ui, active: &mut bool) -> Response {
    let toggle = IconToggle::active(icons::Lock::unlocked(), icons::Lock::locked(), active);
    ui.add(toggle)
}

pub fn focus_mode_toggle(ui: &mut Ui, active: &mut bool) -> Response {
    let toggle = IconToggle::active(icons::Reticle::empty(), icons::Reticle::solid(), active);
    ui.add(toggle)
}

pub fn theme_toggle(ui: &mut Ui) {
    if ui.visuals().dark_mode {
        let light_theme_toggle = IconToggle::new(icons::Sun::outline());
        if ui.add(light_theme_toggle).clicked() {
            ui.ctx().set_theme(Theme::Light);
            ui.ctx().request_discard("theme change");
        }
    } else {
        let dark_theme_toggle = IconToggle::new(icons::Moon::outline());
        if ui.add(dark_theme_toggle).clicked() {
            ui.ctx().set_theme(Theme::Dark);
            ui.ctx().request_discard("theme change");
        }
    }
}

pub fn left_panel_toggle(ui: &mut Ui, active: &mut bool) -> Response {
    let toggle = IconToggle::active(
        icons::PanelToggle::left_panel(),
        icons::PanelToggle::left_panel().solid(),
        active,
    );
    ui.add(toggle)
}

pub fn right_panel_toggle(ui: &mut Ui, active: &mut bool) -> Response {
    let toggle = IconToggle::active(
        icons::PanelToggle::right_panel(),
        icons::PanelToggle::right_panel().solid(),
        active,
    );
    ui.add(toggle)
}

pub fn bottom_panel_toggle(ui: &mut Ui, active: &mut bool) -> Response {
    let toggle = IconToggle::active(
        icons::PanelToggle::bottom_panel(),
        icons::PanelToggle::bottom_panel().solid(),
        active,
    );
    ui.add(toggle)
}
