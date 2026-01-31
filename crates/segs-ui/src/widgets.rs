use egui::Response;
use segs_assets::icons::Icon;

pub mod buttons;

pub trait UiWidgetExt {
    fn checkbox(&mut self, active: &mut bool) -> Response;
    fn toggle(&mut self, active: &mut bool) -> Response;
    fn icon_btn(&mut self, icon: impl Icon + 'static) -> Response;
    fn icon_toggle(
        &mut self,
        inactive_icon: impl Icon + 'static,
        active_icon: impl Icon + 'static,
        active: &mut bool,
    ) -> Response;
}

impl UiWidgetExt for egui::Ui {
    fn checkbox(&mut self, active: &mut bool) -> Response {
        buttons::checkbox(self, active)
    }

    fn toggle(&mut self, active: &mut bool) -> Response {
        buttons::toggle(self, active)
    }

    fn icon_btn(&mut self, icon: impl Icon + 'static) -> Response {
        buttons::icon_btn(self, icon)
    }

    fn icon_toggle(
        &mut self,
        inactive_icon: impl Icon + 'static,
        active_icon: impl Icon + 'static,
        active: &mut bool,
    ) -> Response {
        buttons::icon_toggle(self, inactive_icon, active_icon, active)
    }
}
