use egui::{Color32, Frame, Margin, Response, RichText, Stroke, Ui, Widget, WidgetText};

use crate::style::CtxStyleExt;

/// A compact label used to communicate a short state.
pub struct Badge {
    text: WidgetText,
    fill: Option<Color32>,
    text_color: Option<Color32>,
}

impl Badge {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            fill: None,
            text_color: None,
        }
    }

    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn text_color(mut self, text_color: Color32) -> Self {
        self.text_color = Some(text_color);
        self
    }
}

impl Widget for Badge {
    fn ui(self, ui: &mut Ui) -> Response {
        let app_style = ui.app_style();
        let fill = self.fill.unwrap_or(app_style.widgets.noninteractive.bg_fill);
        let text_color = self.text_color.unwrap_or_else(|| ui.visuals().text_color());

        Frame::new()
            .fill(fill)
            .stroke(Stroke::NONE)
            .corner_radius(3)
            .inner_margin(Margin::symmetric(6, 2))
            .show(ui, |ui| {
                ui.label(RichText::new(self.text.text()).size(10.).color(text_color))
            })
            .response
    }
}
