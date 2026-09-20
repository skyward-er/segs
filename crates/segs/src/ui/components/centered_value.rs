use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use egui::{FontFamily, FontId, Ui, Vec2, pos2, vec2};
use segs_memory::MemoryExt;
use segs_ui::style::CtxStyleExt;

/// Default fixed value text size in logical points.
pub const DEFAULT_TEXT_SIZE: i64 = 32;
/// Preferred width of the text-size setting editor in logical points.
pub const TEXT_SIZE_SETTING_WIDTH: f32 = 96.;
const LABEL_TEXT_SIZE_SCALE: f32 = 0.75;
const MIN_LABEL_TEXT_SIZE: f32 = 1.;
const MIN_VALUE_TEXT_SIZE: f32 = MIN_LABEL_TEXT_SIZE / LABEL_TEXT_SIZE_SCALE;
const MAX_AUTO_TEXT_SIZE: f32 = 4096.;
const TEXT_METRICS_REFERENCE_SIZE: f32 = 100.;
const AUTO_SIZE_MARGIN_RATIO: f32 = 0.05;
const AUTO_SIZE_UPDATE_INTERVAL: Duration = Duration::from_millis(20);
const AUTO_SIZE_CACHE_ID: &str = "centered_value_auto_size";

/// Paints a centered label and value using either automatic or fixed sizing.
///
/// `sizing_value` controls automatic width independently from the currently
/// displayed `value`, allowing callers to keep sizing stable across changes.
/// `reserve_empty_label` preserves the label line when `label` is empty.
pub fn show(
    ui: &Ui,
    label: &str,
    value: &str,
    sizing_value: &str,
    auto_size: bool,
    text_size: i64,
    reserve_empty_label: bool,
) {
    // Resolve the shared text size against the caller's stable sizing input
    let container = ui.max_rect();
    let has_label = reserve_empty_label || !label.is_empty();
    let spacing = if has_label { ui.spacing().item_spacing.y } else { 0. };
    let value_text_size = if auto_size {
        cached_auto_text_size(ui, label, sizing_value, container.size(), spacing, has_label)
    } else {
        text_size as f32
    };

    // Lay out only the visible lines so an empty label consumes no height
    let app_style = ui.app_style();
    let painter = ui.painter();
    let label_color = ui.visuals().weak_text_color();
    let value_color = ui.visuals().text_color();
    let label_galley = has_label.then(|| {
        painter.layout_no_wrap(
            label.to_owned(),
            app_style.base_font_of(value_text_size * LABEL_TEXT_SIZE_SCALE),
            label_color,
        )
    });
    let value_galley = painter.layout_no_wrap(value.to_owned(), monospace_font(value_text_size), value_color);

    // Paint the centered read-only text without creating selectable widgets
    let label_height = label_galley.as_ref().map_or(0., |galley| galley.size().y);
    let total_height = label_height + spacing + value_galley.size().y;
    let top = container.center().y - total_height * 0.5;
    if let Some(label_galley) = label_galley {
        let label_pos = pos2(container.center().x - label_galley.size().x * 0.5, top);
        painter.galley(label_pos, label_galley, label_color);
    }
    let value_pos = pos2(
        container.center().x - value_galley.size().x * 0.5,
        top + label_height + spacing,
    );
    painter.galley(value_pos, value_galley, value_color);
}

/// Returns the rendered monospace width in logical points at the reference size.
pub fn text_width(ui: &Ui, value: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(
            value.to_owned(),
            monospace_font(TEXT_METRICS_REFERENCE_SIZE),
            egui::Color32::WHITE,
        )
        .size()
        .x
}

/// Cached fit result and the inputs that determine whether it can be reused.
#[derive(Clone)]
struct AutoSizeCache {
    available_size: Vec2,
    input_hash: u64,
    text_size: f32,
    updated_at: f64,
    /// Font environment used for this fit calculation.
    fonts: Arc<FontMetrics>,
}

/// Returns a cached automatic size while debouncing widget rectangle changes.
fn cached_auto_text_size(
    ui: &Ui,
    label: &str,
    value: &str,
    available_size: Vec2,
    spacing: f32,
    has_label: bool,
) -> f32 {
    let cache_id = ui.id().with(AUTO_SIZE_CACHE_ID);
    let now = ui.input(|input| input.time);
    let fonts = font_metrics(ui);

    // Hash every stable input that changes text measurements
    let mut hasher = DefaultHasher::new();
    label.hash(&mut hasher);
    value.hash(&mut hasher);
    spacing.to_bits().hash(&mut hasher);
    has_label.hash(&mut hasher);
    let input_hash = hasher.finish();

    // Reuse the cached result or briefly hold it while a resize is in progress
    if let Some(cache) = ui.mem().get_temp::<AutoSizeCache>(cache_id)
        && cache.input_hash == input_hash
        && cache.fonts == fonts
    {
        if cache.available_size == available_size {
            return cache.text_size;
        }

        let elapsed = now - cache.updated_at;
        let update_interval = AUTO_SIZE_UPDATE_INTERVAL.as_secs_f64();
        if elapsed < update_interval {
            ui.ctx()
                .request_repaint_after(Duration::from_secs_f64(update_interval - elapsed));
            return cache.text_size;
        }
    }

    // Recompute after relevant inputs change or the resize debounce expires
    let text_size = compute_auto_text_size(ui, label, value, available_size, spacing, has_label);
    ui.mem().insert_temp(
        cache_id,
        AutoSizeCache {
            available_size,
            input_hash,
            text_size,
            updated_at: now,
            fonts,
        },
    );
    text_size
}

/// Computes a whole-point value text size that fits both lines inside the margin.
fn compute_auto_text_size(
    ui: &Ui,
    label: &str,
    value: &str,
    available_size: Vec2,
    spacing: f32,
    has_label: bool,
) -> f32 {
    if label.is_empty() && value.is_empty() {
        return DEFAULT_TEXT_SIZE as f32;
    }

    // Measure only reserved lines with the fonts used during painting
    let app_style = ui.app_style();
    let painter = ui.painter();
    let text_color = ui.visuals().text_color();
    let label_size_per_point = if has_label {
        painter
            .layout_no_wrap(
                label.to_owned(),
                app_style.base_font_of(TEXT_METRICS_REFERENCE_SIZE),
                text_color,
            )
            .size()
            / TEXT_METRICS_REFERENCE_SIZE
    } else {
        Vec2::ZERO
    };
    let value_size_per_point = painter
        .layout_no_wrap(
            value.to_owned(),
            monospace_font(TEXT_METRICS_REFERENCE_SIZE),
            text_color,
        )
        .size()
        / TEXT_METRICS_REFERENCE_SIZE;

    // Scale one uniform margin from the widget's shorter edge
    let margin = available_size.x.min(available_size.y) * AUTO_SIZE_MARGIN_RATIO;
    let fitting_size = vec2(
        (available_size.x - margin * 2.).max(0.),
        (available_size.y - margin * 2.).max(0.),
    );

    // Solve each fit constraint and keep the tightest upper bound
    let mut value_text_size = MAX_AUTO_TEXT_SIZE;
    if label_size_per_point.x > 0. {
        let label_width_limit = fitting_size.x / (label_size_per_point.x * LABEL_TEXT_SIZE_SCALE);
        value_text_size = value_text_size.min(label_width_limit);
    }
    if value_size_per_point.x > 0. {
        value_text_size = value_text_size.min(fitting_size.x / value_size_per_point.x);
    }

    let combined_height_per_point = label_size_per_point.y * LABEL_TEXT_SIZE_SCALE + value_size_per_point.y;
    if combined_height_per_point > 0. {
        value_text_size = value_text_size.min((fitting_size.y - spacing) / combined_height_per_point);
    }

    value_text_size.floor().clamp(MIN_VALUE_TEXT_SIZE, MAX_AUTO_TEXT_SIZE)
}

/// Returns an equal-width font so changing values do not shift their center.
fn monospace_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

/// Returns a shared snapshot of the active font inputs, replacing it on changes.
/// Cloning the snapshot retains font data through reference-counted ownership.
pub fn font_metrics(ui: &Ui) -> Arc<FontMetrics> {
    let id = egui::Id::new("centered_value_font_metrics");
    let cached = ui.ctx().data(|data| data.get_temp::<Arc<FontMetrics>>(id));
    let pixels_per_point = ui.ctx().pixels_per_point();
    let label_font = ui.app_style().base_font_of(TEXT_METRICS_REFERENCE_SIZE);
    let value_font = monospace_font(TEXT_METRICS_REFERENCE_SIZE);

    // Compare shared definitions without copying font bytes or laying out text
    let metrics = ui.fonts(|fonts| {
        if let Some(cached) = cached
            && cached.pixels_per_point == pixels_per_point
            && cached.label_font == label_font
            && cached.value_font == value_font
            && cached.options == *fonts.options()
            && cached.definitions == *fonts.definitions()
        {
            return cached;
        }
        Arc::new(FontMetrics {
            pixels_per_point,
            label_font,
            value_font,
            options: *fonts.options(),
            definitions: fonts.definitions().clone(),
        })
    });
    ui.ctx().data_mut(|data| data.insert_temp(id, metrics.clone()));
    metrics
}

/// Active font environment shared by measurement and fit caches.
#[derive(PartialEq)]
pub struct FontMetrics {
    /// Effective physical pixels per logical point, including zoom.
    pixels_per_point: f32,
    /// Label font at the measurement reference size.
    label_font: FontId,
    /// Value font at the measurement reference size.
    value_font: FontId,
    /// Active text rasterization and layout options.
    options: egui::epaint::text::TextOptions,
    /// Installed font data and ordered family fallbacks.
    definitions: egui::FontDefinitions,
}
