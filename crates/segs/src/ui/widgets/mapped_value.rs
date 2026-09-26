use std::collections::HashSet;

use egui::{Color32, Ui};
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

/// Displays the latest integer stream value through a user-defined text and background color mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MappedValueWidget {
    /// Optional label displayed above the mapped value.
    label: String,
    /// Integer stream whose latest value is translated.
    stream: Option<StreamKey>,
    /// Ordered integer-to-text translations, each with an optional background color.
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
        self.mappings = [
            ("0", "INIT", Color32::from_rgb(245, 158, 11)),
            ("1", "INIT ERROR", Color32::from_rgb(220, 38, 38)),
            ("2", "OK", Color32::from_rgb(22, 163, 74)),
        ]
        .into_iter()
        .map(|(value, text, color)| IntegerTextMapping {
            value: value.to_owned(),
            text: text.to_owned(),
            color: Some(color),
        })
        .collect();
    }

    fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        // Resolve the current output, its mapped background and whether a configured mapping matched
        let current = self.current_mapping(data_store);
        let (value, fill) = match &current {
            Ok(mapping) => (mapping.text.as_str(), mapping.color),
            Err(fallback) => (fallback.as_str(), None),
        };
        let sizing_value = if self.auto_size && current.is_ok() {
            self.widest_mapping(ui).unwrap_or(value)
        } else {
            value
        };

        // Keep mapped-state transitions sized for the widest configured output
        centered_value::show(
            ui,
            &self.label,
            value,
            sizing_value,
            fill,
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
            WidgetSetting::integer_text_mappings("mappings", "Mapping", &mut self.mappings, true),
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
    /// Returns the configured row matching the latest stream value.
    ///
    /// Returns `Err` with the fallback text to display without a background when
    /// no stream is configured, no integer sample exists, or no row matches.
    fn current_mapping(&self, data_store: &DataStore) -> Result<&IntegerTextMapping, String> {
        let Some(stream_key) = self.stream else {
            return Err("No stream".to_owned());
        };
        let Some(stream) = data_store.stream(stream_key) else {
            return Err("No data".to_owned());
        };
        let DataStream::I64(points) = stream else {
            return Err("Expected integer stream".to_owned());
        };
        let Some(value) = points.last().map(|point| point.value) else {
            return Err("No data".to_owned());
        };

        // Honor persisted row order so malformed duplicate mappings are deterministic
        self.mappings
            .iter()
            .find(|mapping| mapping.parsed_value() == Some(value))
            .ok_or_else(|| format!("Unknown ({value})"))
    }

    /// Returns the widest valid output, or `None` when no valid mapping exists.
    /// Each first-occurrence output is measured once using egui's shaped-layout cache.
    fn widest_mapping(&self, ui: &Ui) -> Option<&str> {
        // Validate and measure in one pass while preserving the first widest match
        let mut seen = HashSet::new();
        let mut widest = None;
        let mut widest_width = f32::NEG_INFINITY;
        for mapping in &self.mappings {
            let Some(value) = mapping.parsed_value() else { continue };
            if !seen.insert(value) {
                continue;
            }
            let width = centered_value::text_width(ui, &mapping.text);
            if width > widest_width {
                widest = Some(mapping.text.as_str());
                widest_width = width;
            }
        }
        widest
    }
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
                color: None,
            })
            .collect(),
            ..Default::default()
        };
        let current_text =
            |widget: &MappedValueWidget| widget.current_mapping(store).map(|mapping| mapping.text.clone());

        // Lookup and persistence must agree on which duplicate owns the value
        assert_eq!(current_text(&widget), Ok("first".to_owned()));
        widget.prepare_for_save();
        assert_eq!(current_text(&widget), Ok("first".to_owned()));

        // Empty mapped output remains a match while missing keys use the fallback
        widget.mappings[0].text.clear();
        assert_eq!(current_text(&widget), Ok(String::new()));
        widget.mappings[0].value = "invalid".to_owned();
        assert_eq!(current_text(&widget), Err(format!("Unknown ({value})")));
    }
}
