use std::hash::Hash;

use egui::{CursorIcon, Rect, Response, Sense, Shape, Stroke, Ui, UiBuilder, Widget, pos2, vec2};

use crate::CtxStyleExt;

pub struct Checkbox<'a> {
    flag: &'a mut bool,
    builder: UiBuilder,
}

impl<'a> Checkbox<'a> {
    pub fn new(flag: &'a mut bool) -> Checkbox<'a> {
        Checkbox {
            flag,
            builder: UiBuilder::default(),
        }
    }

    pub fn with_id(mut self, id: impl Hash) -> Self {
        self.builder = self.builder.id(id);
        self
    }
}

impl Widget for Checkbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.scope_builder(self.builder, |ui| add_checkbox(ui, self.flag)).inner
    }
}

fn add_checkbox(ui: &mut Ui, active: &mut bool) -> Response {
    let size = vec2(15.0, 15.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let id = response.id;

        // Toggle flag on click
        if response.clicked() {
            *active = !*active;
        }

        // Set pointing hand cursor on hover
        let response = response.on_hover_cursor(CursorIcon::PointingHand);

        // Pointer down effects
        let pointer_down = response.is_pointer_button_down_on();
        let pressed_t = ui.ctx().animate_bool(id.with("pressed_t"), pointer_down);
        let rect = rect.shrink(pressed_t * 1.0);

        // Animation factor
        ui.style_mut().animation_time = 0.1;
        let click_t = ui.ctx().animate_bool(id.with("active_t"), *active);

        let painter = ui.painter();

        // Paint background
        let style = &ui.style().interact(&response);
        let accent = ui.app_style().accent_color;
        let bg_color = style.bg_fill.lerp_to_gamma(accent, click_t);
        painter.rect_filled(rect, 2.0, bg_color);

        // Paint cross
        let t = (click_t + pressed_t * 0.5).clamp(0.0, 1.0);
        paint_parametric_check(ui, rect.shrink(1.0), t, style.fg_stroke);

        response
    } else {
        response
    }
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
