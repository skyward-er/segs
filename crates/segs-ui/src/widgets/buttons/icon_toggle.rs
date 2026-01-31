use std::hash::Hash;

use egui::{Id, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};
use segs_assets::icons::Icon;

use crate::StyleExt;

pub struct IconToggle<'a> {
    id_source: Option<Id>,
    variant: Variant<'a>,
}

enum Variant<'a> {
    Inactive {
        icon: Box<dyn Icon>,
    },
    Active {
        inactive_icon: Box<dyn Icon>,
        active_icon: Box<dyn Icon>,
        active: &'a mut bool,
    },
}

// Base constructor - works for any Icon
impl<'a> IconToggle<'a> {
    pub fn new(icon: impl Icon + 'static) -> Self {
        Self {
            id_source: None,
            variant: Variant::Inactive { icon: Box::new(icon) },
        }
    }

    pub fn active(inactive_icon: impl Icon + 'static, active_icon: impl Icon + 'static, flag: &'a mut bool) -> Self {
        Self {
            id_source: None,
            variant: Variant::Active {
                inactive_icon: Box::new(inactive_icon),
                active_icon: Box::new(active_icon),
                active: flag,
            },
        }
    }

    pub fn with_id(mut self, id_source: impl Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }
}

impl<'a> Widget for IconToggle<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let id = self.id_source.unwrap_or_else(|| ui.next_auto_id());
        match self.variant {
            Variant::Inactive { icon } => icon_toggle(ui, id, icon),
            Variant::Active {
                inactive_icon,
                active_icon,
                active,
            } => {
                let icon = if *active { active_icon } else { inactive_icon };
                let response = icon_toggle(ui, id, icon);
                if response.clicked() {
                    *active = !*active;
                }
                response
            }
        }
    }
}

fn icon_toggle(ui: &mut Ui, id_source: impl Hash, icon: Box<dyn Icon>) -> Response {
    let toggle_size = Vec2::new(26.0, 26.0);
    let (rect, response) = ui.allocate_exact_size(toggle_size, Sense::click());
    let id = Id::new(id_source);

    // Animation factors
    let hover_t = ui.ctx().animate_bool(id.with("anim_hover"), response.hovered());
    let active_t = ui
        .ctx()
        .animate_bool(id.with("anim_active"), response.is_pointer_button_down_on());

    // Paint the button
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let scale = 1.0 + (hover_t * 0.02) - (active_t * 0.06);
        let animated_rect = rect.expand2(rect.size() * (scale - 1.0) * 0.5);

        if hover_t > 0.0 {
            let shadow_size = hover_t * 2.5;
            let shadow_color = ui.app_visuals().shadow_color_lerp(hover_t);
            painter.rect_filled(animated_rect.shrink(shadow_size), 4.0, shadow_color);
        }

        let icon_rect = animated_rect.shrink(6.0);
        let snapped_rect = snap_rect_to_pixels(icon_rect, ui.ctx().pixels_per_point());

        let icon_color = ui.app_visuals().icon_color;
        icon.to_image()
            .tint(icon_color)
            .fit_to_exact_size(snapped_rect.size())
            .paint_at(ui, snapped_rect);
    }

    response
}

fn snap_rect_to_pixels(rect: Rect, pixels_per_point: f32) -> Rect {
    let min_px = Pos2::new(rect.min.x * pixels_per_point, rect.min.y * pixels_per_point);
    let max_px = Pos2::new(rect.max.x * pixels_per_point, rect.max.y * pixels_per_point);

    let min_px = Pos2::new(min_px.x.round(), min_px.y.round());
    let max_px = Pos2::new(max_px.x.round(), max_px.y.round());

    Rect::from_min_max(
        Pos2::new(min_px.x / pixels_per_point, min_px.y / pixels_per_point),
        Pos2::new(max_px.x / pixels_per_point, max_px.y / pixels_per_point),
    )
}
