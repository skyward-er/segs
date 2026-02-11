use egui::{CursorIcon, Label, Rect, RectAlign, Response, Sense, Tooltip, Ui, Vec2, Widget, vec2};
use segs_assets::icons::Icon;

use crate::StyleExt;

pub struct RibbonToggle<'a> {
    active: &'a mut bool,
    icon_size: Vec2,
    shadow_size: Vec2,
    tooltip_text: Option<String>,
    inactive_icon: Box<dyn Icon>,
    active_icon: Box<dyn Icon>,
}

impl<'a> RibbonToggle<'a> {
    pub fn new(inactive_icon: Box<dyn Icon>, active_icon: Box<dyn Icon>, active: &'a mut bool) -> Self {
        Self {
            active,
            icon_size: vec2(20., 20.),
            shadow_size: vec2(30., 30.),
            tooltip_text: None,
            inactive_icon,
            active_icon,
        }
    }

    pub fn icon_size(mut self, size: Vec2) -> Self {
        self.icon_size = size;
        self
    }

    pub fn shadow_size(mut self, size: Vec2) -> Self {
        self.shadow_size = size;
        self
    }

    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip_text = Some(text.into());
        self
    }

    fn show(self, ui: &mut Ui) -> Response {
        let Self {
            active,
            icon_size,
            shadow_size,
            tooltip_text,
            inactive_icon,
            active_icon,
        } = self;

        let (rect, response) = ui.allocate_exact_size(shadow_size, Sense::click());
        let id = response.id;

        // Customize tooltip
        if let Some(text) = tooltip_text {
            let mut tooltip = Tooltip::for_enabled(&response);
            tooltip.popup = tooltip.popup.align(RectAlign::RIGHT);
            tooltip.show(|ui| {
                ui.add(Label::new(text).selectable(false));
            });
        }

        // Change cursor on hover
        let response = response.on_hover_cursor(CursorIcon::PointingHand);

        // Paint the button
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Toggle active state on click
            if response.clicked() {
                *active = !*active;
            }

            // Animation factors
            let hover_t = ui
                .ctx()
                .animate_bool_with_time(id.with("anim_hover"), response.hovered(), 0.1);
            let active_t = ui.ctx().animate_bool_with_time(id.with("anim_active"), *active, 0.1);
            let combined_t = hover_t.max(active_t);

            // Paint the shadow
            let visuals = ui.app_visuals();
            let shadow_color = if active_t > 0. {
                visuals.menu_icon_shadow_color_active.gamma_multiply(active_t)
            } else {
                visuals.menu_icon_shadow_color_hover.gamma_multiply(hover_t)
            };
            painter.rect_filled(rect, 0., shadow_color);

            // Paint the icon
            let icon_rect = Rect::from_center_size(rect.center(), icon_size);
            let visuals = ui.app_visuals();
            let inactive_color = visuals.menu_icon_inactive_color;
            let active_color = visuals.menu_icon_active_color;
            let icon_color = inactive_color.lerp_to_gamma(active_color, combined_t);

            if active_t < 1. {
                inactive_icon
                    .to_image()
                    .tint(icon_color.gamma_multiply(1. - active_t))
                    .fit_to_exact_size(icon_rect.size())
                    .paint_at(ui, icon_rect);
            }
            if active_t > 0. {
                active_icon
                    .to_image()
                    .tint(icon_color.gamma_multiply(active_t))
                    .fit_to_exact_size(icon_rect.size())
                    .paint_at(ui, icon_rect);
            }
        }

        response
    }
}

impl Widget for RibbonToggle<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}
