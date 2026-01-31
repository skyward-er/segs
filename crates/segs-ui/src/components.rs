use egui::{Response, Ui};

mod buttons;

pub trait UiComponentExt {
    fn theme_toggle_btn(&mut self);
    fn lock_mode_toggle(&mut self, active: &mut bool) -> Response;
    fn focus_mode_toggle(&mut self, active: &mut bool) -> Response;
    fn left_panel_toggle_btn(&mut self, active: &mut bool) -> Response;
    fn right_panel_toggle_btn(&mut self, active: &mut bool) -> Response;
    fn bottom_panel_toggle_btn(&mut self, active: &mut bool) -> Response;
}

impl UiComponentExt for Ui {
    fn theme_toggle_btn(&mut self) {
        buttons::theme_toggle(self);
    }

    fn lock_mode_toggle(&mut self, active: &mut bool) -> Response {
        buttons::lock_mode_toggle(self, active)
    }

    fn focus_mode_toggle(&mut self, active: &mut bool) -> Response {
        buttons::focus_mode_toggle(self, active)
    }

    fn left_panel_toggle_btn(&mut self, active: &mut bool) -> Response {
        buttons::left_panel_toggle(self, active)
    }

    fn right_panel_toggle_btn(&mut self, active: &mut bool) -> Response {
        buttons::right_panel_toggle(self, active)
    }

    fn bottom_panel_toggle_btn(&mut self, active: &mut bool) -> Response {
        buttons::bottom_panel_toggle(self, active)
    }
}
