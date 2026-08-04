use egui::{CursorIcon, Id, Rect, Response, Sense, Ui, Vec2, Widget, vec2};
use segs_assets::icons::Icon;

const DEFAULT_ICON_SIZE: Vec2 = vec2(24., 24.);
const DEFAULT_ICON_PADDING: f32 = 3.;

pub struct IconBtn<'a> {
    variant: Variant<'a>,
    size: Vec2,
    padding: f32,
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
            size: DEFAULT_ICON_SIZE,
            padding: DEFAULT_ICON_PADDING,
        }
    }

    pub fn active(inactive_icon: impl Icon + 'static, active_icon: impl Icon + 'static, flag: &'a mut bool) -> Self {
        Self {
            variant: Variant::Active {
                inactive_icon: Box::new(inactive_icon),
                active_icon: Box::new(active_icon),
                active: flag,
            },
            size: DEFAULT_ICON_SIZE,
            padding: DEFAULT_ICON_PADDING,
        }
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    /// Overrides the padding between the button's background and the icon glyph.
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Shows the button at an explicit rectangle with a stable interaction ID.
    ///
    /// Unlike [`Ui::place`], this does not create a child UI with an automatic ID.
    pub fn show_at(self, ui: &mut Ui, rect: Rect, id: Id) -> Response {
        let response = ui.interact(rect, id, Sense::click());
        self.show_response(ui, rect, response)
    }

    fn show_response(self, ui: &mut Ui, rect: Rect, response: Response) -> Response {
        match self.variant {
            Variant::Inactive { icon } => icon_toggle(ui, icon, rect, response, self.padding),
            Variant::Active {
                inactive_icon,
                active_icon,
                active,
            } => {
                let icon = if *active { active_icon } else { inactive_icon };
                let response = icon_toggle(ui, icon, rect, response, self.padding);
                if response.clicked() {
                    *active = !*active;
                }
                response
            }
        }
    }
}

impl<'a> Widget for IconBtn<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click());
        self.show_response(ui, rect, response)
    }
}

fn icon_toggle(ui: &mut Ui, icon: Box<dyn Icon>, rect: Rect, response: Response, padding: f32) -> Response {
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

        let icon_rect = rect.shrink(padding);
        let icon_color = ui.visuals().text_color();
        icon.to_image()
            .tint(icon_color)
            .fit_to_exact_size(icon_rect.size())
            .paint_at(ui, icon_rect);
    }

    response.on_hover_cursor(CursorIcon::PointingHand)
}
