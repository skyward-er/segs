use egui::{Id, Response, RichText, Ui, Widget, WidgetText};

use crate::{style::CtxStyleExt, widgets::text::TextEdit};

/// A single-line text input with an optional inline validation error.
pub struct ValidationTextEdit<'a> {
    text: &'a mut String,
    error: Option<WidgetText>,
    hint_text: WidgetText,
    id_salt: Option<Id>,
    desired_width: Option<f32>,
}

impl<'a> ValidationTextEdit<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            error: None,
            hint_text: WidgetText::default(),
            id_salt: None,
            desired_width: None,
        }
    }

    pub fn error(mut self, error: impl Into<WidgetText>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn hint_text(mut self, hint_text: impl Into<WidgetText>) -> Self {
        self.hint_text = hint_text.into();
        self
    }

    pub fn id_salt(mut self, id_salt: impl std::hash::Hash) -> Self {
        self.id_salt = Some(Id::new(id_salt));
        self
    }

    pub fn desired_width(mut self, desired_width: f32) -> Self {
        self.desired_width = Some(desired_width);
        self
    }
}

impl Widget for ValidationTextEdit<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            text,
            error,
            hint_text,
            id_salt,
            desired_width,
        } = self;
        let invalid = error.is_some();
        let mut edit = TextEdit::singleline(text).hint_text(hint_text);
        if let Some(id_salt) = id_salt {
            edit = edit.id_salt(id_salt);
        }
        if let Some(desired_width) = desired_width {
            edit = edit.desired_width(desired_width);
        }
        if invalid {
            edit = edit.background_fill(ui.app_style().text_edit.invalid_fill);
        }

        ui.vertical(|ui| {
            let response = ui.add(edit);
            if let Some(error) = error {
                ui.label(
                    RichText::new(error.text())
                        .size(10.)
                        .color(ui.app_style().error_fg_color),
                );
            }
            response
        })
        .inner
    }
}
