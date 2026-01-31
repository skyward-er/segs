use egui::Response;

pub mod buttons;

pub trait UiWidgetExt {
    fn checkbox(&mut self, active: &mut bool) -> Response;
    fn toggle(&mut self, active: &mut bool) -> Response;
}

impl UiWidgetExt for egui::Ui {
    fn checkbox(&mut self, active: &mut bool) -> Response {
        self.add(buttons::Checkbox::new(active))
    }

    fn toggle(&mut self, active: &mut bool) -> Response {
        self.add(buttons::Toggle::new(active))
    }
}
