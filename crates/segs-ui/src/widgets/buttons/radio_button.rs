use egui::{CursorIcon, Id, Rect, Response, Sense, Stroke, Ui, UiBuilder, Widget, vec2};

use crate::style::CtxStyleExt;

/// A mutually exclusive selection indicator.
pub struct RadioButton<'a> {
    selected: &'a mut bool,
    builder: UiBuilder,
}

impl<'a> RadioButton<'a> {
    /// Default radio button size.
    pub const SIZE: egui::Vec2 = vec2(14., 14.);

    /// Creates a radio button bound to the provided selection flag.
    pub fn new(selected: &'a mut bool) -> Self {
        Self {
            selected,
            builder: UiBuilder::default(),
        }
    }

    /// Paints a radio button at the given rectangle without clearing an active selection.
    pub fn show_at(ui: &mut Ui, selected: &mut bool, rect: Rect, response: Response) -> Response {
        let selection_id = response.id;
        Self::show_at_with_selection_id(ui, selected, rect, response, selection_id)
    }

    /// Paints a radio button with selection animation attached to the provided identity.
    pub fn show_at_with_selection_id(
        ui: &mut Ui,
        selected: &mut bool,
        rect: Rect,
        response: Response,
        selection_id: Id,
    ) -> Response {
        if ui.is_rect_visible(rect) && response.clicked() {
            *selected = true;
        }

        show_radio_button(ui, *selected, rect, response, selection_id)
    }
}

impl Widget for RadioButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.scope_builder(self.builder, |ui| {
            let (rect, response) = ui.allocate_exact_size(Self::SIZE, Sense::click());
            Self::show_at(ui, self.selected, rect, response)
        })
        .inner
    }
}

fn show_radio_button(ui: &mut Ui, selected: bool, rect: Rect, response: Response, selection_id: Id) -> Response {
    if !ui.is_rect_visible(rect) {
        return response;
    }

    let id = response.id;
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    let pointer_down = response.is_pointer_button_down_on();
    let pressed_t = ui.ctx().animate_bool(id.with("pressed_t"), pointer_down);
    let rect = rect.shrink(pressed_t);

    ui.style_mut().animation_time = 0.1;
    let selected_t = ui.ctx().animate_bool(selection_id.with("_selected_t"), selected);

    let style = ui.app_style();
    let radius = rect.width().min(rect.height()) * 0.5;
    ui.painter()
        .circle(rect.center(), radius, style.widgets.inactive.bg_fill, Stroke::NONE);
    ui.painter().circle(
        rect.center(),
        radius * 0.55 * selected_t,
        style.accent_fill,
        Stroke::NONE,
    );

    response
}
