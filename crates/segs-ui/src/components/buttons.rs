use egui::{Response, Theme, Ui, Widget, vec2};
use segs_assets::icons::{self, Icon};

use crate::widgets::{UiWidgetExt, buttons::RibbonToggle};

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

pub struct LeftBarMenuButton<'a, V: PartialEq> {
    selector: &'a mut Option<V>,
    selected_variant: V,
    inactive_icon: Box<dyn Icon>,
    active_icon: Box<dyn Icon>,
}

impl<'a, V: PartialEq> LeftBarMenuButton<'a, V> {
    pub fn new(
        selector: &'a mut Option<V>,
        selected_variant: V,
        inactive_icon: impl Icon + 'static,
        active_icon: impl Icon + 'static,
    ) -> Self {
        Self {
            selector,
            selected_variant,
            inactive_icon: Box::new(inactive_icon),
            active_icon: Box::new(active_icon),
        }
    }
}

impl<'a, V: PartialEq> Widget for LeftBarMenuButton<'a, V> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            selector,
            selected_variant,
            inactive_icon,
            active_icon,
        } = self;

        let mut toggled = selector.as_ref().is_some_and(|v| *v == selected_variant);
        let widget = RibbonToggle::new(inactive_icon, active_icon, &mut toggled)
            .icon_size(vec2(20., 20.))
            .shadow_size(vec2(34., 26.));
        let response = ui.add(widget);

        // Toggle logic: if the button is clicked, set the selector to the selected
        // variant if it was previously not selected, or to None if it was already
        // selected.
        if toggled && response.clicked() {
            *selector = Some(selected_variant);
        } else if !toggled && response.clicked() {
            *selector = None;
        }
        response
    }
}
