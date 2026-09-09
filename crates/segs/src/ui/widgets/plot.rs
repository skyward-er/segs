use std::ops::RangeInclusive;

use egui::{Id, Stroke, Ui};
use segs_memory::MemoryExt;
use segs_plot::mapped_line;
use segs_ui::widgets::plot::{LineSettings, PlotOptions, plot_widget};
use serde::{Deserialize, Serialize};

use crate::{
    dataflow::{DataPoint, DataStream, StreamKey, store::DataStore},
    ui::{
        widget_settings::{WidgetDataSetting, WidgetSetting},
        widgets::WidgetTrait,
    },
};

const DEFAULT_HISTORY_SECONDS: f64 = 90.;
const DEFAULT_Y_MIN: f64 = 0.;
const DEFAULT_Y_MAX: f64 = 1.;

/// Displays one selected numeric data stream as a time-series line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWidget {
    /// Stream plotted with timestamps on X and sample values on Y.
    stream: Option<StreamKey>,
    /// Configured width of the live history window in seconds.
    #[serde(default = "default_history_seconds")]
    history_seconds: String,
    /// Whether the vertical range is calculated from visible samples.
    #[serde(default = "default_auto_y_bounds")]
    auto_y_bounds: bool,
    /// Configured lower vertical bound used when automatic bounds are disabled.
    #[serde(default = "default_y_min")]
    y_min: String,
    /// Configured upper vertical bound used when automatic bounds are disabled.
    #[serde(default = "default_y_max")]
    y_max: String,
}

impl Default for PlotWidget {
    /// Creates a plot with no selected stream.
    fn default() -> Self {
        Self {
            stream: None,
            history_seconds: default_history_seconds(),
            auto_y_bounds: default_auto_y_bounds(),
            y_min: default_y_min(),
            y_max: default_y_max(),
        }
    }
}

impl WidgetTrait for PlotWidget {
    fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        // Borrow the selected stream directly from the central store
        let stream = self.stream.and_then(|key| data_store.stream(key));
        let plot_id = ui.id().with("plot");
        let settings = LineSettings::default();
        let stroke = Stroke::new(settings.width, settings.color);
        let history_seconds = configured_history_seconds(&self.history_seconds);
        let y_bounds = configured_y_bounds(self.auto_y_bounds, &self.y_min, &self.y_max);
        let y_configuration = YConfiguration::from_bounds(y_bounds.as_ref());
        let reset = y_configuration_changed(ui, plot_id, y_configuration);

        // Configure the live horizontal range and selected vertical bounds
        let latest_timestamp = match stream {
            Some(DataStream::F64(points)) => points.last().map(|point| point.timestamp),
            Some(DataStream::I64(points)) => points.last().map(|point| point.timestamp),
            Some(DataStream::String(_)) | None => None,
        };
        let (options, live_range) = plot_options(latest_timestamp, history_seconds, y_bounds, reset);

        // Select a statically dispatched mapper for the borrowed numeric stream
        match stream {
            Some(DataStream::F64(points)) => {
                plot_widget(ui, plot_id, &options, |plot_ui| {
                    let following = plot_ui.auto_bounds().x || plot_ui.response().double_clicked();
                    let range = if following {
                        live_range.clone().unwrap_or_else(|| plot_ui.plot_bounds().range_x())
                    } else {
                        plot_ui.plot_bounds().range_x()
                    };
                    let points = points_in_range(points, range, !following);

                    // Map the selected borrowed suffix directly into the rendered path
                    if !points.is_empty() {
                        plot_ui.add(mapped_line(
                            "Stream",
                            points,
                            |point| (point.timestamp, point.value),
                            stroke,
                        ));
                    }
                })
            }
            Some(DataStream::I64(points)) => {
                plot_widget(ui, plot_id, &options, |plot_ui| {
                    let following = plot_ui.auto_bounds().x || plot_ui.response().double_clicked();
                    let range = if following {
                        live_range.clone().unwrap_or_else(|| plot_ui.plot_bounds().range_x())
                    } else {
                        plot_ui.plot_bounds().range_x()
                    };
                    let points = points_in_range(points, range, !following);

                    // Map the selected borrowed suffix directly into the rendered path
                    if !points.is_empty() {
                        plot_ui.add(mapped_line(
                            "Stream",
                            points,
                            |point| (point.timestamp, point.value as f64),
                            stroke,
                        ));
                    }
                })
            }
            // Retain plot interaction state while no numeric data is available
            Some(DataStream::String(_)) | None => plot_widget(ui, plot_id, &options, |_| {}),
        };
    }

    fn data_settings(&mut self) -> Vec<WidgetDataSetting<'_>> {
        vec![WidgetDataSetting::single_stream("stream", "Stream", &mut self.stream)]
    }

    fn settings(&mut self) -> Vec<WidgetSetting<'_>> {
        // Always expose the history and vertical bounds mode
        let auto_y_bounds = self.auto_y_bounds;
        let mut settings = vec![
            WidgetSetting::text_box("history_seconds", "History (s)", &mut self.history_seconds),
            WidgetSetting::checkbox("auto_y_bounds", "Auto Y bounds", &mut self.auto_y_bounds),
        ];

        // Show fixed bounds only while they are relevant
        if !auto_y_bounds {
            settings.extend([
                WidgetSetting::text_box("y_min", "Y minimum", &mut self.y_min),
                WidgetSetting::text_box("y_max", "Y maximum", &mut self.y_max),
            ]);
        }

        settings
    }

    fn display_name(&self) -> &'static str {
        "Plot"
    }
}

fn default_history_seconds() -> String {
    DEFAULT_HISTORY_SECONDS.to_string()
}

const fn default_auto_y_bounds() -> bool {
    true
}

fn default_y_min() -> String {
    DEFAULT_Y_MIN.to_string()
}

fn default_y_max() -> String {
    DEFAULT_Y_MAX.to_string()
}

fn configured_history_seconds(history_seconds: &str) -> f64 {
    // Accept only finite positive durations and fall back to the widget default
    history_seconds
        .parse::<f64>()
        .ok()
        .filter(|seconds| seconds.is_finite() && *seconds > 0.)
        .unwrap_or(DEFAULT_HISTORY_SECONDS)
}

fn configured_y_bounds(auto_y_bounds: bool, y_min: &str, y_max: &str) -> Option<RangeInclusive<f64>> {
    // Use automatic bounds unless both configured endpoints form a finite increasing range
    if auto_y_bounds {
        return None;
    }
    let min = y_min.parse::<f64>().ok().filter(|value| value.is_finite())?;
    let max = y_max.parse::<f64>().ok().filter(|value| value.is_finite())?;
    (min < max).then_some(min..=max)
}

fn plot_options(
    latest_timestamp: Option<f64>,
    history_seconds: f64,
    y_bounds: Option<RangeInclusive<f64>>,
    reset: bool,
) -> (PlotOptions, Option<RangeInclusive<f64>>) {
    // Anchor the live window to the newest sample without consulting wall time
    let live_range = latest_timestamp.map(|latest| latest - history_seconds..=latest);

    // Use exact horizontal bounds while retaining the normal vertical margin
    let mut margin_fraction = PlotOptions::default().margin_fraction;
    if live_range.is_some() {
        margin_fraction.x = 0.;
    }
    let options = PlotOptions {
        x_auto_range: live_range.clone(),
        y_bounds,
        margin_fraction,
        reset,
        ..Default::default()
    };

    (options, live_range)
}

fn points_in_range<T>(points: &[DataPoint<T>], range: RangeInclusive<f64>, include_neighbors: bool) -> &[DataPoint<T>] {
    // Locate both range boundaries in the chronologically ordered stream
    let mut start = points.partition_point(|point| point.timestamp < *range.start());
    let mut end = points.partition_point(|point| point.timestamp <= *range.end());

    // Retain adjacent samples when inspecting history so clipped segments stay continuous
    if include_neighbors {
        start = start.saturating_sub(1);
        end = end.saturating_add(1).min(points.len());
    }

    &points[start.min(end)..end]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum YConfiguration {
    Auto,
    Fixed { min: u64, max: u64 },
}

impl YConfiguration {
    fn from_bounds(bounds: Option<&RangeInclusive<f64>>) -> Self {
        match bounds {
            Some(bounds) => Self::Fixed {
                min: bounds.start().to_bits(),
                max: bounds.end().to_bits(),
            },
            None => Self::Auto,
        }
    }
}

fn y_configuration_changed(ui: &Ui, plot_id: Id, configuration: YConfiguration) -> bool {
    // Reset the plot once whenever the effective vertical configuration changes
    let state_id = plot_id.with("y_configuration");
    let previous = ui.mem().get_temp::<YConfiguration>(state_id);
    ui.mem().insert_temp(state_id, configuration);
    previous != Some(configuration)
}
