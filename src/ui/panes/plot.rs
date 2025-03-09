mod source_window;

use super::PaneBehavior;
use crate::{
    MAVLINK_PROFILE,
    error::ErrInstrument,
    mavlink::{
        MessageData, ROCKET_FLIGHT_TM_DATA, TimedMessage,
        reflection::{FieldLike, IndexedField},
    },
    ui::{app::PaneResponse, cache::ChangeTracker},
};
use egui::{Color32, Vec2b};
use egui_plot::{Legend, Line, PlotPoint, PlotPoints};
use egui_tiles::TileId;
use serde::{self, Deserialize, Serialize, ser::SerializeStruct};
use source_window::sources_window;
use std::{hash::Hash, iter::zip};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Plot2DPane {
    settings: PlotSettings,
    // UI settings
    #[serde(skip)]
    line_data: Vec<Vec<PlotPoint>>,
    #[serde(skip)]
    state_valid: bool,
    // UI settings
    #[serde(skip)]
    settings_visible: bool,
    #[serde(skip)]
    pub contains_pointer: bool,
}

impl PartialEq for Plot2DPane {
    fn eq(&self, other: &Self) -> bool {
        self.settings == other.settings
    }
}

impl PaneBehavior for Plot2DPane {
    #[profiling::function]
    fn ui(&mut self, ui: &mut egui::Ui, _: TileId) -> PaneResponse {
        let mut response = PaneResponse::default();

        let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);

        // plot last 100 messages
        egui_plot::Plot::new("plot")
            .auto_bounds(Vec2b::TRUE)
            .legend(Legend::default())
            .label_formatter(|name, value| format!("{} - x:{:.2} y:{:.2}", name, value.x, value.y))
            .show(ui, |plot_ui| {
                self.contains_pointer = plot_ui.response().contains_pointer();
                if plot_ui.response().dragged() && ctrl_pressed {
                    response.set_drag_started();
                }

                for ((field, settings), points) in zip(self.settings.plot_lines(), &self.line_data)
                {
                    plot_ui.line(
                        Line::new(PlotPoints::from(
                            &points[points.len().saturating_sub(100)..], // FIXME: don't show just the last 100 points
                        ))
                        .color(settings.color)
                        .width(settings.width)
                        .name(&field.field().name),
                    );
                }
                plot_ui
                    .response()
                    .context_menu(|ui| show_menu(ui, &mut self.settings_visible));
            });

        let settings_hash = ChangeTracker::record_initial_state(&self.settings);
        egui::Window::new("Plot Settings")
            .id(ui.auto_id_with("plot_settings")) // TODO: fix this issue with ids
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut self.settings_visible)
            .show(ui.ctx(), |ui| sources_window(ui, &mut self.settings));

        if settings_hash.has_changed(&self.settings) {
            self.state_valid = false;
        }

        response
    }

    fn contains_pointer(&self) -> bool {
        self.contains_pointer
    }

    #[profiling::function]
    fn update(&mut self, messages: &[TimedMessage]) {
        if !self.state_valid {
            self.line_data.clear();
        }

        let PlotSettings {
            x_field, y_fields, ..
        } = &self.settings;

        for msg in messages {
            let x: f64 = x_field.extract_as_f64(&msg.message).log_unwrap();
            let ys: Vec<f64> = y_fields
                .iter()
                .map(|(field, _)| field.extract_as_f64(&msg.message).log_unwrap())
                .collect();

            if self.line_data.len() < ys.len() {
                self.line_data.resize(ys.len(), Vec::new());
            }

            for (line, y) in zip(&mut self.line_data, ys) {
                let point = if x_field.field().name == "timestamp" {
                    PlotPoint::new(x / 1e6, y)
                } else {
                    PlotPoint::new(x, y)
                };

                line.push(point);
            }
        }

        self.state_valid = true;
    }

    fn get_message_subscription(&self) -> Option<u32> {
        Some(self.settings.plot_message_id)
    }

    fn should_send_message_history(&self) -> bool {
        !self.state_valid
    }
}

fn show_menu(ui: &mut egui::Ui, settings_visible: &mut bool) {
    ui.set_max_width(200.0); // To make sure we wrap long text

    if ui.button("Settings…").clicked() {
        *settings_visible = true;
        ui.close_menu();
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PlotSettings {
    plot_message_id: u32,
    x_field: IndexedField,
    y_fields: Vec<(IndexedField, LineSettings)>,
}

impl PlotSettings {
    fn plot_lines(&self) -> &[(IndexedField, LineSettings)] {
        &self.y_fields
    }

    fn fields_empty(&self) -> bool {
        self.y_fields.is_empty()
    }

    fn get_msg_id(&self) -> u32 {
        self.plot_message_id
    }

    fn get_x_field(&self) -> &IndexedField {
        &self.x_field
    }

    fn get_mut_x_field(&mut self) -> &mut IndexedField {
        &mut self.x_field
    }

    fn get_mut_y_fields(&mut self) -> &mut [(IndexedField, LineSettings)] {
        &mut self.y_fields[..]
    }

    fn set_x_field(&mut self, field: IndexedField) {
        self.x_field = field;
    }

    fn fields_len(&self) -> usize {
        self.y_fields.len()
    }

    fn contains_field(&self, field: &IndexedField) -> bool {
        self.y_fields.iter().any(|(f, _)| f == field)
    }

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
        }
    }
}

impl Hash for PlotSettings {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.plot_message_id.hash(state);
        self.x_field.hash(state);
        self.y_fields.hash(state);
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

impl Serialize for PlotSettings {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("PlotSettings", 3)?;
        let y_fields: Vec<_> = self.y_fields.iter().map(|(f, s)| (f.id(), s)).collect();
        state.serialize_field("msg_id", &self.plot_message_id)?;
        state.serialize_field("x_field", &self.x_field.id())?;
        state.serialize_field("y_fields", &y_fields)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for PlotSettings {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct FieldSettings {
            field: usize,
            settings: LineSettings,
        }

        #[derive(Deserialize)]
        struct PlotSettingsData {
            msg_id: u32,
            x_field: usize,
            y_fields: Vec<FieldSettings>,
        }

        let data = PlotSettingsData::deserialize(deserializer)?;
        let x_field = data
            .x_field
            .to_mav_field(data.msg_id, &MAVLINK_PROFILE)
            .log_unwrap();
        let y_fields = data
            .y_fields
            .into_iter()
            .map(|FieldSettings { field, settings }| {
                (
                    field
                        .to_mav_field(data.msg_id, &MAVLINK_PROFILE)
                        .log_unwrap(),
                    settings,
                )
            })
            .collect();
        Ok(Self {
            plot_message_id: data.msg_id,
            x_field,
            y_fields,
        })
    }
}
