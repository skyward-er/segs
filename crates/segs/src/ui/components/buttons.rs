use egui::{Response, Theme, Ui};
use segs_assets::icons;
use segs_ui::widgets::UiWidgetExt;

pub fn lock_mode_toggle(ui: &mut Ui, active: &mut bool) -> Response {
    ui.icon_toggle(icons::Lock::unlocked(), icons::Lock::locked(), active)
}

pub fn theme_toggle(ui: &mut Ui) {
    if ui.visuals().dark_mode {
        if ui.icon_btn(icons::Sun::outline()).clicked() {
            ui.ctx().set_theme(Theme::Light);
            ui.ctx().request_discard("theme change");
        }
    } else if ui.icon_btn(icons::Moon::outline()).clicked() {
        ui.ctx().set_theme(Theme::Dark);
        ui.ctx().request_discard("theme change");
    }
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
