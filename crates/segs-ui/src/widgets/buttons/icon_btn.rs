use egui::{Response, Sense, Ui, Vec2, Widget};
use segs_assets::icons::Icon;

use crate::StyleExt;

pub struct IconBtn<'a> {
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
impl<'a> IconBtn<'a> {
    pub fn new(icon: impl Icon + 'static) -> Self {
        Self {
            variant: Variant::Inactive { icon: Box::new(icon) },
        }
    }

    pub fn active(inactive_icon: impl Icon + 'static, active_icon: impl Icon + 'static, flag: &'a mut bool) -> Self {
        Self {
            variant: Variant::Active {
                inactive_icon: Box::new(inactive_icon),
                active_icon: Box::new(active_icon),
                active: flag,
            },
        }
    }
}

impl<'a> Widget for IconBtn<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        match self.variant {
            Variant::Inactive { icon } => icon_toggle(ui, icon),
            Variant::Active {
                inactive_icon,
                active_icon,
                active,
            } => {
                let icon = if *active { active_icon } else { inactive_icon };
                let response = icon_toggle(ui, icon);
                if response.clicked() {
                    *active = !*active;
                }
                response
            }
        }
    }
}

fn icon_toggle(ui: &mut Ui, icon: Box<dyn Icon>) -> Response {
    let toggle_size = Vec2::new(25., 25.);
    let (rect, response) = ui.allocate_exact_size(toggle_size, Sense::click());
    let id = response.id;

    // Animation factors
    let hover_t = ui.ctx().animate_bool(id.with("anim_hover"), response.hovered());
    let active_t = ui
        .ctx()
        .animate_bool(id.with("anim_active"), response.is_pointer_button_down_on());

    // Paint the button
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let scale = 1. + (hover_t * 0.1) - (active_t * 0.15);
        let animated_rect = rect.expand2(rect.shrink(1.).size() * (scale - 1.0) * 0.5);

        if hover_t > 0. {
            let shadow_color = ui.app_visuals().shadow_color_lerp(hover_t);
            painter.rect_filled(animated_rect.shrink(1.), 5., shadow_color);
        }

        let icon_rect = animated_rect.shrink(2.);
        let icon_color = ui.visuals().text_color();
        icon.to_image()
            .tint(icon_color)
            .fit_to_exact_size(icon_rect.size())
            .paint_at(ui, icon_rect);
    }

    response
}
