use std::f32::consts::PI;

use egui::{
    CursorIcon, Margin, Response, Sense, TextStyle, TextWrapMode, Ui, Vec2, Widget, WidgetInfo, WidgetText, WidgetType,
    pos2, vec2,
};
use segs_assets::icons::{CaretDown, Icon};

use crate::style::CtxStyleExt;

const VERTICAL_PADDING: f32 = 3.;
const ROW_SPACING: f32 = 2.;
const INDICATOR_SIZE: f32 = 14.;
const INDICATOR_RIGHT_PADDING: f32 = 6.;
const INDICATOR_TEXT_SPACING: f32 = 4.;
const INDICATOR_ANIMATION_DURATION_FACTOR: f32 = 2.;

/// A two-row selector that toggles caller-owned expanded content.
pub struct ExpandableSelector<'a> {
    label: WidgetText,
    preview: WidgetText,
    expanded: &'a mut bool,
    preview_weak: bool,
    horizontal_bleed: Margin,
}

impl<'a> ExpandableSelector<'a> {
    /// Creates a selector for the provided label, preview, and expanded state.
    pub fn new(label: impl Into<WidgetText>, preview: impl Into<WidgetText>, expanded: &'a mut bool) -> Self {
        Self {
            label: label.into(),
            preview: preview.into(),
            expanded,
            preview_weak: false,
            horizontal_bleed: Margin::ZERO,
        }
    }

    /// Uses reduced foreground emphasis for the preview.
    pub fn preview_weak(mut self, weak: bool) -> Self {
        self.preview_weak = weak;
        self
    }

    /// Extends painting and interaction through the provided horizontal margins.
    pub fn horizontal_bleed(mut self, margin: Margin) -> Self {
        self.horizontal_bleed = margin;
        self
    }
}

impl Widget for ExpandableSelector<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            label,
            preview,
            expanded,
            preview_weak,
            horizontal_bleed,
        } = self;

        // Preserve plain text before galley conversion consumes the widget text
        let label_text = label.text().to_owned();
        let preview_text = preview.text().to_owned();

        // Reserve indicator space so truncated text cannot paint beneath it
        let width = ui.available_width().max(0.);
        let text_width =
            (width + horizontal_bleed.rightf() - INDICATOR_SIZE - INDICATOR_RIGHT_PADDING - INDICATOR_TEXT_SPACING)
                .max(0.);
        let label_galley = label.into_galley(ui, Some(TextWrapMode::Truncate), text_width, TextStyle::Body);
        let preview_galley = preview.into_galley(ui, Some(TextWrapMode::Truncate), text_width, TextStyle::Body);

        // Size the normal layout from two body rows and compact vertical padding
        let height = VERTICAL_PADDING * 2. + label_galley.size().y + ROW_SPACING + preview_galley.size().y;
        let (id, content_rect) = ui.allocate_space(vec2(width, height));

        // Bleed painting and interaction into outer margins without changing layout
        let interaction_rect = egui::Rect::from_min_max(
            pos2(content_rect.left() - horizontal_bleed.leftf(), content_rect.top()),
            pos2(content_rect.right() + horizontal_bleed.rightf(), content_rect.bottom()),
        );
        let response = ui
            .interact(interaction_rect, id, Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);

        // Keep expansion state owned by the caller
        if response.clicked() {
            *expanded = !*expanded;
        }

        // Expose both rows and current expansion state as one accessible control
        let accessibility_label = format!(
            "{label_text}: {preview_text}, {}",
            if *expanded { "expanded" } else { "collapsed" }
        );
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), &accessibility_label));

        if ui.is_rect_visible(interaction_rect) {
            // Select custom colors from the complete interaction state
            let app_style = ui.app_style();
            let interaction = if response.is_pointer_button_down_on() {
                &app_style.widgets.active
            } else if response.hovered() || response.has_focus() {
                &app_style.widgets.hovered
            } else {
                &app_style.widgets.inactive
            };

            // Keep the idle state flat and paint only transient feedback
            if response.is_pointer_button_down_on() || response.hovered() || response.has_focus() {
                ui.painter().rect_filled(interaction_rect, 0., interaction.bg_fill);
            }

            // Align text to content and the indicator to the bled right edge
            let text_color = ui.visuals().text_color();
            let label_pos = pos2(content_rect.left(), content_rect.top() + VERTICAL_PADDING);
            let preview_pos = pos2(content_rect.left(), label_pos.y + label_galley.size().y + ROW_SPACING);
            let indicator_rect = egui::Rect::from_center_size(
                pos2(
                    interaction_rect.right() - INDICATOR_RIGHT_PADDING - INDICATOR_SIZE * 0.5,
                    interaction_rect.center().y,
                ),
                Vec2::splat(INDICATOR_SIZE),
            );
            ui.painter().galley(label_pos, label_galley, text_color);

            // Deemphasize empty or unavailable previews when requested
            let preview_color = if preview_weak {
                text_color.gamma_multiply(ui.visuals().weak_text_alpha)
            } else {
                text_color
            };
            ui.painter().galley(preview_pos, preview_galley, preview_color);
            let animation_time = ui.style().animation_time * INDICATOR_ANIMATION_DURATION_FACTOR;
            let openness = ui.ctx().animate_bool_with_time_and_easing(
                response.id.with("indicator_openness"),
                *expanded,
                animation_time,
                egui::emath::easing::cubic_out,
            );

            // Rotate the persistent caret through half a turn as state changes
            CaretDown::solid()
                .to_image()
                .tint(text_color)
                .rotate(PI * openness, Vec2::splat(0.5))
                .fit_to_exact_size(indicator_rect.size())
                .paint_at(ui, indicator_rect);
        }

        response
    }
}
