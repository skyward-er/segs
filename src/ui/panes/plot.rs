mod source_window;

use super::PaneBehavior;
use crate::{
    MAVLINK_PROFILE,
    error::ErrInstrument,
    mavlink::{
        MessageData, ROCKET_FLIGHT_TM_DATA, TimedMessage,
        reflection::{FieldLike, IndexedField},
    },
    ui::{app::PaneResponse, shortcuts::ShortcutHandler},
    utils::units::UnitOfMeasure,
};
use egui::{Color32, Ui, Vec2, Vec2b};
use egui_plot::{AxisHints, HPlacement, Legend, Line, PlotPoint, log_grid_spacer};
use serde::{self, Deserialize, Serialize};
use source_window::sources_window;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    iter::zip,
    time::{Duration, Instant},
};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Plot2DPane {
    settings: PlotSettings,
    // UI settings
    #[serde(skip)]
    line_data: Vec<TimeAwarePlotPoints>,
    #[serde(skip)]
    state_valid: bool,
    #[serde(skip)]
    settings_visible: bool,
}

impl PartialEq for Plot2DPane {
    fn eq(&self, other: &Self) -> bool {
        self.settings == other.settings
    }
}

impl PaneBehavior for Plot2DPane {
    #[profiling::function]
    fn ui(&mut self, ui: &mut Ui, _shortcut_handler: &mut ShortcutHandler) -> PaneResponse {
        let mut response = PaneResponse::default();
        let data_settings_digest = self.settings.data_digest();

        let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);

        let x_unit = UnitOfMeasure::from(
            &self
                .settings
                .x_field
                .field()
                .unit
                .clone()
                .unwrap_or_default(),
        );
        let y_units = self
            .settings
            .y_fields
            .iter()
            .map(|(field, _)| field.field().unit.as_ref().map(UnitOfMeasure::from))
            .collect::<Vec<_>>();
        // define y_unit as the common unit of the y_fields if they are all the same
        let y_unit = y_units
            .iter()
            .fold(y_units.first().log_unwrap(), |acc, unit| {
                match (acc, unit) {
                    (Some(uom), Some(unit)) if uom == unit => acc,
                    _ => &None,
                }
            });
        let x_name = self.settings.x_field.field().name.clone();

        let x_axis = match x_unit {
            UnitOfMeasure::Time(ref time_unit) => {
                AxisHints::new_x().label(&x_name).formatter(move |m, r| {
                    let scaling_factor_to_nanos = time_unit.scale() * 1e9;
                    let r_span_in_nanos = (r.end() - r.start()).abs() * scaling_factor_to_nanos;
                    let m_in_nanos = m.value * scaling_factor_to_nanos;
                    // all the following numbers are arbitrary
                    // they are chosen based on common sense
                    if r_span_in_nanos < 4e3 {
                        format!("{:.0}ns", m_in_nanos)
                    } else if r_span_in_nanos < 4e6 {
                        format!("{:.0}µs", m_in_nanos / 1e3)
                    } else if r_span_in_nanos < 4e9 {
                        format!("{:.0}ms", m_in_nanos / 1e6)
                    } else if r_span_in_nanos < 24e10 {
                        format!("{:.0}s", m_in_nanos / 1e9)
                    } else if r_span_in_nanos < 144e11 {
                        format!("{:.0}m{:.0}s", m_in_nanos / 60e9, (m_in_nanos % 60e9) / 1e9)
                    } else if r_span_in_nanos < 3456e11 {
                        format!(
                            "{:.0}h{:.0}m",
                            m_in_nanos / 3600e9,
                            (m_in_nanos % 3600e9) / 60e9
                        )
                    } else {
                        format!(
                            "{:.0}d{:.0}h",
                            m_in_nanos / 86400e9,
                            (m_in_nanos % 86400e9) / 3600e9
                        )
                    }
                })
            }
            _ => AxisHints::new_x().label(&x_name),
        };
        let y_axis = AxisHints::new_y().placement(HPlacement::Right);

        let cursor_formatter = |name: &str, value: &PlotPoint| {
            let x_unit = format!(" [{}]", x_unit);
            let y_unit = y_unit
                .as_ref()
                .map(|unit| format!(" [{}]", unit))
                .unwrap_or_default();
            if name.is_empty() {
                format!(
                    "{}: {:.2}{}\ny: {:.2}{}",
                    x_name, value.x, x_unit, value.y, y_unit
                )
            } else {
                format!(
                    "{}: {:.2}{}\n{}: {:.2}{}",
                    x_name, value.x, x_unit, name, value.y, y_unit
                )
            }
        };

        let mut plot = egui_plot::Plot::new("plot")
            .x_grid_spacer(log_grid_spacer(4)) // 4 was an arbitrary choice
            .auto_bounds(Vec2b::TRUE)
            .set_margin_fraction(Vec2::splat(0.))
            .legend(Legend::default())
            .label_formatter(cursor_formatter);

        if self.settings.axes_visible {
            plot = plot.custom_x_axes(vec![x_axis]).custom_y_axes(vec![y_axis]);
        } else {
            plot = plot.show_axes(Vec2b::FALSE);
        }

        plot.show(ui, |plot_ui| {
            if plot_ui.response().dragged() && ctrl_pressed {
                response.set_drag_started();
            }

            for ((field, settings), TimeAwarePlotPoints { points, .. }) in
                zip(&self.settings.y_fields, &self.line_data)
            {
                plot_ui.line(
                    Line::new(&points[..])
                        .color(settings.color)
                        .width(settings.width)
                        .name(&field.field().name),
                );
            }
            plot_ui
                .response()
                .context_menu(|ui| show_menu(ui, &mut self.settings_visible, &mut self.settings));
        });

        egui::Window::new("Plot Settings")
            .id(ui.auto_id_with("plot_settings"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut self.settings_visible)
            .show(ui.ctx(), |ui| sources_window(ui, &mut self.settings));

        if data_settings_digest != self.settings.data_digest() {
            self.state_valid = false;
        }

        response
    }

    #[profiling::function]
    fn update(&mut self, messages: &[&TimedMessage]) {
        if !self.state_valid {
            self.line_data.clear();
        }

        let PlotSettings {
            x_field,
            y_fields,
            points_lifespan,
            ..
        } = &self.settings;

        // iter on filtered messages based on lifespan set
        for msg in messages
            .iter()
            .filter(|msg| points_lifespan > &msg.time.elapsed())
        {
            let x: f64 = x_field.extract_as_f64(&msg.message).log_unwrap();
            let ys: Vec<f64> = y_fields
                .iter()
                .map(|(field, _)| field.extract_as_f64(&msg.message).log_unwrap())
                .collect();

            if self.line_data.len() < ys.len() {
                self.line_data.resize(ys.len(), TimeAwarePlotPoints::new());
            }

            for (points, y) in zip(&mut self.line_data, ys) {
                points.push(msg.time, PlotPoint::new(x, y));
            }
        }

        // clear points older than lifespan set
        for line in &mut self.line_data {
            line.clear_older_than(*points_lifespan);
        }

        self.state_valid = true;
    }

    fn get_message_subscriptions(&self) -> Box<dyn Iterator<Item = u32>> {
        Box::new(Some(self.settings.plot_message_id).into_iter())
    }

    fn should_send_message_history(&self) -> bool {
        !self.state_valid
    }
}

fn show_menu(ui: &mut Ui, settings_visible: &mut bool, settings: &mut PlotSettings) {
    ui.set_max_width(200.0); // To make sure we wrap long text

    if ui.button("Source Data Settings…").clicked() {
        *settings_visible = true;
        ui.close_menu();
    }

    ui.checkbox(&mut settings.axes_visible, "Show Axes");
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct PlotSettings {
    /// The message id to plot
    pub(super) plot_message_id: u32,
    /// The field to plot on the x-axis
    pub(super) x_field: IndexedField,
    /// The fields to plot, with their respective line settings
    pub(super) y_fields: Vec<(IndexedField, LineSettings)>,
    /// Whether to show the axes of the plot
    pub(super) axes_visible: bool,
    /// Points will be shown for this duration before being removed
    pub(super) points_lifespan: Duration,
}

impl PlotSettings {
    fn add_field(&mut self, field: IndexedField) {
        let line_settings = LineSettings::default();
        self.y_fields.push((field, line_settings));
    }

    fn clear_fields(&mut self) {
        self.x_field = 0
            .to_mav_field(self.plot_message_id, &MAVLINK_PROFILE)
            .log_unwrap();
        self.y_fields.clear();
    }

    /// Returns a digest of the data settings, used to check if the settings
    /// have changed IMPORTANT: To trigger a redraw, hash the settings that need
    /// to redraw the plot here
    fn data_digest(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.x_field.hash(&mut hasher);
        for (field, _) in &self.y_fields {
            field.hash(&mut hasher);
        }
        self.points_lifespan.as_secs().hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for PlotSettings {
    fn default() -> Self {
        let msg_id = ROCKET_FLIGHT_TM_DATA::ID;
        let x_field = 0.to_mav_field(msg_id, &MAVLINK_PROFILE).log_unwrap();
        let y_fields = vec![(
            1.to_mav_field(msg_id, &MAVLINK_PROFILE).log_unwrap(),
            LineSettings::default(),
        )];
        Self {
            plot_message_id: msg_id,
            x_field,
            y_fields,
            axes_visible: true,
            points_lifespan: Duration::from_secs(600),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LineSettings {
    width: f32,
    color: Color32,
}

impl Default for LineSettings {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: Color32::BLUE,
        }
    }
}

impl Hash for LineSettings {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.width.to_bits().hash(state);
        self.color.hash(state);
    }
}

#[derive(Clone, Debug)]
struct TimeAwarePlotPoints {
    times: Vec<Instant>,
    points: Vec<PlotPoint>,
}

impl TimeAwarePlotPoints {
    fn new() -> Self {
        Self {
            times: Vec::new(),
            points: Vec::new(),
        }
    }

    fn push(&mut self, time: Instant, point: PlotPoint) {
        self.times.push(time);
        self.points.push(point);
    }

    fn clear_older_than(&mut self, lifespan: Duration) {
        while let Some(time) = self.times.first().copied() {
            if time.elapsed() > lifespan {
                self.times.remove(0);
                self.points.remove(0);
            } else {
                break;
            }
        }
    }
}
