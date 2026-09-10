use std::{fmt::Display, ops::RangeInclusive, str::FromStr};

use egui::{Align, Id, Response, Ui, UiBuilder, Widget, vec2};
use segs_memory::MemoryExt;

use crate::{
    style::CtxStyleExt,
    widgets::text::{TextEdit, text_edit::default_singleline_height},
};

pub struct ValueEdit<'a, V: FromStr + Display> {
    text: &'a mut V,
    id: Option<Id>,
    text_hint: String,
    horizontal_align: Align,
    vertical_align: Align,
    desided_width: Option<f32>,
    update_while_editing: bool,
    char_limit: Option<usize>,
    normalize: Option<Box<dyn Fn(V) -> Option<V> + 'a>>,
    // Type erasure keeps Copy and PartialOrd bounds local to range configuration
    range_resolver: Option<Box<dyn Fn(V) -> RangeResolution<V> + 'a>>,
    empty_value: Option<V>,
    frameless: bool,
}

impl<'a, V: FromStr + Display> ValueEdit<'a, V> {
    pub fn new(text: &'a mut V) -> Self {
        Self {
            text,
            id: None,
            text_hint: String::new(),
            horizontal_align: Align::LEFT,
            vertical_align: Align::TOP,
            desided_width: None,
            update_while_editing: false,
            char_limit: None,
            normalize: None,
            range_resolver: None,
            empty_value: None,
            frameless: false,
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

    pub fn vertical_align(mut self, align: Align) -> Self {
        self.vertical_align = align;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.desided_width = Some(width);
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

    /// Configures normalization and validation for parsed values.
    ///
    /// The callback returns the value to store, or `None` to reject the edit.
    /// This returns the configured editor.
    pub fn normalize(mut self, normalize: impl Fn(V) -> Option<V> + 'a) -> Self {
        self.normalize = Some(Box::new(normalize));
        self
    }

    /// Sets the inclusive range applied to parsed values.
    ///
    /// Out-of-range drafts remain visible while the edited value is clamped to
    /// the nearest endpoint. This returns the configured editor. The range must
    /// not be empty.
    pub fn range(mut self, range: RangeInclusive<V>) -> Self
    where
        V: Copy + PartialOrd + 'a,
    {
        debug_assert!(!range.is_empty(), "value edit range must not be empty");
        let minimum = *range.start();
        let maximum = *range.end();
        self.range_resolver = Some(Box::new(move |value| {
            if value < minimum {
                RangeResolution::Clamped(minimum)
            } else if value > maximum {
                RangeResolution::Clamped(maximum)
            } else {
                RangeResolution::Valid(value)
            }
        }));
        self
    }

    /// Sets the value represented by an empty editing buffer.
    ///
    /// The empty buffer remains visible while focused and is formatted back to
    /// this value when focus is lost. This returns the configured editor.
    pub fn empty_value(mut self, empty_value: V) -> Self {
        self.empty_value = Some(empty_value);
        self
    }

    /// Removes the editor's independent background and rounded frame.
    ///
    /// This returns the configured editor while preserving its padding and interaction behavior.
    pub fn frameless(mut self) -> Self {
        self.frameless = true;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        self.show_with_status(ui).response
    }

    /// Adds the editor and reports its transient validation status.
    ///
    /// The returned output contains the interaction response and whether the
    /// visible draft is invalid.
    pub fn show_with_status(self, ui: &mut Ui) -> ValueEditOutput {
        let builder = UiBuilder::new();
        let builder = if let Some(id) = self.id {
            builder.id(id)
        } else {
            builder.id(ui.next_auto_id())
        };
        ui.scope_builder(builder, |ui| self.show_inner(ui)).inner
    }

    fn show_inner(self, ui: &mut Ui) -> ValueEditOutput {
        let Self {
            text,
            id: _,
            text_hint,
            horizontal_align,
            vertical_align,
            desided_width,
            update_while_editing,
            char_limit,
            normalize,
            range_resolver: resolve_range,
            empty_value,
            frameless,
        } = self;

        let edit_id = ui.id().with("text_edit");
        let state_id = ui.id().with("value_edit_state");
        let frame = ui.ctx().cumulative_frame_nr();
        let model_text = text.to_string();

        // Reload canonical state after external changes or a gap in rendering
        let has_focus = ui.memory(|memory| memory.has_focus(edit_id));
        let mut state = ui
            .mem()
            .get_temp_or_insert_with(state_id, || ValueEditState::new(model_text.clone(), frame));
        let was_hidden = frame.saturating_sub(state.last_seen_frame) > 1;
        if was_hidden || state.model_text != model_text {
            state.reset(model_text.clone());
        } else if !has_focus && !state.invalid {
            state.buffer = model_text;
        }

        // Edit a draft buffer independently from the live model value
        let mut buffer_text = state.buffer;
        let text_edit = {
            let mut text_edit = TextEdit::singleline(&mut buffer_text)
                .id(edit_id)
                .hint_text(text_hint)
                .horizontal_align(horizontal_align)
                .vertical_align(vertical_align);

            if let Some(limit) = char_limit {
                text_edit = text_edit.char_limit(limit);
            }
            if state.invalid && !frameless {
                text_edit = text_edit.background_fill(ui.app_style().text_edit.invalid_fill);
            }
            if frameless {
                text_edit = text_edit.frameless();
            }

            text_edit
        };

        // Check if the edit content has changed or if the edit has lost focus.
        let response = if let Some(width) = desided_width {
            // Match the standard single-line editor height for fixed-width controls
            ui.add_sized(vec2(width, default_singleline_height(ui)), text_edit)
        } else {
            ui.add(text_edit)
        };
        let update = if update_while_editing {
            // Update changed drafts and canonicalize them when focus is lost
            response.changed() || response.lost_focus()
        } else {
            // Update only when the edit has lost focus
            response.lost_focus()
        };

        let mut invalid = state.invalid;
        if update {
            // Parse and normalize the draft before applying its range
            let is_empty = buffer_text.is_empty();
            let empty_is_valid = empty_value.is_some();
            let parsed_value = if is_empty {
                empty_value
            } else {
                buffer_text.parse::<V>().ok()
            }
            .and_then(|value| match normalize.as_ref() {
                Some(normalize) => normalize(value),
                None => Some(value),
            });
            let bounded = resolve_range.is_some();
            let resolved_value = parsed_value.map(|value| match resolve_range.as_ref() {
                Some(resolve_range) => resolve_range(value),
                None => RangeResolution::Valid(value),
            });

            match resolved_value {
                Some(RangeResolution::Valid(value)) => {
                    *text = value;
                    invalid = false;
                }
                Some(RangeResolution::Clamped(value)) => {
                    *text = value;
                    invalid = true;
                }
                None => {
                    invalid = bounded;
                }
            }

            // Preserve invalid ranged drafts and focused valid empty drafts
            let preserve_empty = is_empty && empty_is_valid && !response.lost_focus();
            if !(bounded && invalid) && !preserve_empty {
                buffer_text = text.to_string();
            }
        }

        // Repaint standard framed editors when their validation fill changes
        if invalid != state.invalid && !frameless {
            ui.ctx().request_repaint();
        }

        // Retain the draft and detect later model changes or visibility gaps
        state.buffer = buffer_text;
        state.invalid = invalid;
        state.model_text = text.to_string();
        state.last_seen_frame = frame;
        ui.mem().insert_temp(state_id, state);

        ValueEditOutput { response, invalid }
    }
}

impl<V: FromStr + Display> Widget for ValueEdit<'_, V> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// Result of rendering a [`ValueEdit`] with validation status.
pub struct ValueEditOutput {
    /// Interaction response for the underlying text editor.
    pub response: Response,
    /// Whether the currently displayed draft failed parsing or range validation.
    pub invalid: bool,
}

enum RangeResolution<V> {
    Valid(V),
    Clamped(V),
}

#[derive(Clone)]
struct ValueEditState {
    buffer: String,
    invalid: bool,
    model_text: String,
    last_seen_frame: u64,
}

impl ValueEditState {
    fn new(model_text: String, frame: u64) -> Self {
        Self {
            buffer: model_text.clone(),
            invalid: false,
            model_text,
            last_seen_frame: frame,
        }
    }

    fn reset(&mut self, model_text: String) {
        self.buffer = model_text.clone();
        self.invalid = false;
        self.model_text = model_text;
    }
}
