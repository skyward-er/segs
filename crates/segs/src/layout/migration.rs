use serde::Deserialize;
use serde_json::Value;

use super::{CURRENT_LAYOUT_SCHEMA, Layout};

const FIRST_LAYOUT_SCHEMA: u32 = 1;

/// A current layout paired with the schema version found in its persisted file.
#[derive(Debug)]
pub(super) struct MigratedLayout {
    /// Layout converted to the current in-memory representation.
    pub layout: Layout,
    /// Schema version read before any migrations were applied.
    pub persisted_schema_version: u32,
}

/// A failure encountered while decoding or migrating a persisted layout.
#[derive(Debug)]
pub(super) enum LayoutMigrationError {
    /// The layout does not contain valid JSON for a supported schema.
    Json(serde_json::Error),
    /// The layout uses a schema version for which no migration path exists.
    UnsupportedSchema(String),
}

/// Migrates serialized layout JSON to the current schema and deserializes it.
///
/// Returns the current layout together with the schema version originally stored
/// in the JSON. Invalid JSON and schemas without a complete migration path return
/// a [`LayoutMigrationError`].
pub(super) fn migrate(bytes: &[u8]) -> Result<MigratedLayout, LayoutMigrationError> {
    // Parse the document and read the migration dispatch fields
    let mut value = serde_json::from_slice::<Value>(bytes).map_err(LayoutMigrationError::Json)?;
    let header = LayoutHeader::deserialize(&value).map_err(LayoutMigrationError::Json)?;
    let persisted_schema_version = header.schema_version;

    // Apply every adjacent migration until the current schema is reached
    let mut schema_version = persisted_schema_version;
    while schema_version != CURRENT_LAYOUT_SCHEMA {
        schema_version = match schema_version {
            FIRST_LAYOUT_SCHEMA => v1::migrate(&mut value),
            v1::NEXT_SCHEMA_VERSION => v2::migrate(&mut value),
            v2::NEXT_SCHEMA_VERSION => v3::migrate(&mut value),
            _ => return Err(LayoutMigrationError::UnsupportedSchema(header.slug)),
        };
    }

    // Deserialize only after the document has the complete current shape
    let layout = serde_json::from_value(value).map_err(LayoutMigrationError::Json)?;
    Ok(MigratedLayout {
        layout,
        persisted_schema_version,
    })
}

/// Fields required to select a layout migration path and report failures.
#[derive(Deserialize)]
struct LayoutHeader {
    /// Schema version stored in the layout file.
    schema_version: u32,
    /// Stable layout identifier used in migration errors.
    slug: String,
}

/// Migration from layout schema v1 to v2.
mod v1 {
    use serde_json::{Map, Value};

    pub const NEXT_SCHEMA_VERSION: u32 = 2;

    /// Adds the plot settings introduced in v2 and advances the version marker.
    ///
    /// Returns the next schema version.
    pub(super) fn migrate(layout: &mut Value) -> u32 {
        // Fill missing plot settings without replacing values already persisted
        if let Some(widgets) = layout.get_mut("widgets").and_then(Value::as_array_mut) {
            for widget in widgets {
                let Some(plot) = widget
                    .get_mut("variant")
                    .and_then(|variant| variant.get_mut("Plot"))
                    .and_then(Value::as_object_mut)
                else {
                    continue;
                };
                insert_plot_defaults(plot);
            }
        }

        // Mark the document ready for the next migration or current deserialization
        if let Some(layout) = layout.as_object_mut() {
            layout.insert("schema_version".to_owned(), NEXT_SCHEMA_VERSION.into());
        }

        NEXT_SCHEMA_VERSION
    }

    /// Inserts settings absent from the v1 plot representation.
    fn insert_plot_defaults(plot: &mut Map<String, Value>) {
        plot.entry("history_seconds")
            .or_insert_with(|| Value::String("90".to_owned()));
        plot.entry("auto_y_bounds").or_insert(Value::Bool(true));
        plot.entry("y_min").or_insert_with(|| Value::String("0".to_owned()));
        plot.entry("y_max").or_insert_with(|| Value::String("1".to_owned()));
    }
}

/// Migration from layout schema v2 to v3.
mod v2 {
    use serde_json::{Map, Number, Value};

    pub const NEXT_SCHEMA_VERSION: u32 = 3;
    const DEFAULT_VALUE_DISPLAY_TEXT_SIZE: i64 = 32;
    const DEFAULT_MESSAGE_VIEWER_TEXT_SIZE: i64 = 12;
    const DEFAULT_STALE_AFTER_SECONDS: f64 = 5.;
    const DEFAULT_HISTORY_SECONDS: f64 = 60.;
    const DEFAULT_Y_MIN: f64 = 0.;
    const DEFAULT_Y_MAX: f64 = 1.;
    const MIN_VALUE_DISPLAY_TEXT_SIZE: f32 = 1. / 0.75;

    /// Converts numeric widget strings to native JSON numbers and advances the version marker.
    ///
    /// Returns the next schema version.
    pub(super) fn migrate(layout: &mut Value) -> u32 {
        // Convert each supported widget without allocating a second document
        if let Some(widgets) = layout.get_mut("widgets").and_then(Value::as_array_mut) {
            for widget in widgets {
                let Some(variant) = widget.get_mut("variant").and_then(Value::as_object_mut) else {
                    continue;
                };

                if let Some(value_display) = variant.get_mut("ValueDisplay").and_then(Value::as_object_mut) {
                    migrate_text_size(value_display, DEFAULT_VALUE_DISPLAY_TEXT_SIZE, 2, |size| {
                        size >= MIN_VALUE_DISPLAY_TEXT_SIZE
                    });
                }
                if let Some(message_viewer) = variant.get_mut("MessageViewer").and_then(Value::as_object_mut) {
                    migrate_text_size(message_viewer, DEFAULT_MESSAGE_VIEWER_TEXT_SIZE, 1, |size| size > 0.);
                    migrate_positive_float(message_viewer, "stale_after", DEFAULT_STALE_AFTER_SECONDS);
                }
                if let Some(plot) = variant.get_mut("Plot").and_then(Value::as_object_mut) {
                    migrate_plot(plot);
                }
            }
        }

        // Mark the document ready for current deserialization
        if let Some(layout) = layout.as_object_mut() {
            layout.insert("schema_version".to_owned(), NEXT_SCHEMA_VERSION.into());
        }

        NEXT_SCHEMA_VERSION
    }

    /// Converts one text-size string to its nearest supported whole point.
    fn migrate_text_size(widget: &mut Map<String, Value>, default: i64, minimum: i64, valid: impl FnOnce(f32) -> bool) {
        // Preserve the old renderer's validity rules before rounding the value
        let size = widget
            .get("text_size")
            .and_then(Value::as_str)
            .and_then(|size| size.parse::<f32>().ok())
            .filter(|size| size.is_finite() && valid(*size))
            .map_or(default, |size| (size.round() as i64).clamp(minimum, i64::MAX));
        widget.insert("text_size".to_owned(), size.into());
    }

    /// Converts a positive floating-point string or substitutes its effective default.
    fn migrate_positive_float(widget: &mut Map<String, Value>, key: &str, default: f64) {
        // Retain only values previously accepted by the renderer
        let value = finite_string(widget.get(key))
            .filter(|value| *value > 0.)
            .unwrap_or(default);
        insert_f64(widget, key, value);
    }

    /// Converts plot duration and bound strings while preserving effective automatic bounds.
    fn migrate_plot(plot: &mut Map<String, Value>) {
        // Normalize the positive history duration independently
        migrate_positive_float(plot, "history_seconds", DEFAULT_HISTORY_SECONDS);

        // Convert endpoints and remember whether the old fixed bounds were usable
        let parsed_y_min = finite_string(plot.get("y_min"));
        let parsed_y_max = finite_string(plot.get("y_max"));
        let y_min = parsed_y_min.unwrap_or(DEFAULT_Y_MIN);
        let y_max = parsed_y_max.unwrap_or(DEFAULT_Y_MAX);
        insert_f64(plot, "y_min", y_min);
        insert_f64(plot, "y_max", y_max);

        // Keep malformed or non-increasing fixed bounds in their prior automatic mode
        let fixed_bounds_invalid = parsed_y_min.is_none() || parsed_y_max.is_none() || y_min >= y_max;
        if plot.get("auto_y_bounds").and_then(Value::as_bool) == Some(false) && fixed_bounds_invalid {
            plot.insert("auto_y_bounds".to_owned(), Value::Bool(true));
        }
    }

    /// Parses a finite float from an optional JSON string.
    fn finite_string(value: Option<&Value>) -> Option<f64> {
        value
            .and_then(Value::as_str)
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite())
    }

    /// Inserts a finite `f64` as a JSON number.
    fn insert_f64(object: &mut Map<String, Value>, key: &str, value: f64) {
        let number = Number::from_f64(value).expect("migration values are finite");
        object.insert(key.to_owned(), Value::Number(number));
    }
}

/// Migration from layout schema v3 to v4.
mod v3 {
    use egui::Color32;
    use serde_json::{Map, Number, Value};

    /// Schema version produced by this migration.
    pub const NEXT_SCHEMA_VERSION: u32 = 4;
    const DEFAULT_LINE_WIDTH: f64 = 1.5;

    /// Adds configurable plot line appearance and advances the version marker.
    ///
    /// Returns the next schema version after inserting defaults where needed.
    pub(super) fn migrate(layout: &mut Value) -> u32 {
        // Fill missing line settings without replacing values already persisted
        if let Some(widgets) = layout.get_mut("widgets").and_then(Value::as_array_mut) {
            for widget in widgets {
                let Some(plot) = widget
                    .get_mut("variant")
                    .and_then(|variant| variant.get_mut("Plot"))
                    .and_then(Value::as_object_mut)
                else {
                    continue;
                };
                insert_line_defaults(plot);
            }
        }

        // Mark the document ready for current deserialization
        if let Some(layout) = layout.as_object_mut() {
            layout.insert("schema_version".to_owned(), NEXT_SCHEMA_VERSION.into());
        }

        NEXT_SCHEMA_VERSION
    }

    /// Inserts the line appearance used before it became configurable.
    fn insert_line_defaults(plot: &mut Map<String, Value>) {
        // Serialize the native color type so the migration matches its current wire format
        plot.entry("line_color")
            .or_insert_with(|| serde_json::to_value(Color32::BLUE).expect("default line color is serializable"));
        plot.entry("line_width").or_insert_with(|| {
            Value::Number(Number::from_f64(DEFAULT_LINE_WIDTH).expect("default line width is finite"))
        });
    }
}

#[cfg(test)]
mod tests {
    use egui::{Rect, pos2, vec2};

    use super::*;
    use crate::ui::{
        grid::GRect,
        widgets::{MessageViewerWidget, PlotWidget, ValueDisplayWidget},
    };

    #[test]
    fn progressively_migrates_v1_layout_to_current_schema() {
        // Build a v1-shaped document with the plot settings that predate v2 omitted
        let mut layout = Layout::empty("Legacy".into(), "legacy-deadbeef".into());
        let rect = GRect::new(Rect::from_min_size(pos2(1., 2.), vec2(3., 4.)));
        layout.add_widget(PlotWidget::default().into(), rect);
        layout.add_widget(PlotWidget::default().into(), rect);
        layout.add_widget(MessageViewerWidget::default().into(), rect);
        layout.add_widget(ValueDisplayWidget::default().into(), rect);
        let mut value = serde_json::to_value(&layout).unwrap();
        value["schema_version"] = FIRST_LAYOUT_SCHEMA.into();

        // Exercise both v1 plot defaults and v2 numeric normalization in one migration chain
        let plot = value
            .pointer_mut("/widgets/0/variant/Plot")
            .unwrap()
            .as_object_mut()
            .unwrap();
        plot.remove("history_seconds");
        plot.remove("auto_y_bounds");
        plot.remove("y_min");
        plot.remove("y_max");
        plot.remove("line_color");
        plot.remove("line_width");
        let invalid_plot = value
            .pointer_mut("/widgets/1/variant/Plot")
            .unwrap()
            .as_object_mut()
            .unwrap();
        invalid_plot.insert("history_seconds".to_owned(), Value::String("invalid".to_owned()));
        invalid_plot.insert("auto_y_bounds".to_owned(), Value::Bool(false));
        invalid_plot.insert("y_min".to_owned(), Value::String("invalid".to_owned()));
        invalid_plot.insert("y_max".to_owned(), Value::String("4".to_owned()));
        invalid_plot.insert(
            "line_color".to_owned(),
            serde_json::to_value(egui::Color32::RED).unwrap(),
        );
        invalid_plot.insert("line_width".to_owned(), 2.5.into());
        value["widgets"][2]["variant"]["MessageViewer"]["text_size"] = Value::String("18.6".to_owned());
        value["widgets"][2]["variant"]["MessageViewer"]["stale_after"] = Value::String("invalid".to_owned());
        value["widgets"][3]["variant"]["ValueDisplay"]["text_size"] = Value::String("1.5".to_owned());

        // Run the production dispatcher and inspect the fully migrated wire shape
        let bytes = serde_json::to_vec(&value).unwrap();
        let migrated = migrate(&bytes).unwrap();
        let migrated_value = serde_json::to_value(&migrated.layout).unwrap();
        let plot = migrated_value.pointer("/widgets/0/variant/Plot").unwrap();
        assert_eq!(migrated.persisted_schema_version, FIRST_LAYOUT_SCHEMA);
        assert_eq!(migrated.layout.schema_version, CURRENT_LAYOUT_SCHEMA);
        assert_eq!(plot["history_seconds"], 90.);
        assert_eq!(plot["auto_y_bounds"], true);
        assert_eq!(plot["y_min"], 0.);
        assert_eq!(plot["y_max"], 1.);
        assert_eq!(plot["line_color"], serde_json::to_value(egui::Color32::BLUE).unwrap());
        assert_eq!(plot["line_width"], 1.);
        let invalid_plot = migrated_value.pointer("/widgets/1/variant/Plot").unwrap();
        assert_eq!(invalid_plot["history_seconds"], 90.);
        assert_eq!(invalid_plot["auto_y_bounds"], true);
        assert_eq!(invalid_plot["y_min"], 0.);
        assert_eq!(invalid_plot["y_max"], 4.);
        assert_eq!(
            invalid_plot["line_color"],
            serde_json::to_value(egui::Color32::RED).unwrap()
        );
        assert_eq!(invalid_plot["line_width"], 2.5);
        let message_viewer = migrated_value.pointer("/widgets/2/variant/MessageViewer").unwrap();
        assert_eq!(message_viewer["text_size"], 19);
        assert_eq!(message_viewer["stale_after"], 5.);
        let value_display = migrated_value.pointer("/widgets/3/variant/ValueDisplay").unwrap();
        assert_eq!(value_display["text_size"], 2);
    }
}
