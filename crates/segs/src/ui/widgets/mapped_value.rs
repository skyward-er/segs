use std::{
    collections::{HashSet, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    sync::Arc,
};

use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::{
    dataflow::{DataStream, StreamKey, store::DataStore},
    ui::{
        components::{
            centered_value,
            mapping_table::{self, IntegerTextMapping},
        },
        widget_settings::{WidgetDataSetting, WidgetSetting},
        widgets::WidgetTrait,
    },
};

/// Displays the latest integer stream value through a user-defined text mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MappedValueWidget {
    /// Optional label displayed above the mapped value.
    label: String,
    /// Integer stream whose latest value is translated.
    stream: Option<StreamKey>,
    /// Ordered integer-to-text translations.
    #[serde(deserialize_with = "mapping_table::deserialize_mappings")]
    mappings: Vec<IntegerTextMapping>,
    /// Whether text is automatically sized to fit the widget.
    auto_size: bool,
    /// Fixed value text size in logical points when automatic sizing is disabled.
    text_size: i64,
}

impl Default for MappedValueWidget {
    /// Creates an unconfigured mapped value display with no visible label.
    fn default() -> Self {
        Self {
            label: String::new(),
            stream: None,
            mappings: Vec::new(),
            auto_size: true,
            text_size: centered_value::DEFAULT_TEXT_SIZE,
        }
    }
}

impl WidgetTrait for MappedValueWidget {
    /// Configures sample mappings and the integer stream on a gallery-only clone.
    fn configure_preview(&mut self, preview: &crate::dataflow::preview::PreviewContext) {
        self.stream = Some(preview.integer_stream);
        self.mappings = [("0", "INIT"), ("1", "INIT ERROR"), ("2", "OK")]
            .into_iter()
            .map(|(value, text)| IntegerTextMapping {
                value: value.to_owned(),
                text: text.to_owned(),
            })
            .collect();
    }

    fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        // Resolve the current output and whether it uses a configured mapping
        let (value, is_mapped) = self.value_text(data_store);
        let sizing_value = if self.auto_size && is_mapped {
            self.widest_mapping(ui).unwrap_or(value.as_str())
        } else {
            value.as_str()
        };

        // Keep mapped-state transitions sized for the widest configured output
        centered_value::show(
            ui,
            &self.label,
            &value,
            sizing_value,
            self.auto_size,
            self.text_size,
            false,
        );
    }

    fn data_settings(&mut self) -> Vec<WidgetDataSetting<'_>> {
        vec![WidgetDataSetting::integer_stream("stream", "Stream", &mut self.stream)]
    }

    fn settings(&mut self) -> Vec<WidgetSetting<'_>> {
        let show_text_size = !self.auto_size;
        let mut settings = vec![
            WidgetSetting::text_box("label", "Label", &mut self.label),
            WidgetSetting::integer_text_mappings("mappings", "Mapping", &mut self.mappings),
            WidgetSetting::checkbox("auto_size", "Auto size", &mut self.auto_size),
        ];

        if show_text_size {
            settings.push(WidgetSetting::integer(
                "text_size",
                "Text size",
                &mut self.text_size,
                2..=500,
                Some(1),
                Some(centered_value::TEXT_SIZE_SETTING_WIDTH),
            ));
        }

        settings
    }

    fn display_name(&self) -> &'static str {
        "Mapped value"
    }

    /// Normalizes mapping drafts before layout persistence.
    fn prepare_for_save(&mut self) {
        mapping_table::prepare_for_save(&mut self.mappings);
    }
}

impl MappedValueWidget {
    /// Returns the current displayed text and whether a configured row matched.
    fn value_text(&self, data_store: &DataStore) -> (String, bool) {
        let Some(stream_key) = self.stream else {
            return ("No stream".to_owned(), false);
        };
        let Some(stream) = data_store.stream(stream_key) else {
            return ("No data".to_owned(), false);
        };
        let DataStream::I64(points) = stream else {
            return ("Expected integer stream".to_owned(), false);
        };
        let Some(value) = points.last().map(|point| point.value) else {
            return ("No data".to_owned(), false);
        };

        // Honor persisted row order so malformed duplicate mappings are deterministic
        if let Some(mapping) = self
            .mappings
            .iter()
            .find(|mapping| mapping.parsed_value() == Some(value))
        {
            (mapping.text.clone(), true)
        } else {
            (format!("Unknown ({value})"), false)
        }
    }

    /// Returns the widest valid output, or `None` when no valid mapping exists.
    /// Measurements are cached until ordered mapping contents or fonts change.
    fn widest_mapping(&self, ui: &Ui) -> Option<&str> {
        // Fingerprint editable contents without parsing or measuring unchanged rows
        let mut hasher = DefaultHasher::new();
        self.mappings.len().hash(&mut hasher);
        for mapping in &self.mappings {
            mapping.value.hash(&mut hasher);
            mapping.text.hash(&mut hasher);
        }
        let input_hash = hasher.finish();
        let fonts = centered_value::font_metrics(ui);
        let cache_id = ui.id().with("mapped_value_widest");
        if let Some(cache) = ui.ctx().data(|data| data.get_temp::<WidestMappingCache>(cache_id))
            && cache.input_hash == input_hash
            && cache.fonts == fonts
        {
            return cache.index.map(|index| self.mappings[index].text.as_str());
        }

        // Validate and measure each first-occurrence output once on a cache miss
        let mut seen = HashSet::new();
        let mut widest = None;
        let mut widest_width = f32::NEG_INFINITY;
        for (index, mapping) in self.mappings.iter().enumerate() {
            let Some(value) = mapping.parsed_value() else { continue };
            if !seen.insert(value) {
                continue;
            }
            let width = centered_value::text_width(ui, &mapping.text);
            if width > widest_width {
                widest = Some(index);
                widest_width = width;
            }
        }
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                cache_id,
                WidestMappingCache {
                    input_hash,
                    fonts,
                    index: widest,
                },
            );
        });
        widest.map(|index| self.mappings[index].text.as_str())
    }
}

/// Transient widest-output selection, independent of the current stream value.
#[derive(Clone)]
struct WidestMappingCache {
    /// Fingerprint of ordered raw mapping keys and outputs.
    input_hash: u64,
    /// Font inputs used to measure the outputs.
    fonts: Arc<centered_value::FontMetrics>,
    /// Winning row, or `None` when all rows are invalid.
    index: Option<usize>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::dataflow::{DataValue, preview::PreviewContext};

    use super::*;

    #[test]
    fn reload_sorts_numeric_keys_stably_and_save_round_trips_numbers() {
        // Load through the widget so the field's deserialization hook is exercised
        let mut widget: MappedValueWidget = serde_json::from_value(json!({
            "label": "State",
            "stream": null,
            "auto_size": true,
            "text_size": 32,
            "mappings": [
                {"value": 10, "text": "ten"},
                {"value": "bad", "text": "invalid first"},
                {"value": "02", "text": "first two"},
                {"value": -2, "text": "negative"},
                {"value": 2, "text": "second two"},
                {"value": "", "text": "invalid second"}
            ]
        }))
        .unwrap();
        let rows: Vec<_> = widget
            .mappings
            .iter()
            .map(|row| (row.value.as_str(), row.text.as_str()))
            .collect();
        assert_eq!(
            rows,
            [
                ("-2", "negative"),
                ("02", "first two"),
                ("2", "second two"),
                ("10", "ten"),
                ("bad", "invalid first"),
                ("", "invalid second"),
            ]
        );

        // Drafts cannot serialize until the widget's save hook normalizes them
        assert!(serde_json::to_value(&widget).is_err());
        widget.prepare_for_save();
        let saved = serde_json::to_value(&widget).unwrap();
        assert_eq!(
            saved["mappings"],
            json!([
                {"value": -2, "text": "negative"},
                {"value": 2, "text": "first two"},
                {"value": 10, "text": "ten"}
            ])
        );
        let restored: MappedValueWidget = serde_json::from_value(saved).unwrap();
        assert_eq!(restored, widget);
    }

    #[test]
    fn lookup_ignores_invalid_drafts_and_preserves_first_match_across_save() {
        // Capture an isolated sample once so assertions do not depend on wall-clock timing
        let mut preview = PreviewContext::new();
        let stream = preview.integer_stream;
        let store = preview.data_store();
        let Some((_, DataValue::I64(value))) = store.latest(stream) else {
            panic!("integer preview must contain a sample");
        };
        let mut widget = MappedValueWidget {
            stream: Some(stream),
            mappings: [
                ("".to_owned(), "invalid"),
                ("-".to_owned(), "incomplete"),
                (format!(" {value} "), "first"),
                (value.to_string(), "duplicate"),
            ]
            .into_iter()
            .map(|(value, text)| IntegerTextMapping {
                value,
                text: text.to_owned(),
            })
            .collect(),
            ..Default::default()
        };

        // Lookup and persistence must agree on which duplicate owns the value
        assert_eq!(widget.value_text(store), ("first".to_owned(), true));
        widget.prepare_for_save();
        assert_eq!(widget.value_text(store), ("first".to_owned(), true));

        // Empty mapped output remains a match while missing keys use the fallback
        widget.mappings[0].text.clear();
        assert_eq!(widget.value_text(store), (String::new(), true));
        widget.mappings[0].value = "invalid".to_owned();
        assert_eq!(widget.value_text(store), (format!("Unknown ({value})"), false));
    }
}
