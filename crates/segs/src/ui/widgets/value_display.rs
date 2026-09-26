use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::{
    dataflow::{StreamKey, store::DataStore},
    ui::{
        components::centered_value,
        widget_settings::{WidgetDataSetting, WidgetSetting},
        widgets::WidgetTrait,
    },
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueDisplayWidget {
    label: String,
    stream: Option<StreamKey>,
    auto_size: bool,
    text_size: i64,
}

impl Default for ValueDisplayWidget {
    /// Creates an unconfigured value display.
    fn default() -> Self {
        Self {
            label: "Value".to_owned(),
            stream: None,
            auto_size: true,
            text_size: centered_value::DEFAULT_TEXT_SIZE,
        }
    }
}

impl WidgetTrait for ValueDisplayWidget {
    fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        let value = self.value_text(data_store);
        centered_value::show(
            ui,
            &self.label,
            &value,
            &value,
            None,
            self.auto_size,
            self.text_size,
            true,
        );
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
