use egui::{Response, RichText, Ui, Widget, WidgetText};

/// A subdued label used to divide content within a larger titled section.
pub struct SectionHeader {
    text: WidgetText,
}

impl SectionHeader {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self { text: text.into() }
    }
}

impl Widget for SectionHeader {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.label(RichText::new(self.text.text()).size(10.).weak())
    }
}
