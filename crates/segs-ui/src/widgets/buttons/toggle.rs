use egui::{CursorIcon, Response, Sense, Ui, Widget, emath::easing, lerp, pos2, vec2};

use crate::style::CtxStyleExt;

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
        let id = response.id;

        // Toggle flag on click
        if response.clicked() {
            *active = !*active;
        }

        // Change cursor on hover
        let response = response.on_hover_cursor(CursorIcon::PointingHand);

        // Animation factor
        let click_t = ui
            .ctx()
            .animate_bool_with_easing(id.with("_click_t"), *active, easing::cubic_in);
        let hover_t = ui
            .ctx()
            .animate_bool_responsive(id.with("_hover_t"), response.hovered());
        let style = ui.app_style();

        // Paint background
        let enabled_color = style.confirmation_fill;
        let bg_fill = style
            .widgets
            .inactive
            .bg_fill
            .lerp_to_gamma(style.widgets.hovered.bg_fill, hover_t);
        let bg_color = bg_fill.lerp_to_gamma(enabled_color, click_t);
        let corner_radius = height / 2.0;
        painter.rect_filled(rect, corner_radius, bg_color);

        // Paint circle
        let off_x = rect.min.x + corner_radius;
        let on_x = rect.max.x - corner_radius;
        let x = lerp(off_x..=on_x, click_t);
        let y = rect.min.y + corner_radius;
        let center = pos2(x, y);
        let radius = corner_radius - 2.0;

        let circle_color = style.current_background_fill;
        painter.circle_filled(center, radius, circle_color);
    }
}
