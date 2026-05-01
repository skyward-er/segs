use egui::{Response, Ui, UiBuilder, Vec2, Widget, emath::easing, vec2};
use segs_assets::{Font, fonts::Figtree};

use crate::{AnimationExt, style::CtxStyleExt};

pub struct VerticalSelectableLabel<'a, V: PartialEq> {
    selector: &'a mut V,
    variants: Vec<SelectableLabel<V>>,
}

struct SelectableLabel<V: PartialEq> {
    variant: V,
    text: String,
}

impl<V: PartialEq> Widget for VerticalSelectableLabel<'_, V> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

impl<'a, V: PartialEq> VerticalSelectableLabel<'a, V> {
    pub fn new(selector: &'a mut V) -> Self {
        Self {
            selector,
            variants: Vec::new(),
        }
    }

    pub fn add_variant(mut self, variant: V, text: impl Into<String>) -> Self {
        self.variants
            .push(SelectableLabel::new(variant, text.into().to_uppercase()));
        self
    }

    fn show(self, ui: &mut Ui) -> Response {
        let Self { selector, variants } = self;

        ui.spacing_mut().item_spacing = Vec2::ZERO;

        // Show all label variants
        let (responses, selected): (Vec<_>, Vec<_>) = variants
            .into_iter()
            // Here use a scope to ensure unique ids for each label
            .map(|v| {
                let builder = UiBuilder::new().id(&v.text);
                ui.scope_builder(builder, |ui| v.show(ui, selector)).inner
            })
            .unzip();

        // Find the position of the selected label
        let selected = selected.iter().position(|s| *s);
        if let Some(selected) = selected {
            // Get the y positions of all icons to calculate the offset for the selector
            let icon_positions = responses.iter().map(|r| r.rect.left_center()).collect::<Vec<_>>();
            // Use a fixed point (the first icon) to calculate the offset for the selector,
            // so it doesn't jump when resizing the window.
            let fixed_pos = icon_positions[0];

            // Calculate the offset for the selector based on the selected label's position
            let offset = icon_positions[selected].y - fixed_pos.y;
            let id = ui.id().with("_selector");

            // Animate the offset using a cubic easing function for a smooth transition
            let offset_t = ui
                .ctx()
                .animate_value_with_time_and_easing(id, offset, 0.2, easing::cubic_out);
            let selector_pos = fixed_pos + vec2(10., offset_t);

            // Paint the selector icon at the calculated position
            let radius = 3.0;
            let sel_color = ui.app_style().left_bar.icon_active_color;
            ui.painter().circle_filled(selector_pos, radius, sel_color);
        }

        responses.into_iter().reduce(|a, b| a.union(b)).unwrap_or(ui.response())
    }
}

impl<V: PartialEq> SelectableLabel<V> {
    fn new(variant: V, text: impl Into<String>) -> Self {
        Self {
            variant,
            text: text.into(),
        }
    }

    fn show(self, ui: &mut Ui, selector: &mut V) -> (Response, bool) {
        let Self { variant, text } = self;

        // Allocate space for the label and get a response for interaction
        // FIXME (federico): This hardcoded size is a band-aid for a layout issue where
        // the label text can be layed out outside of the allocated rect, causing clicks
        // and hovers to not register. One should split this in two part (a begin that
        // creates the galley and calculates the size, and an end that takes the galley
        // and paints it).
        let min_option_size = vec2(100., 19.);
        let (rect, mut response) = ui.allocate_at_least(min_option_size, egui::Sense::click());

        let is_selected = *selector == variant;

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Define positions for painted elements
            let text_start = rect.left_top() + vec2(20., 3.);

            // Style elements
            let inactive_color = ui.app_style().left_bar.icon_inactive_color;
            let active_color = ui.app_style().left_bar.icon_active_color;
            let animation_time = 0.3;

            // Set ids for animations
            let id = ui.id();
            let hover_id = id.with("_hover_animation");
            let select_id = id.with("_selection_animation");

            // Animate selection state
            let selection_t = ui.ctx().animate_bool_with_time(select_id, is_selected, animation_time);
            let hover_t = ui.ctx().animate_bool_with_time_and_easing(
                hover_id,
                response.hovered(),
                animation_time,
                easing::cubic_out,
            );

            // Layout text with appropriate font and color based on selection state
            let font_id = if is_selected {
                Figtree::bold()
            } else {
                Figtree::medium()
            }
            .sized(11.);
            let text_color = inactive_color.lerp_to_gamma(active_color, selection_t.max(hover_t));
            let galley = painter.layout_no_wrap(text.to_string(), font_id, text_color);

            // Change cursor on hover
            response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

            // Handle click to update selection
            if response.clicked() {
                *selector = variant;
            }

            // Paint text
            painter.galley(text_start, galley, text_color);
        }

        (response, is_selected)
    }
}
