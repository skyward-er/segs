mod source_window;

use super::PaneBehavior;
use crate::{
    mavlink::{MessageData, ROCKET_FLIGHT_TM_DATA, TimedMessage, extract_from_message},
    ui::app::PaneResponse,
};
use egui::{Color32, Vec2b};
use egui_plot::{Legend, Line, PlotPoints};
use egui_tiles::TileId;
use serde::{Deserialize, Serialize};
use source_window::{SourceSettings, sources_window};
use std::iter::zip;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Plot2DPane {
    // UI settings
    #[serde(skip)]
    pub contains_pointer: bool,
    #[serde(skip)]
    settings_visible: bool,

    line_settings: Vec<LineSettings>,
    #[serde(skip)]
    line_data: Vec<Vec<[f64; 2]>>,

    settings: MsgSources,

    #[serde(skip)]
    state_valid: bool,
}

impl PartialEq for Plot2DPane {
    fn eq(&self, other: &Self) -> bool {
        self.settings == other.settings && self.line_settings == other.line_settings
    }
}

impl PaneBehavior for Plot2DPane {
    #[profiling::function]
    fn ui(&mut self, ui: &mut egui::Ui, _: TileId) -> PaneResponse {
        let mut response = PaneResponse::default();

        let mut settings = SourceSettings::new(&mut self.settings, &mut self.line_settings);
        egui::Window::new("Plot Settings")
            .id(ui.auto_id_with("plot_settings")) // TODO: fix this issue with ids
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut self.settings_visible)
            .show(ui.ctx(), |ui| sources_window(ui, &mut settings));

        if settings.are_sources_changed() {
            self.state_valid = false;
        }

        let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);

        egui_plot::Plot::new("plot")
            .auto_bounds(Vec2b::TRUE)
            .legend(Legend::default())
            .label_formatter(|name, value| format!("{} - x:{:.2} y:{:.2}", name, value.x, value.y))
            .show(ui, |plot_ui| {
                self.contains_pointer = plot_ui.response().contains_pointer();
                if plot_ui.response().dragged() && ctrl_pressed {
                    response.set_drag_started();
                }

                for (settings, points) in zip(&self.line_settings, &mut self.line_data) {
                    plot_ui.line(
                        // TODO: remove clone when PlotPoints supports borrowing
                        Line::new(PlotPoints::from(points.clone()))
                            .color(settings.color)
                            .width(settings.width)
                            .name(&settings.field),
                    );
                }
                plot_ui
                    .response()
                    .context_menu(|ui| show_menu(ui, &mut self.settings_visible));
            });

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

        let MsgSources {
            x_field, y_fields, ..
        } = &self.settings;

        for msg in messages {
            let x: f64 = extract_from_message(&msg.message, [x_field]).unwrap()[0];
            let ys: Vec<f64> = extract_from_message(&msg.message, y_fields).unwrap();

            if self.line_data.len() < ys.len() {
                self.line_data.resize(ys.len(), Vec::new());
            }

            for (line, y) in zip(&mut self.line_data, ys) {
                let point = if x_field == "timestamp" {
                    [x / 1e6, y]
                } else {
                    [x, y]
                };

                line.push(point);
            }
        }

        self.state_valid = true;
    }

    fn get_message_subscription(&self) -> Option<u32> {
        Some(self.settings.msg_id)
    }

    fn should_send_message_history(&self) -> bool {
        !self.state_valid
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MsgSources {
    msg_id: u32,
    x_field: String,
    y_fields: Vec<String>,
}

impl Default for MsgSources {
    fn default() -> Self {
        Self {
            msg_id: ROCKET_FLIGHT_TM_DATA::ID,
            x_field: "timestamp".to_owned(),
            y_fields: Vec::new(),
        }
    }
}

impl PartialEq for MsgSources {
    fn eq(&self, other: &Self) -> bool {
        self.msg_id == other.msg_id
            && self.x_field == other.x_field
            && self.y_fields == other.y_fields
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LineSettings {
    field: String,
    width: f32,
    color: Color32,
}

impl Default for LineSettings {
    fn default() -> Self {
        Self {
            field: "".to_owned(),
            width: 1.0,
            color: Color32::BLUE,
        }
    }
}

impl LineSettings {
    fn new(field_y: String) -> Self {
        Self {
            field: field_y,
            ..Default::default()
        }
    }
}

fn show_menu(ui: &mut egui::Ui, settings_visible: &mut bool) {
    ui.set_max_width(200.0); // To make sure we wrap long text

    if ui.button("Settings…").clicked() {
        *settings_visible = true;
        ui.close_menu();
    }
}
