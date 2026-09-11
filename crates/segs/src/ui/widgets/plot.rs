use std::ops::RangeInclusive;

use egui::{Color32, Id, Stroke, Ui};
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

const DEFAULT_HISTORY_SECONDS: f64 = 60.;
const HISTORY_STEP_VALUE: f64 = 60.;
const STEP_VALUE_SETTING_WIDTH: f32 = 96.;
const DEFAULT_Y_MIN: f64 = 0.;
const DEFAULT_Y_MAX: f64 = 1.;
const DEFAULT_LINE_WIDTH: f64 = 1.5;
const MINIMUM_LINE_WIDTH: f64 = 0.25;
const MAXIMUM_LINE_WIDTH: f64 = 10.;
const LINE_WIDTH_STEP: f64 = 0.25;
const DEFAULT_LINE_COLOR: Color32 = Color32::BLUE;

/// Displays one selected numeric data stream as a time-series line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWidget {
    /// Stream plotted with timestamps on X and sample values on Y.
    stream: Option<StreamKey>,
    /// Opaque color used to draw the plotted line.
    line_color: Color32,
    /// Stroke width of the plotted line in logical points.
    line_width: f64,
    /// Configured width of the live history window in seconds.
    history_seconds: f64,
    /// Whether the vertical range is calculated from visible samples.
    auto_y_bounds: bool,
    /// Configured lower vertical bound used when automatic bounds are disabled.
    y_min: f64,
    /// Configured upper vertical bound used when automatic bounds are disabled.
    y_max: f64,
}

impl Default for PlotWidget {
    /// Creates a plot with no selected stream.
    fn default() -> Self {
        Self {
            stream: None,
            line_color: DEFAULT_LINE_COLOR,
            line_width: DEFAULT_LINE_WIDTH,
            history_seconds: DEFAULT_HISTORY_SECONDS,
            auto_y_bounds: true,
            y_min: DEFAULT_Y_MIN,
            y_max: DEFAULT_Y_MAX,
        }
    }
}

impl WidgetTrait for PlotWidget {
    fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        // Borrow the selected stream directly from the central store
        let stream = self.stream.and_then(|key| data_store.stream(key));
        let plot_id = ui.id().with("plot");
        let settings = LineSettings {
            width: effective_line_width(self.line_width),
            color: self.line_color.to_opaque(),
        };
        let stroke = Stroke::new(settings.width, settings.color);
        let history_seconds = self.history_seconds;
        let y_bounds = configured_y_bounds(self.auto_y_bounds, self.y_min, self.y_max);
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
        // Always expose line appearance, history, and vertical bounds mode
        let auto_y_bounds = self.auto_y_bounds;
        let mut settings = vec![
            WidgetSetting::float(
                "history_seconds",
                "History (s)",
                &mut self.history_seconds,
                1.0..=f64::MAX,
                Some(HISTORY_STEP_VALUE),
                Some(STEP_VALUE_SETTING_WIDTH),
            ),
            WidgetSetting::color("line_color", "Line color", &mut self.line_color),
            WidgetSetting::float(
                "line_width",
                "Line width",
                &mut self.line_width,
                MINIMUM_LINE_WIDTH..=MAXIMUM_LINE_WIDTH,
                Some(LINE_WIDTH_STEP),
                Some(STEP_VALUE_SETTING_WIDTH),
            ),
            WidgetSetting::checkbox("auto_y_bounds", "Auto Y bounds", &mut self.auto_y_bounds),
        ];

        // Show fixed bounds only while they are relevant
        if !auto_y_bounds {
            settings.extend([
                WidgetSetting::float("y_min", "Y minimum", &mut self.y_min, f64::MIN..=f64::MAX, None, None),
                WidgetSetting::float("y_max", "Y maximum", &mut self.y_max, f64::MIN..=f64::MAX, None, None),
            ]);
        }

        settings
    }

    fn display_name(&self) -> &'static str {
        "Plot"
    }
}

fn effective_line_width(line_width: f64) -> f32 {
    // Keep malformed persisted values from producing invalid rendering geometry
    if line_width.is_finite() {
        line_width.clamp(MINIMUM_LINE_WIDTH, MAXIMUM_LINE_WIDTH) as f32
    } else {
        DEFAULT_LINE_WIDTH as f32
    }
}

fn configured_y_bounds(auto_y_bounds: bool, y_min: f64, y_max: f64) -> Option<RangeInclusive<f64>> {
    // Use automatic bounds unless both configured endpoints form a finite increasing range
    if auto_y_bounds {
        return None;
    }
    (y_min.is_finite() && y_max.is_finite() && y_min < y_max).then_some(y_min..=y_max)
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
