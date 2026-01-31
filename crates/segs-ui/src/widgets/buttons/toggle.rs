use egui::{CursorIcon, Response, Sense, Ui, Widget, emath::easing, lerp, pos2, vec2};

use crate::StyleExt;

pub struct Toggle<'a> {
    flag: &'a mut bool,
}

impl<'a> Toggle<'a> {
    pub fn new(flag: &'a mut bool) -> Toggle<'a> {
        Toggle { flag }
    }
}

impl Widget for Toggle<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.scope(|ui| add_toggle(ui, self.flag)).response
    }
}

fn add_toggle(ui: &mut Ui, active: &mut bool) {
    let height = 17.5;
    let width = 30.0;

    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let id = ui.next_auto_id().with("toggle_asd");

        // Toggle flag on click
        if response.clicked() {
            *active = !*active;
        }

        // Change cursor on hover
        let response = response.on_hover_cursor(CursorIcon::PointingHand);

        // Animation factor
        let click_t = ui
            .ctx()
            .animate_bool_with_time_and_easing(id.with("anim"), *active, 0.1, easing::cubic_in);
        let style = ui.style().interact(&response);

        // Paint background
        let enabled_color = ui.app_visuals().enabled_color;
        let bg_color = style.bg_fill.lerp_to_gamma(enabled_color, click_t);
        let corner_radius = height / 2.0;
        painter.rect_filled(rect, corner_radius, bg_color);

        // Paint circle
        let off_x = rect.min.x + corner_radius;
        let on_x = rect.max.x - corner_radius;
        let x = lerp(off_x..=on_x, click_t);
        let y = rect.min.y + corner_radius;
        let center = pos2(x, y);
        let radius = corner_radius - 2.0;

        let circle_color = ui.visuals().panel_fill;
        painter.circle_filled(center, radius, circle_color);
    }
}
