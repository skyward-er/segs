use egui::{ImageSource, Rect, Theme, Ui};

#[derive(Debug, Clone, Copy)]
pub enum Icon {
    Aperture,
    Timing,
}

impl Icon {
    fn get_image(&self, theme: Theme) -> ImageSource {
        match (&self, theme) {
            (Icon::Aperture, Theme::Light) => {
                egui::include_image!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/icons/valve_control/light/aperture.svg"
                ))
            }
            (Icon::Aperture, Theme::Dark) => {
                egui::include_image!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/icons/valve_control/dark/aperture.svg"
                ))
            }
            (Icon::Timing, Theme::Light) => {
                egui::include_image!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/icons/valve_control/light/timing.svg"
                ))
            }
            (Icon::Timing, Theme::Dark) => {
                egui::include_image!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/icons/valve_control/dark/timing.svg"
                ))
            }
        }
    }
}

impl Icon {
    pub fn paint(&mut self, ui: &mut Ui, image_rect: Rect) {
        let theme = ui.ctx().theme();
        egui::Image::new(self.get_image(theme)).paint_at(ui, image_rect);
    }
}
