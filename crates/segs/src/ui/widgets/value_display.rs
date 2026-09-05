use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};

use egui::{FontFamily, FontId, Ui, Vec2, pos2, vec2};
use segs_memory::MemoryExt;
use segs_ui::style::CtxStyleExt;
use serde::{Deserialize, Serialize};

use crate::{
    dataflow::{StreamKey, store::DataStore},
    ui::{
        widget_settings::{WidgetDataSetting, WidgetSetting},
        widgets::WidgetTrait,
    },
};

const DEFAULT_TEXT_SIZE: f32 = 32.;
const LABEL_TEXT_SIZE_SCALE: f32 = 0.75;
const MIN_LABEL_TEXT_SIZE: f32 = 1.;
const MIN_VALUE_TEXT_SIZE: f32 = MIN_LABEL_TEXT_SIZE / LABEL_TEXT_SIZE_SCALE;
const MAX_AUTO_TEXT_SIZE: f32 = 4096.;
const TEXT_METRICS_REFERENCE_SIZE: f32 = 100.;
const AUTO_SIZE_MARGIN_RATIO: f32 = 0.05;
const AUTO_SIZE_UPDATE_INTERVAL: Duration = Duration::from_millis(20);
const AUTO_SIZE_CACHE_ID: &str = "value_display_auto_size";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueDisplayWidget {
    label: String,
    stream: Option<StreamKey>,
    auto_size: bool,
    text_size: String,
}

impl Default for ValueDisplayWidget {
    /// Creates an unconfigured value display.
    fn default() -> Self {
        Self {
            label: "Value".to_owned(),
            stream: None,
            auto_size: true,
            text_size: DEFAULT_TEXT_SIZE.to_string(),
        }
    }
}

impl WidgetTrait for ValueDisplayWidget {
    fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        let value = self.value_text(data_store);

        let container = ui.max_rect();
        let spacing = ui.spacing().item_spacing.y;
        let value_text_size = if self.auto_size {
            cached_auto_text_size(ui, &self.label, &value, container.size(), spacing)
        } else {
            configured_text_size(&self.text_size)
        };
        let label_text_size = value_text_size * LABEL_TEXT_SIZE_SCALE;

        let app_style = ui.app_style();
        let painter = ui.painter();
        let label_color = ui.visuals().weak_text_color();
        let value_color = ui.visuals().text_color();
        let label_galley =
            painter.layout_no_wrap(self.label.clone(), app_style.base_font_of(label_text_size), label_color);
        let value_galley = painter.layout_no_wrap(value, monospace_font(value_text_size), value_color);

        let total_height = label_galley.size().y + spacing + value_galley.size().y;
        let top = container.center().y - total_height * 0.5;
        let label_pos = pos2(container.center().x - label_galley.size().x * 0.5, top);
        let value_pos = pos2(
            container.center().x - value_galley.size().x * 0.5,
            top + label_galley.size().y + spacing,
        );

        // Paint the centered read-only text without creating selectable widgets
        painter.galley(label_pos, label_galley, label_color);
        painter.galley(value_pos, value_galley, value_color);
    }

    fn data_settings(&mut self) -> Vec<WidgetDataSetting<'_>> {
        vec![WidgetDataSetting::single_stream("stream", "Stream", &mut self.stream)]
    }

    fn settings(&mut self) -> Vec<WidgetSetting<'_>> {
        let show_text_size = !self.auto_size;
        let mut settings = vec![
            WidgetSetting::text_box("label", "Label", &mut self.label),
            WidgetSetting::checkbox("auto_size", "Auto size", &mut self.auto_size),
        ];

        if show_text_size {
            settings.push(WidgetSetting::text_box("text_size", "Text size", &mut self.text_size));
        }

        settings
    }

    /// Returns the widget's gallery name.
    fn display_name(&self) -> &'static str {
        "Value display"
    }
}

impl ValueDisplayWidget {
    fn value_text(&self, data_store: &DataStore) -> String {
        let Some(stream) = self.stream else {
            return "No stream".to_owned();
        };

        data_store
            .latest(stream)
            .map_or_else(|| "No data".to_owned(), |(_, value)| value.to_string())
    }
}

#[derive(Clone, Copy)]
struct AutoSizeCache {
    available_size: Vec2,
    input_hash: u64,
    text_size: f32,
    updated_at: f64,
}

/// Parses a configured text size, falling back when the text box contains an invalid value.
fn configured_text_size(text_size: &str) -> f32 {
    text_size
        .parse::<f32>()
        .ok()
        .filter(|size| size.is_finite() && *size >= MIN_VALUE_TEXT_SIZE)
        .unwrap_or(DEFAULT_TEXT_SIZE)
}

/// Returns a cached auto size, debouncing recomputation while the widget rect changes.
fn cached_auto_text_size(ui: &Ui, label: &str, value: &str, available_size: Vec2, spacing: f32) -> f32 {
    let cache_id = ui.id().with(AUTO_SIZE_CACHE_ID);
    let now = ui.input(|input| input.time);

    // Build a hash to detect changes in input that require a recomputation
    let mut hasher = DefaultHasher::new();
    label.hash(&mut hasher);
    value.hash(&mut hasher);
    spacing.to_bits().hash(&mut hasher);
    ui.ctx().pixels_per_point().to_bits().hash(&mut hasher);
    let input_hash = hasher.finish();

    // Reuse the cached result or briefly hold it while a resize is in progress
    if let Some(cache) = ui.mem().get_temp::<AutoSizeCache>(cache_id)
        && cache.input_hash == input_hash
    {
        if cache.available_size == available_size {
            return cache.text_size;
        }

        let elapsed = now - cache.updated_at;
        let update_interval = AUTO_SIZE_UPDATE_INTERVAL.as_secs_f64();
        if elapsed < update_interval {
            // Schedule the final resize update in case no more input events arrive
            ui.ctx()
                .request_repaint_after(Duration::from_secs_f64(update_interval - elapsed));
            return cache.text_size;
        }
    }

    // Recompute after relevant inputs change or the resize debounce expires
    let text_size = compute_auto_text_size(ui, label, value, available_size, spacing);
    ui.mem().insert_temp(
        cache_id,
        AutoSizeCache {
            available_size,
            input_hash,
            text_size,
            updated_at: now,
        },
    );
    text_size
}

/// Computes a whole-point value text size that fits both lines inside the widget margin.
fn compute_auto_text_size(ui: &Ui, label: &str, value: &str, available_size: Vec2, spacing: f32) -> f32 {
    if label.is_empty() && value.is_empty() {
        return DEFAULT_TEXT_SIZE;
    }

    let app_style = ui.app_style();
    let painter = ui.painter();
    let text_color = ui.visuals().text_color();

    // Measure both lines with the same fonts used during painting.
    let label_size_per_point = painter
        .layout_no_wrap(
            label.to_owned(),
            app_style.base_font_of(TEXT_METRICS_REFERENCE_SIZE),
            text_color,
        )
        .size()
        / TEXT_METRICS_REFERENCE_SIZE;
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
        let height_limit = (fitting_size.y - spacing) / combined_height_per_point;
        value_text_size = value_text_size.min(height_limit);
    }

    value_text_size.floor().clamp(MIN_VALUE_TEXT_SIZE, MAX_AUTO_TEXT_SIZE)
}

/// Uses equal-width glyphs so changing numeric values does not shift the display width.
fn monospace_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}
