use std::sync::Arc;

use egui::{
    Align2, CornerRadius, CursorIcon, Galley, Image, Rect, Response, Sense, StrokeKind, Ui, Vec2, Widget, pos2, vec2,
};
use segs_assets::{Font, fonts::Figtree, icons::Icon};

pub struct SelectableLabel<'a, E: Eq> {
    selected: &'a mut E,
    options: Vec<LabelOption<E>>,
}

struct LabelOption<E: Eq> {
    icon: Box<dyn Icon>,
    text: String,
    value: E,
}

impl<'a, E: Eq> Widget for SelectableLabel<'a, E> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.scope(|ui| self.show(ui)).response
    }
}

impl<'a, E: Eq> SelectableLabel<'a, E> {
    pub fn new(selected: &'a mut E) -> Self {
        Self {
            selected,
            options: Vec::new(),
        }
    }

    pub fn option(mut self, icon: impl Icon + 'static, text: impl Into<String>, value: E) -> Self {
        let option = LabelOption {
            icon: Box::new(icon),
            text: text.into(),
            value,
        };
        self.options.push(option);
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let SelectableLabel { selected, options } = self;
        let (jobs, values): (Vec<LabelJob>, Vec<E>) = options.into_iter().map(LabelOption::decompose).unzip();

        // Layout texts and tint images
        let layouts = jobs
            .into_iter()
            .zip(values.iter())
            .map(|(job, value)| job.layout(ui, value == selected))
            .collect::<Vec<_>>();

        let cr = 4; // corner radius
        let sizes = layouts.iter().map(LabelLayout::compute_size).collect::<Vec<_>>();
        let total_x = sizes.iter().map(|s| s.x).sum::<f32>();
        let max_y = sizes.first().map(|s| s.y).unwrap_or(0.0);
        let (id, rect) = ui.allocate_space(vec2(total_x, max_y));

        if ui.is_rect_visible(rect) {
            // Paint border
            let stroke = ui.visuals().widgets.inactive.bg_stroke;
            ui.painter().rect_stroke(rect, cr, stroke, StrokeKind::Outside);

            let mut cursor_x: f32 = rect.min.x;
            let len = layouts.len();
            for (i, ((layout, size), value)) in layouts.into_iter().zip(sizes).zip(values).enumerate() {
                // Allocate response area
                let rect = Rect::from_min_size(pos2(cursor_x, rect.min.y), size);
                let id = id.with(i);
                let response = ui.interact(rect, id, Sense::click());

                // Adjust corner radius for first and last items
                let mut corner_radius = CornerRadius::ZERO;
                if i == 0 {
                    corner_radius.sw = cr;
                    corner_radius.nw = cr;
                } else if i == len - 1 {
                    corner_radius.ne = cr;
                    corner_radius.se = cr;
                }

                // Paint background
                let bg_fill = ui.visuals().widgets.hovered.bg_fill;
                if response.hovered() || layout.is_active {
                    ui.painter().rect_filled(rect, corner_radius, bg_fill);
                }

                // Change cursor on hover
                if ui.rect_contains_pointer(rect) {
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                }

                // Update selection on click
                if response.clicked() {
                    *selected = value;
                }

                // Paint layout
                layout.paint_at(ui, rect);

                // Advance cursor
                cursor_x += size.x;
            }
        }
    }
}

impl<E: Eq> LabelOption<E> {
    fn decompose(self) -> (LabelJob, E) {
        let Self { icon, text, value } = self;
        let job = LabelJob { icon, text };
        (job, value)
    }
}

struct LabelJob {
    icon: Box<dyn Icon>,
    text: String,
}

impl LabelJob {
    fn layout(self, ui: &mut Ui, is_active: bool) -> LabelLayout {
        let LabelJob { icon, text } = self;

        let style = &ui.visuals().widgets;
        let stroke_color = if is_active { style.active } else { style.inactive }.fg_stroke.color;

        let font = Figtree::medium().sized(15.);

        let galley = ui.painter().layout_no_wrap(text, font, stroke_color);
        let image = icon.to_image().tint(stroke_color);

        LabelLayout {
            image,
            galley,
            is_active,
        }
    }
}

struct LabelLayout {
    image: Image<'static>,
    galley: Arc<Galley>,
    is_active: bool,
}

const ICON_SIZE: Vec2 = vec2(17., 17.);
const INSET: f32 = 5.;
const PADDING: Vec2 = vec2(7., 2.);

impl LabelLayout {
    fn compute_size(&self) -> Vec2 {
        let text_size = self.galley.size();
        let x = PADDING.x * 2.0 + INSET + ICON_SIZE.x + text_size.x;
        let y = PADDING.y * 2.0 + ICON_SIZE.y.max(text_size.y);
        vec2(x, y)
    }

    fn paint_at(self, ui: &mut Ui, rect: Rect) {
        let Self {
            image,
            galley,
            is_active,
        } = self;

        // Set cursor position after padding
        let mut cursor_pos = pos2(rect.min.x + PADDING.x, rect.center().y);

        // Set up icon rect and advance cursor
        let icon_rect = Align2::LEFT_CENTER.anchor_size(cursor_pos, ICON_SIZE);
        cursor_pos.x += ICON_SIZE.x + INSET;

        // Set up text rect
        let text_rect = Align2::LEFT_CENTER.anchor_size(cursor_pos, galley.size());

        let painter = ui.painter();

        // Paint Icon
        image.fit_to_exact_size(ICON_SIZE).paint_at(ui, icon_rect);

        // Paint text
        let style = &ui.visuals().widgets;
        let stroke_color = if is_active { style.active } else { style.inactive }.fg_stroke.color;
        painter.galley(text_rect.min, galley, stroke_color);
    }
}
