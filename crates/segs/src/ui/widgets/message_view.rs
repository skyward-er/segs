use egui::Ui;
use segs_ui::widgets::message_view::{MessageRow, MessageViewOptions, message_view_widget};
use serde::{Deserialize, Serialize};

use crate::{
    dataflow::{DataStore, DataStream},
    ui::{widget_settings::WidgetDataSetting, widgets::WidgetTrait},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageViewWidget;

impl WidgetTrait for MessageViewWidget {
    fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        let mut streams: Vec<_> = data_store.streams.iter().collect();
        streams.sort_by_key(|(key, _)| **key);
        let rows = streams.into_iter().enumerate().filter_map(|(index, (_, stream))| {
            latest_value(stream).map(|value| MessageRow::new(format!("Stream {}", index + 1), value))
        });

        message_view_widget(ui, ui.id().with("message_view"), rows, &MessageViewOptions::default());
    }

    fn data_settings(&mut self) -> Vec<WidgetDataSetting<'_>> {
        vec![]
    }

    fn display_name(&self) -> &'static str {
        "Message view"
    }
}

fn latest_value(stream: &DataStream) -> Option<String> {
    match stream {
        DataStream::F64(points) => points.last().map(|point| point.value.to_string()),
        DataStream::I64(points) => points.last().map(|point| point.value.to_string()),
        DataStream::String(points) => points.last().map(|point| point.value.clone()),
    }
}
