use std::hash::Hash;

use egui::{CursorIcon, Id, Rect, Response, Sense, Shape, Stroke, Ui, UiBuilder, Widget, pos2, vec2};

use crate::style::CtxStyleExt;

/// The visual selection state of a checkbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckState {
    Unchecked,
    Partial,
    Checked,
}

/// A selectable check indicator.
pub struct Checkbox<'a> {
    flag: &'a mut bool,
    builder: UiBuilder,
}

impl<'a> Checkbox<'a> {
    /// Default checkbox size.
    pub const SIZE: egui::Vec2 = vec2(15., 15.);

    /// Creates a checkbox bound to the provided flag.
    pub fn new(flag: &'a mut bool) -> Checkbox<'a> {
        Checkbox {
            flag,
            builder: UiBuilder::default(),
        }
    }

    pub fn with_id(mut self, id: impl Hash) -> Self {
        self.builder = self.builder.id_salt(id);
        self
    }

    /// Paints a checkbox at the given rectangle.
    pub fn show_at(ui: &mut Ui, active: &mut bool, rect: Rect, response: Response) -> Response {
        let selection_id = response.id;
        Self::show_at_with_selection_id(ui, active, rect, response, selection_id)
    }

    /// Paints a checkbox with selection animation attached to the provided identity.
    pub fn show_at_with_selection_id(
        ui: &mut Ui,
        active: &mut bool,
        rect: Rect,
        response: Response,
        selection_id: Id,
    ) -> Response {
        if ui.is_rect_visible(rect) && response.clicked() {
            *active = !*active;
        }

        let state = if *active {
            CheckState::Checked
        } else {
            CheckState::Unchecked
        };
        show_checkbox(ui, state, rect, response, selection_id)
    }

    /// Paints a checkbox state at the given rectangle without changing it.
    pub fn show_state_at(ui: &mut Ui, state: CheckState, rect: Rect, response: Response) -> Response {
        let selection_id = response.id;
        Self::show_state_at_with_selection_id(ui, state, rect, response, selection_id)
    }

    /// Paints a checkbox state with selection animation attached to the provided identity.
    pub fn show_state_at_with_selection_id(
        ui: &mut Ui,
        state: CheckState,
        rect: Rect,
        response: Response,
        selection_id: Id,
    ) -> Response {
        show_checkbox(ui, state, rect, response, selection_id)
    }
}

impl Widget for Checkbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.scope_builder(self.builder, |ui| {
            let (rect, response) = ui.allocate_exact_size(Self::SIZE, Sense::click());
            Self::show_at(ui, self.flag, rect, response)
        })
        .inner
    }
}

fn show_checkbox(ui: &mut Ui, state: CheckState, rect: Rect, response: Response, selection_id: Id) -> Response {
    if ui.is_rect_visible(rect) {
        let id = response.id;

        // Set pointing hand cursor on hover
        let response = response.on_hover_cursor(CursorIcon::PointingHand);

        // Pointer down effects
        let pointer_down = response.is_pointer_button_down_on();
        let pressed_t = ui.ctx().animate_bool(id.with("pressed_t"), pointer_down);
        let rect = rect.shrink(pressed_t * 1.0);

        // Animation factor
        ui.style_mut().animation_time = 0.1;
        let active = state != CheckState::Unchecked;
        let click_t = ui.ctx().animate_bool(selection_id.with("_active_t"), active);
        let hover_t = ui
            .ctx()
            .animate_bool_responsive(id.with("_hover_t"), response.hovered());

        let painter = ui.painter();

        // Paint background
        let style = ui.app_style();
        let accent = style.accent_fill;
        let bg_fill = style
            .widgets
            .inactive
            .bg_fill
            .lerp_to_gamma(style.widgets.hovered.bg_fill, hover_t);
        let bg_color = bg_fill.lerp_to_gamma(accent, click_t);
        painter.rect_filled(rect, 2.0, bg_color);

        // Paint the state mark
        let t = (click_t + pressed_t * 0.5).clamp(0.0, 1.0);
        let interact_style = ui.style().interact(&response);
        match state {
            CheckState::Unchecked | CheckState::Checked => {
                paint_parametric_check(ui, rect.shrink(1.0), t, interact_style.fg_stroke);
            }
            CheckState::Partial => {
                paint_partial_check(ui, rect.shrink(1.0), t, interact_style.fg_stroke);
            }
        }

        response
    } else {
        response
    }
}

fn paint_partial_check(ui: &mut Ui, rect: Rect, t: f32, stroke: Stroke) {
    let half_width = rect.width() * 0.3 * t;
    let center = rect.center();
    ui.painter().line_segment(
        [
            pos2(center.x - half_width, center.y),
            pos2(center.x + half_width, center.y),
        ],
        stroke,
    );
}

fn paint_parametric_check(ui: &mut Ui, rect: Rect, t: f32, stroke: Stroke) {
    let painter = ui.painter();

    // Scale our normalized coordinates (0.2 to 0.8) to the actual UI rect
    let paint_pos = |x: f32, y: f32| pos2(rect.min.x + x * rect.width(), rect.min.y + y * rect.height());

    // Segmented line drawing
    let p1 = paint_pos(0.2, 0.5);
    // let p2 = paint_pos(0.5, 0.8);
    let p2 = paint_pos(0.4, 0.7);
    let p3 = paint_pos(0.8, 0.3);

    let t1 = (t / 0.5).min(1.0); // p1-p2
    let t2 = ((t - 0.5) / 0.5).max(0.0); // p2-p3

    // Define points based on t1 and t2
    let mut points = vec![p1];
    if t1 < 1.0 {
        points.push(p1.lerp(p2, t1));
    } else if t2 < 1.0 {
        points.push(p2);
        points.push(p2.lerp(p3, t2));
    } else {
        points.push(p2);
        points.push(p3);
    };

    // Draw the line
    painter.add(Shape::line(points, stroke));
}
