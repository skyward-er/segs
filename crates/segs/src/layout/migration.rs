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

#[cfg(test)]
mod tests {
    use egui::{Rect, pos2, vec2};

    use super::*;
    use crate::ui::{grid::GRect, widgets::PlotWidget};

    #[test]
    fn progressively_migrates_v1_layout_to_current_schema() {
        // Build a v1-shaped document with the plot settings that predate v2 omitted
        let mut layout = Layout::empty("Legacy".into(), "legacy-deadbeef".into());
        layout.add_widget(
            PlotWidget::default().into(),
            GRect::new(Rect::from_min_size(pos2(1., 2.), vec2(3., 4.))),
        );
        let mut value = serde_json::to_value(&layout).unwrap();
        value["schema_version"] = FIRST_LAYOUT_SCHEMA.into();
        let plot = value
            .pointer_mut("/widgets/0/variant/Plot")
            .unwrap()
            .as_object_mut()
            .unwrap();
        plot.remove("history_seconds");
        plot.remove("auto_y_bounds");
        plot.remove("y_min");
        plot.remove("y_max");

        // Run the production dispatcher and inspect the fully migrated wire shape
        let bytes = serde_json::to_vec(&value).unwrap();
        let migrated = migrate(&bytes).unwrap();
        let migrated_value = serde_json::to_value(&migrated.layout).unwrap();
        let plot = migrated_value.pointer("/widgets/0/variant/Plot").unwrap();
        assert_eq!(migrated.persisted_schema_version, FIRST_LAYOUT_SCHEMA);
        assert_eq!(migrated.layout.schema_version, CURRENT_LAYOUT_SCHEMA);
        assert_eq!(plot["history_seconds"], "90");
        assert_eq!(plot["auto_y_bounds"], true);
        assert_eq!(plot["y_min"], "0");
        assert_eq!(plot["y_max"], "1");
    }
}
