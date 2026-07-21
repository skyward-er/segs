use egui::Ui;
use segs_plot::PlotPoint;
use segs_ui::widgets::plot::{LineSettings, PlotOptions, PlotSeries, plot_widget};
use serde::{Deserialize, Serialize};

use crate::{
    dataflow::{DataStore, DataStream},
    ui::widgets::WidgetTrait,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWidget;

impl WidgetTrait for PlotWidget {
    fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        let mut streams: Vec<_> = data_store.streams.iter().collect();
        streams.sort_by_key(|(key, _)| **key);

        let series: Vec<_> = streams
            .into_iter()
            .enumerate()
            .find_map(|(index, (_, stream))| {
                points_from_stream(stream).map(|points| PlotSeries {
                    id: format!("Stream {}", index + 1),
                    points,
                    settings: LineSettings::default(),
                })
            })
            .into_iter()
            .collect();

        plot_widget(ui, ui.id().with("plot"), &series, &PlotOptions::default());
    }

    fn data_settings(&mut self) -> Vec<crate::ui::widget_settings::WidgetDataSetting<'_>> {
        vec![]
    }

    fn display_name(&self) -> &'static str {
        "Plot"
    }
}

fn points_from_stream(stream: &DataStream) -> Option<Vec<PlotPoint>> {
    match stream {
        DataStream::F64(points) => Some(
            points
                .iter()
                .map(|point| PlotPoint::new(point.timestamp, point.value))
                .collect(),
        ),
        DataStream::I64(points) => Some(
            points
                .iter()
                .map(|point| PlotPoint::new(point.timestamp, point.value as f64))
                .collect(),
        ),
        DataStream::String(_) => None,
    }
}
