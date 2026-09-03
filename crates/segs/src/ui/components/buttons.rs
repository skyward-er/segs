#![allow(dead_code)]

use egui::{Response, Ui};
use segs_assets::icons;
use segs_ui::widgets::UiWidgetExt;

pub fn lock_mode_toggle(ui: &mut Ui, active: &mut bool) -> Response {
    ui.icon_toggle(icons::Lock::unlocked(), icons::Lock::locked(), active)
}

pub fn left_panel_toggle(ui: &mut Ui, active: &mut bool) -> Response {
    ui.icon_toggle(
        icons::PanelToggle::left_panel(),
        icons::PanelToggle::left_panel().solid(),
        active,
    )
}

pub fn right_panel_toggle(ui: &mut Ui, active: &mut bool) -> Response {
    ui.icon_toggle(
        icons::PanelToggle::right_panel(),
        icons::PanelToggle::right_panel().solid(),
        active,
    )
}

pub fn bottom_panel_toggle(ui: &mut Ui, active: &mut bool) -> Response {
    ui.icon_toggle(
        icons::PanelToggle::bottom_panel(),
        icons::PanelToggle::bottom_panel().solid(),
        active,
    )
}
