use egui::{Color32, Response, Sense, Stroke, Ui, Vec2, Widget};

use crate::{error::ErrInstrument, msg_broker};

pub struct ReceptionLed {
    pub active: bool,
}

impl ReceptionLed {
    /// Create a new `ReceptionLed` widget based on the given state.
    pub fn new(active: bool) -> Self {
        Self { active }
    }
}

impl ReceptionLed {
    fn show_led(&self, ui: &mut Ui) -> Response {
        // Allocate an exact size for the widget
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(9.0), Sense::click());
        // Get the visuals for the UI (to display the widget with coherent style)
        // in this case we use the visuals for inactive widgets, since this is a passive component
        let visuals = ui.style().visuals.widgets.noninteractive;
        let inactive_bg = Color32::TRANSPARENT;
        let active_bg = Color32::from_hex("#03C04A").log_unwrap();

        // Determine colors based on state
        let fill_color = if self.active { active_bg } else { inactive_bg };
        let stroke = Stroke::new(1.0, visuals.fg_stroke.color);

        // Use the painter to draw a rectangle
        if ui.is_rect_visible(rect) {
            ui.painter()
                .circle(rect.center(), rect.width() / 2.0, fill_color, stroke);
        }

        response
    }

    fn show_label(&self, ui: &mut Ui) -> Response {
        if self.active {
            let freq = msg_broker!().reception_frequency();
            let text = format!("{} Hz", freq);
            ui.label(text)
        } else {
            ui.label("N/A")
        }
    }
}

impl Widget for ReceptionLed {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.label("Receiving at:");
            self.show_led(ui);
            self.show_label(ui);
        })
        .response
    }
}
