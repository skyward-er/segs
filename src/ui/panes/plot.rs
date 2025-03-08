mod source_window;

use super::PaneBehavior;
use crate::{
    MAVLINK_PROFILE,
    error::ErrInstrument,
    mavlink::{
        Message, MessageData, ROCKET_FLIGHT_TM_DATA, TimedMessage,
        reflection::{self, FieldLike},
    },
    ui::app::PaneResponse,
};
use egui::{Color32, Vec2b};
use egui_plot::{Legend, Line, PlotPoint, PlotPoints};
use egui_tiles::TileId;
use mavlink_bindgen::parser::MavType;
use serde::{Deserialize, Serialize};
use source_window::{ChangeTracker, sources_window};
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
                            &points[points.len().saturating_sub(100)..],
                        ))
                        .color(settings.color)
                        .width(settings.width)
                        .name(&field.field.name),
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
            let x: f64 = x_field.extract_from_message(&msg.message).log_unwrap();
            let ys: Vec<f64> = y_fields
                .iter()
                .map(|(field, _)| field.extract_from_message(&msg.message).log_unwrap())
                .collect();

            if self.line_data.len() < ys.len() {
                self.line_data.resize(ys.len(), Vec::new());
            }

            for (line, y) in zip(&mut self.line_data, ys) {
                let point = if x_field.field.name == "timestamp" {
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct PlotSettings {
    plot_message_id: u32,
    x_field: FieldWithID,
    y_fields: Vec<(FieldWithID, LineSettings)>,
}

impl PlotSettings {
    fn plot_lines(&self) -> &[(FieldWithID, LineSettings)] {
        &self.y_fields
    }

    fn fields_empty(&self) -> bool {
        self.y_fields.is_empty()
    }

    fn get_msg_id(&self) -> u32 {
        self.plot_message_id
    }

    fn get_x_field(&self) -> &FieldWithID {
        &self.x_field
    }

    fn get_y_fields(&self) -> Vec<&FieldWithID> {
        self.y_fields.iter().map(|(field, _)| field).collect()
    }

    // fn get_mut_msg_id(&mut self) -> &mut u32 {
    //     &mut self.msg_sources.plot_message_id
    // }

    fn get_mut_x_field(&mut self) -> &mut FieldWithID {
        &mut self.x_field
    }

    fn get_mut_y_fields(&mut self) -> &mut [(FieldWithID, LineSettings)] {
        &mut self.y_fields[..]
    }

    fn set_x_field(&mut self, field: FieldWithID) {
        self.x_field = field;
    }

    fn fields_len(&self) -> usize {
        self.y_fields.len()
    }

    // fn is_msg_id_changed(&self) -> bool {
    //     self.msg_sources.plot_message_id != self.old_msg_sources.plot_message_id
    // }

    fn contains_field(&self, field: &FieldWithID) -> bool {
        self.y_fields.iter().any(|(f, _)| f == field)
    }

    fn add_field(&mut self, field: FieldWithID) {
        let line_settings = LineSettings::default();
        self.y_fields.push((field, line_settings));
    }

    fn clear_fields(&mut self) {
        self.x_field = 0
            .to_mav_field(self.plot_message_id, &MAVLINK_PROFILE)
            .log_unwrap()
            .into();
        self.y_fields.clear();
    }
}

impl Default for PlotSettings {
    fn default() -> Self {
        let msg_id = ROCKET_FLIGHT_TM_DATA::ID;
        let x_field = FieldWithID::new(msg_id, 0).log_unwrap();
        let y_fields = vec![(
            FieldWithID::new(msg_id, 1).log_unwrap(),
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

impl LineSettings {
    fn new(width: f32, color: Color32) -> Self {
        Self { width, color }
    }
}

/// A struct to hold a field and its ID in a message
/// We use this and not `reflection::IndexedField` because we need to serialize it
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct FieldWithID {
    id: usize,
    field: reflection::MavField,
}

impl FieldWithID {
    fn new(msg_id: u32, field_id: usize) -> Option<Self> {
        Some(Self {
            id: field_id,
            field: field_id
                .to_mav_field(msg_id, &MAVLINK_PROFILE)
                .ok()?
                .field()
                .clone(),
        })
    }

    fn extract_from_message(&self, message: &impl Message) -> Result<f64, String> {
        macro_rules! downcast {
            ($value: expr, $type: ty) => {
                Ok(*$value
                    .downcast::<$type>()
                    .map_err(|_| "Type mismatch".to_string())? as f64)
            };
        }

        let value = message
            .get_field(self.id)
            .ok_or("Field not found".to_string())?;
        match self.field.mavtype {
            MavType::UInt8 => downcast!(value, u8),
            MavType::UInt16 => downcast!(value, u16),
            MavType::UInt32 => downcast!(value, u32),
            MavType::UInt64 => downcast!(value, u64),
            MavType::Int8 => downcast!(value, i8),
            MavType::Int16 => downcast!(value, i16),
            MavType::Int32 => downcast!(value, i32),
            MavType::Int64 => downcast!(value, i64),
            MavType::Float => downcast!(value, f32),
            MavType::Double => downcast!(value, f64),
            _ => Err("Field type not supported".to_string()),
        }
    }
}

impl Hash for FieldWithID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl From<reflection::IndexedField<'_>> for FieldWithID {
    fn from(indexed_field: reflection::IndexedField<'_>) -> Self {
        Self {
            id: indexed_field.id(),
            field: indexed_field.field().clone(),
        }
    }
}
