use egui::{CursorIcon, Response, Sense, Ui, Vec2, Widget};
use segs_assets::icons::Icon;

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
    let toggle_size = Vec2::new(24., 24.);
    let (rect, response) = ui.allocate_exact_size(toggle_size, Sense::click());

    // Paint the button
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let rounded = 6.;
        let is_active = response.is_pointer_button_down_on();
        let is_hovered = response.hovered();
        if is_hovered || is_active {
            let bg_color = if is_active {
                ui.visuals().widgets.active.bg_fill
            } else {
                ui.visuals().widgets.hovered.bg_fill
            };
            painter.rect_filled(rect.shrink(1.), rounded, bg_color);
        }

        let icon_rect = rect.shrink(3.);
        let icon_color = ui.visuals().text_color();
        icon.to_image()
            .tint(icon_color)
            .fit_to_exact_size(icon_rect.size())
            .paint_at(ui, icon_rect);
    }

    response.on_hover_cursor(CursorIcon::PointingHand)
}
