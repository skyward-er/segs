use egui::{Align, Color32, FontId, Ui, Vec2, pos2};
use segs_ui::style::CtxStyleExt;

use super::auto_size;

/// Default fixed value text size in logical points.
pub const DEFAULT_TEXT_SIZE: i64 = 32;
/// Preferred width of the text-size setting editor in logical points.
pub const TEXT_SIZE_SETTING_WIDTH: f32 = 96.;
const LABEL_TEXT_SIZE_SCALE: f32 = 0.75;
const MIN_VALUE_TEXT_SIZE: f32 = 1. / LABEL_TEXT_SIZE_SCALE;
const MAX_AUTO_TEXT_SIZE: f32 = 4096.;
const AUTO_SIZE_MARGIN_RATIO: f32 = 0.05;

/// Paints a centered label and value using either automatic or fixed sizing.
///
/// `sizing_value` controls automatic width independently from the currently
/// displayed `value`, allowing callers to keep sizing stable across changes.
/// `fill` paints an opaque background over the whole container and switches
/// both lines to black or white, whichever contrasts more with it.
/// `reserve_empty_label` preserves the label line when `label` is empty.
#[expect(clippy::too_many_arguments)]
pub fn show(
    ui: &Ui,
    label: &str,
    value: &str,
    sizing_value: &str,
    fill: Option<Color32>,
    auto_size: bool,
    text_size: i64,
    reserve_empty_label: bool,
) {
    // Resolve the shared text size against the caller's stable sizing input
    let container = ui.max_rect();
    let has_label = reserve_empty_label || !label.is_empty();
    let spacing = if has_label { ui.spacing().item_spacing.y } else { 0. };
    let value_text_size = if auto_size {
        compute_auto_text_size(ui, label, sizing_value, container.size(), spacing, has_label)
    } else {
        text_size as f32
    };

    // Paint the optional background and keep both lines legible against it
    let painter = ui.painter();
    let (label_color, value_color) = match fill {
        Some(fill) => {
            let fill = fill.to_opaque();
            painter.rect_filled(container, 0., fill);
            let foreground = contrasting_text(fill);
            (foreground.gamma_multiply(0.8), foreground)
        }
        None => (ui.visuals().weak_text_color(), ui.visuals().text_color()),
    };

    // Lay out only visible lines using reusable raster sizes for automatic text
    let mut label_galley = has_label.then(|| {
        auto_size::layout(
            ui,
            label.to_owned(),
            ui.app_style().base_font_of(value_text_size * LABEL_TEXT_SIZE_SCALE),
            Align::LEFT,
            auto_size,
        )
    });
    let mut value_galley = auto_size::layout(
        ui,
        value.to_owned(),
        FontId::monospace(value_text_size),
        Align::LEFT,
        auto_size,
    );

    // Correct reference-size rounding against final bounds without another font layout
    if auto_size {
        let label_size = label_galley.as_ref().map_or(Vec2::ZERO, |galley| galley.size());
        let width = label_size.x.max(value_galley.size().x);
        let height = label_size.y + value_galley.size().y;
        let mut scale = 1_f32;
        if width > 0. {
            scale = scale.min(container.width().max(0.) / width);
        }
        if height > 0. {
            scale = scale.min((container.height() - spacing).max(0.) / height);
        }
        let scale = scale.max(MIN_VALUE_TEXT_SIZE / value_text_size);
        label_galley = label_galley.map(|galley| auto_size::scaled(galley, scale));
        value_galley = auto_size::scaled(value_galley, scale);
    }

    // Paint the centered read-only text without creating selectable widgets
    let label_height = label_galley.as_ref().map_or(0., |galley| galley.size().y);
    let total_height = label_height + spacing + value_galley.size().y;
    let top = container.center().y - total_height * 0.5;
    if let Some(label_galley) = label_galley {
        let label_pos = pos2(container.center().x - label_galley.size().x * 0.5, top);
        painter.galley_with_override_text_color(label_pos, label_galley, label_color);
    }
    let value_pos = pos2(
        container.center().x - value_galley.size().x * 0.5,
        top + label_height + spacing,
    );
    painter.galley_with_override_text_color(value_pos, value_galley, value_color);
}

/// Returns black or white text with the higher contrast against an opaque fill.
pub fn contrasting_text(fill: Color32) -> Color32 {
    let linear = |channel: u8| {
        let value = f32::from(channel) / 255.;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance = 0.2126 * linear(fill.r()) + 0.7152 * linear(fill.g()) + 0.0722 * linear(fill.b());
    if luminance > 0.179 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

/// Returns the shaped monospace width in logical points at the shared reference size.
pub fn text_width(ui: &Ui, value: &str) -> f32 {
    auto_size::measure(ui, value, FontId::monospace(auto_size::REFERENCE_SIZE))
        .size()
        .x
}

/// Returns a whole-point value size satisfying the label and value constraints, within size limits.
fn compute_auto_text_size(ui: &Ui, label: &str, value: &str, available: Vec2, spacing: f32, has_label: bool) -> f32 {
    if label.is_empty() && value.is_empty() {
        return DEFAULT_TEXT_SIZE as f32;
    }

    // Measure shaped text once at a small reference size instead of rendering trial sizes
    let label_size = if has_label {
        auto_size::measure(ui, label, ui.app_style().base_font_of(auto_size::REFERENCE_SIZE)).size()
            * (LABEL_TEXT_SIZE_SCALE / auto_size::REFERENCE_SIZE)
    } else {
        Vec2::ZERO
    };
    let value_size =
        auto_size::measure(ui, value, FontId::monospace(auto_size::REFERENCE_SIZE)).size() / auto_size::REFERENCE_SIZE;

    // Solve the tightest width and height constraints inside a uniform margin
    let margin = available.x.min(available.y).max(0.) * AUTO_SIZE_MARGIN_RATIO;
    let fitting = (available - Vec2::splat(2. * margin)).max(Vec2::ZERO);
    let width = label_size.x.max(value_size.x);
    let height = label_size.y + value_size.y;
    let mut size = MAX_AUTO_TEXT_SIZE;
    if width > 0. {
        size = size.min(fitting.x / width);
    }
    if height > 0. {
        size = size.min((fitting.y - spacing).max(0.) / height);
    }
    size.floor().clamp(MIN_VALUE_TEXT_SIZE, MAX_AUTO_TEXT_SIZE)
}
