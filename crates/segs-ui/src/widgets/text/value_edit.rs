use std::{fmt::Display, str::FromStr};

use egui::{Align, Id, Response, Ui, UiBuilder, Widget, vec2};
use segs_memory::MemoryExt;

use crate::widgets::text::TextEdit;

pub struct ValueEdit<'a, V: FromStr + Display> {
    text: &'a mut V,
    id: Option<Id>,
    text_hint: String,
    horizontal_align: Align,
    desired_width: Option<f32>,
    update_while_editing: bool,
    char_limit: Option<usize>,
}

impl<'a, V: FromStr + Display> ValueEdit<'a, V> {
    pub fn new(text: &'a mut V) -> Self {
        Self {
            text,
            id: None,
            text_hint: String::new(),
            horizontal_align: Align::LEFT,
            desired_width: None,
            update_while_editing: false,
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
        self.desired_width = Some(width);
        self
    }

    pub fn update_while_editing(mut self, update: bool) -> Self {
        self.update_while_editing = update;
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
            desired_width,
            update_while_editing,
            ..
        } = self;

        let edit_id = ui.id().with("text_edit");
        let buffer_id = ui.id().with("text_edit_buffer");

        // Text edit with the buffer string. The buffer is used to allow editing the
        // text without immediately updating the value, which allows for better handling
        // of parsing and validation.
        let mut buffer_text: String = ui.mem().get_temp_or_insert(buffer_id, text.to_string());
        let text_edit = {
            let mut text_edit = TextEdit::singleline(&mut buffer_text)
                .id(edit_id)
                .hint_text(text_hint)
                .horizontal_align(horizontal_align);

            if let Some(limit) = self.char_limit {
                text_edit = text_edit.char_limit(limit);
            }

            text_edit
        };

        // Check if the edit content has changed or if the edit has lost focus.
        let response = if let Some(width) = desired_width {
            ui.add_sized(vec2(width, 0.), text_edit)
        } else {
            ui.add(text_edit)
        };
        let update = if update_while_editing {
            // Update when the edit content has changed.
            response.changed()
        } else {
            // Update only when the edit has lost focus.
            response.lost_focus()
        };

        if update {
            // Parse the buffer text and update the value if parsing is successful.
            if let Ok(parsed_value) = buffer_text.parse::<V>() {
                *text = parsed_value;
            }
            // Format the current value back to the buffer text to reflect any formatting
            // changes (e.g. removing leading zeros).
            buffer_text = text.to_string();
        }

        // Save back the buffer text to memory for the next frame
        ui.mem().insert_temp(buffer_id, buffer_text);

        response
    }
}

impl<V: FromStr + Display> Widget for ValueEdit<'_, V> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}
