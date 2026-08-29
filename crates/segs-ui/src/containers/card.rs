use egui::{Frame, InnerResponse, Margin, Stroke, Ui};

use crate::style::CtxStyleExt;

/// A theme-aware framed surface for grouping related content.
#[derive(Clone, Copy, Default)]
pub struct Card;

impl Card {
    pub fn new() -> Self {
        Self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let app_style = ui.app_style();
        Frame::new()
            .fill(app_style.widgets.noninteractive.bg_fill)
            .stroke(Stroke::new(1_f32, app_style.widgets.noninteractive.bg_stroke_color))
            .corner_radius(4)
            .inner_margin(Margin::same(8))
            .show(ui, add_contents)
    }
}
