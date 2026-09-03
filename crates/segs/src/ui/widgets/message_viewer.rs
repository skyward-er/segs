use std::{collections::HashMap, time::Duration};

use egui::{CornerRadius, FontId, Frame, Label, Margin, Rect, RichText, ScrollArea, Sense, Ui, Vec2, pos2, vec2};
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

const DEFAULT_TEXT_SIZE: f32 = 12.;
const HEADER_TEXT_SIZE_SCALE: f32 = 1.25;
const DEFAULT_STALE_AFTER_SECONDS: f64 = 5.;
const INNER_MARGIN: i8 = 2;
const ROW_SPACING: f32 = 3.;
const STALE_VALUE_HORIZONTAL_PADDING: f32 = 3.;
const STALE_VALUE_VERTICAL_PADDING: f32 = 1.;
const AGE_STATE_ID: &str = "message_viewer_age";

/// Displays the latest values of selected streams in a borderless table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageViewerWidget {
    /// Title displayed above the stream rows.
    header: String,
    /// Streams displayed in descriptor selection order.
    streams: Vec<StreamKey>,
    /// Persistent leaf names parallel to `streams`.
    stream_names: Vec<String>,
    /// Configured row text size in logical points.
    text_size: String,
    /// Whether values are highlighted after no new samples have been received.
    show_stale_warning: bool,
    /// Duration in seconds before highlighting a non-recent value.
    stale_after: String,
}

impl Default for MessageViewerWidget {
    /// Creates an unconfigured message viewer with its standard presentation.
    fn default() -> Self {
        Self {
            header: "Values".to_owned(),
            streams: Vec::new(),
            stream_names: Vec::new(),
            text_size: DEFAULT_TEXT_SIZE.to_string(),
            show_stale_warning: true,
            stale_after: DEFAULT_STALE_AFTER_SECONDS.to_string(),
        }
    }
}

impl WidgetTrait for MessageViewerWidget {
    fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        // Parse the display settings and restore age tracking for the selected streams
        let text_size = configured_text_size(&self.text_size);
        let stale_after = configured_stale_after(&self.stale_after);
        let age_state_id = ui.id().with(AGE_STATE_ID);
        let now = ui.input(|input| input.time);
        let mut age_state = ui.mem().get_temp_or_default::<AgeState>(age_state_id);
        age_state.retain(&self.streams);

        Frame::new().inner_margin(ui.spacing().window_margin).show(ui, |ui| {
            // Draw the inset header outside the scroll area so it remains fixed
            if !self.header.is_empty() {
                Frame::new()
                    .inner_margin(Margin {
                        left: INNER_MARGIN,
                        right: INNER_MARGIN,
                        top: INNER_MARGIN,
                        bottom: 0,
                    })
                    .show(ui, |ui| {
                        ui.add(
                            Label::new(
                                RichText::new(&self.header)
                                    .size(text_size * HEADER_TEXT_SIZE_SCALE)
                                    .strong(),
                            )
                            .extend()
                            .selectable(false),
                        );
                    });
            }

            // Show the padded empty state without constructing grid geometry
            if self.streams.is_empty() {
                ui.mem().insert_temp(age_state_id, age_state);
                Frame::new().inner_margin(INNER_MARGIN).show(ui, |ui| {
                    ui.add(
                        Label::new(RichText::new("No fields selected").size(text_size).weak())
                            .extend()
                            .selectable(false),
                    );
                });
                return;
            }

            // Represent the laid-out data needed to paint one row without further measurement
            struct PreparedRow {
                name_galley: std::sync::Arc<egui::Galley>,
                value_galley: std::sync::Arc<egui::Galley>,
                name_baseline: f32,
                value_baseline: f32,
                stale: bool,
            }

            // Initialize shared rendering resources and grid geometry accumulators
            let app_style = ui.app_style();
            let text_color = ui.visuals().text_color();
            let timeout_fill = app_style.timeout_fill;
            let name_font = app_style.base_font_of(text_size);
            let value_font = FontId::monospace(text_size);
            let mut next_repaint_seconds: Option<f64> = None;
            let mut name_width: f32 = 0.;
            let mut value_width: f32 = 0.;
            // Track the largest distance above the shared baseline, including badge padding
            let mut row_ascent: f32 = 0.;
            // Track the largest distance below the shared baseline, including badge padding
            let mut row_descent: f32 = 0.;

            // Prepare each selected stream and accumulate grid measurements in the same pass
            let rows = self
                .streams
                .iter()
                .enumerate()
                .map(|(position, stream)| {
                    // Resolve the persistent display name and latest formatted stream value
                    let name = self
                        .stream_names
                        .get(position)
                        .map(String::as_str)
                        .unwrap_or("Unavailable field");
                    let latest = data_store.latest(*stream);
                    let value_text = latest
                        .as_ref()
                        .map(|(_, value)| value.to_string())
                        .unwrap_or("No data".to_owned());

                    // Update sample age state and determine stale highlighting and repaint timing
                    let age = match latest.as_ref() {
                        Some((timestamp, _)) => Some(age_state.observe(*stream, *timestamp, now)),
                        None => {
                            age_state.remove(*stream);
                            None
                        }
                    };
                    let stale = self.show_stale_warning && age.is_some_and(|age| age >= stale_after);
                    if self.show_stale_warning
                        && let Some(age) = age
                        && age < stale_after
                    {
                        let remaining = stale_after - age;
                        next_repaint_seconds =
                            Some(next_repaint_seconds.map_or(remaining, |current| current.min(remaining)));
                    }

                    // Lay out both text columns and cache their typographic baselines
                    let name_galley = ui
                        .painter()
                        .layout_no_wrap(name.to_owned(), name_font.clone(), text_color);
                    let value_galley = ui.painter().layout_no_wrap(value_text, value_font.clone(), text_color);
                    let name_baseline = galley_baseline(&name_galley);
                    let value_baseline = galley_baseline(&value_galley);

                    // Accumulate the shared grid geometry before storing the prepared row
                    name_width = name_width.max(name_galley.size().x);
                    value_width = value_width.max(value_galley.size().x + STALE_VALUE_HORIZONTAL_PADDING * 2.);
                    row_ascent = row_ascent.max(name_baseline.max(value_baseline + STALE_VALUE_VERTICAL_PADDING));
                    row_descent = row_descent.max(
                        (name_galley.size().y - name_baseline)
                            .max(value_galley.size().y - value_baseline + STALE_VALUE_VERTICAL_PADDING),
                    );

                    PreparedRow {
                        name_galley,
                        value_galley,
                        name_baseline,
                        value_baseline,
                        stale,
                    }
                })
                .collect::<Vec<_>>();

            // Persist the updated age observations and schedule the nearest stale transition
            ui.mem().insert_temp(age_state_id, age_state);

            if let Some(seconds) = next_repaint_seconds
                && let Ok(delay) = Duration::try_from_secs_f64(seconds)
            {
                ui.ctx().request_repaint_after(delay);
            }

            // Finalize the shared row height and minimum unwrapped grid width
            let row_height = row_ascent + row_descent;
            let minimum_width = name_width + ui.spacing().item_spacing.x + value_width;

            // Fill the viewport and scroll whenever the unwrapped rows no longer fit
            ScrollArea::both()
                .id_salt("message_viewer_scroll")
                .auto_shrink([false, false])
                .min_scrolled_width(0.)
                .min_scrolled_height(0.)
                .content_margin(INNER_MARGIN)
                .show_viewport(ui, |ui, viewport| {
                    // Fit the grid to the padded viewport or preserve its minimum content width
                    ui.spacing_mut().item_spacing.y = ROW_SPACING;
                    let content_margin = f32::from(INNER_MARGIN) * 2.;
                    let available_width = (viewport.width() - content_margin).max(0.);
                    let row_width = available_width.max(minimum_width);
                    ui.set_min_width(row_width);

                    // Allocate each row and position both columns on the shared baseline
                    for prepared_row in rows {
                        let (row, _) = ui.allocate_exact_size(vec2(row_width, row_height), Sense::hover());
                        let name_position = pos2(row.left(), row.top() + row_ascent - prepared_row.name_baseline);
                        let value_position = pos2(
                            row.right() - STALE_VALUE_HORIZONTAL_PADDING - prepared_row.value_galley.size().x,
                            row.top() + row_ascent - prepared_row.value_baseline,
                        );

                        // Paint the timeout fill behind stale values while preserving fresh geometry
                        if prepared_row.stale {
                            let value_rect = Rect::from_min_size(value_position, prepared_row.value_galley.size())
                                .expand2(vec2(STALE_VALUE_HORIZONTAL_PADDING, STALE_VALUE_VERTICAL_PADDING));
                            ui.painter()
                                .rect_filled(value_rect, CornerRadius::same(3), timeout_fill);
                        }

                        // Paint the prepared name and value text over the completed row background
                        ui.painter().galley(name_position, prepared_row.name_galley, text_color);
                        ui.painter()
                            .galley(value_position, prepared_row.value_galley, text_color);
                    }
                });
        });
    }

    fn data_settings(&mut self) -> Vec<WidgetDataSetting<'_>> {
        vec![WidgetDataSetting::multiple_streams_with_names(
            "streams",
            "Fields",
            &mut self.streams,
            &mut self.stream_names,
        )]
    }

    fn settings(&mut self) -> Vec<WidgetSetting<'_>> {
        let show_stale_after = self.show_stale_warning;
        let mut settings = vec![
            WidgetSetting::text_box("header", "Header", &mut self.header),
            WidgetSetting::text_box("text_size", "Text size", &mut self.text_size),
            WidgetSetting::checkbox("show_stale_warning", "Show stale warning", &mut self.show_stale_warning),
        ];
        if show_stale_after {
            settings.push(WidgetSetting::text_box(
                "stale_after",
                "Stale after (s)",
                &mut self.stale_after,
            ));
        }

        settings
    }

    fn display_name(&self) -> &'static str {
        "Message viewer"
    }

    fn default_size(&self) -> Vec2 {
        vec2(3., 2.)
    }
}

/// Tracks when this viewer first observed each current sample.
#[derive(Clone, Default)]
struct AgeState {
    observations: HashMap<StreamKey, AgeObservation>,
}

impl AgeState {
    fn retain(&mut self, streams: &[StreamKey]) {
        self.observations.retain(|stream, _| streams.contains(stream));
    }

    fn observe(&mut self, stream: StreamKey, timestamp: f64, now: f64) -> f64 {
        let observation = self.observations.entry(stream).or_insert(AgeObservation {
            timestamp,
            observed_at: now,
        });
        if observation.timestamp.to_bits() != timestamp.to_bits() {
            *observation = AgeObservation {
                timestamp,
                observed_at: now,
            };
        }

        (now - observation.observed_at).max(0.)
    }

    fn remove(&mut self, stream: StreamKey) {
        self.observations.remove(&stream);
    }
}

/// One sample timestamp and the UI time when it was first observed.
#[derive(Clone, Copy)]
struct AgeObservation {
    timestamp: f64,
    observed_at: f64,
}

/// Parses a positive configured text size, falling back for invalid input.
fn configured_text_size(text_size: &str) -> f32 {
    text_size
        .parse::<f32>()
        .ok()
        .filter(|size| size.is_finite() && *size > 0.)
        .unwrap_or(DEFAULT_TEXT_SIZE)
}

/// Parses a positive stale threshold in seconds, falling back for invalid input.
fn configured_stale_after(stale_after: &str) -> f64 {
    stale_after
        .parse::<f64>()
        .ok()
        .filter(|seconds| seconds.is_finite() && *seconds > 0.)
        .unwrap_or(DEFAULT_STALE_AFTER_SECONDS)
}

/// Returns the first text row's baseline relative to the galley's top edge.
fn galley_baseline(galley: &egui::Galley) -> f32 {
    galley
        .rows
        .first()
        .and_then(|row| row.glyphs.first().map(|glyph| row.pos.y + glyph.pos.y))
        .unwrap_or(galley.size().y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_names_round_trip_through_json() {
        let widget = MessageViewerWidget {
            header: "Telemetry".to_owned(),
            streams: vec![StreamKey::mock()],
            stream_names: vec!["Altitude".to_owned()],
            text_size: "18".to_owned(),
            show_stale_warning: false,
            stale_after: "2.5".to_owned(),
        };

        let json = serde_json::to_string(&widget).unwrap();
        let restored: MessageViewerWidget = serde_json::from_str(&json).unwrap();

        assert_eq!(restored, widget);
    }

    #[test]
    fn age_resets_when_the_observed_sample_changes() {
        let stream = StreamKey::mock();
        let mut state = AgeState::default();

        assert_eq!(state.observe(stream, 1., 10.), 0.);
        assert_eq!(state.observe(stream, 1., 12.5), 2.5);
        assert_eq!(state.observe(stream, 2., 12.5), 0.);
    }
}
