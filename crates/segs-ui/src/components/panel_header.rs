use egui::{Align, Frame, Layout, Response, RichText, Ui, Widget};

use crate::widgets::Separator;

pub struct PanelHeader {
    title: String,
    subtitle: Option<String>,
}

impl PanelHeader {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
}

impl Widget for PanelHeader {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { title, subtitle } = self;

        let layout = Layout::top_down(Align::Min);
        let frame = Frame::new().inner_margin(ui.spacing().window_margin);

        let res = frame
            .show(ui, |ui| {
                ui.with_layout(layout, |ui| {
                    ui.label(RichText::new(title));
                    if let Some(subtitle) = subtitle {
                        ui.label(RichText::new(subtitle).size(10.));
                    }
                })
                .response
            })
            .response;
        ui.add(Separator::default().spacing(0.));

        res
    }
}
