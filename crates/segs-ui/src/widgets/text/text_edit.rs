use egui::{Align, Color32, Frame, Id, Response, RichText, Ui, UiBuilder};

pub struct TextEdit<'a> {
    text: &'a mut String,
    id: Option<Id>,
    text_hint: String,
    horizontal_align: Align,
    width: Option<f32>,
    char_limit: Option<usize>,
}

impl<'a> TextEdit<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            id: None,
            text_hint: String::new(),
            horizontal_align: Align::LEFT,
            width: None,
            char_limit: None,
        }
    }

    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn hint_text(mut self, hint: impl Into<String>) -> Self {
        self.text_hint = hint.into();
        self
    }

    pub fn horizontal_align(mut self, align: Align) -> Self {
        self.horizontal_align = align;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = Some(limit);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let builder = UiBuilder::new();
        let builder = if let Some(id) = self.id {
            builder.id(id)
        } else {
            builder.id(ui.next_auto_id())
        };
        ui.scope_builder(builder, |ui| self.show_inner(ui)).inner
    }

    fn show_inner(self, ui: &mut Ui) -> Response {
        let Self {
            text,
            text_hint,
            horizontal_align,
            width,
            ..
        } = self;

        Frame::new()
            .fill(Color32::BLACK.gamma_multiply(0.4))
            .corner_radius(3)
            .inner_margin(1)
            .show(ui, |ui| {
                let mut edit = egui::TextEdit::singleline(text)
                    .id(ui.id())
                    .frame(false)
                    .horizontal_align(horizontal_align)
                    .hint_text(RichText::new(text_hint).color(ui.visuals().text_color().gamma_multiply(0.5)));

                if let Some(width) = width {
                    ui.set_width(width);
                    edit = edit.desired_width(width);
                }

                if let Some(limit) = self.char_limit {
                    edit = edit.char_limit(limit);
                }

                ui.add(edit)
            })
            .inner
    }
}
