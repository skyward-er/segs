mod source_window;

use super::PaneBehavior;
use crate::{
    mavlink::{extract_from_message, MessageData, TimedMessage, ROCKET_FLIGHT_TM_DATA},
    ui::composable_view::PaneResponse,
};
use egui::{Color32, Vec2b};
use egui_plot::{Legend, Line, PlotPoints};
use egui_tiles::TileId;
use serde::{Deserialize, Serialize};
use source_window::{sources_window, SourceSettings};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Plot2DPane {
    // UI settings
    #[serde(skip)]
    pub contains_pointer: bool,
    #[serde(skip)]
    settings_visible: bool,
    line_settings: Vec<LineSettings>,
    plot_active: bool,
    settings: MsgSources,
    #[serde(skip)]
    cache_valid: bool,
    #[serde(skip)]
    points: Vec<(f64, Vec<f64>)>,
}

impl PartialEq for Plot2DPane {
    fn eq(&self, other: &Self) -> bool {
        self.settings == other.settings
            && self.line_settings == other.line_settings
            && self.plot_active == other.plot_active
    }
}

impl PaneBehavior for Plot2DPane {
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
        // if settings are changed, invalidate the cache
        self.cache_valid = !settings.are_sources_changed();
        // if there are no fields, do not plot
        self.plot_active = !settings.fields_empty();

        let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);

        let mut plot_lines = Vec::new();
        if self.plot_active {
            let acc_points = &self.points;

            let field_x = &self.settings.x_field;
            if !acc_points.is_empty() {
                for (i, plot_line) in self.line_settings.iter().enumerate() {
                    let points: Vec<[f64; 2]> = {
                        let iter = acc_points.iter();
                        if field_x == "timestamp" {
                            iter.map(|(x, ys)| [x / 1e6, ys[i]]).collect()
                        } else {
                            iter.map(|(x, ys)| [*x, ys[i]]).collect()
                        }
                    };
                    plot_lines.push((plot_line.clone(), points));
                }
            }
        }

        let plot = egui_plot::Plot::new("plot")
            .auto_bounds(Vec2b::TRUE)
            .legend(Legend::default())
            .label_formatter(|name, value| format!("{} - x:{:.2} y:{:.2}", name, value.x, value.y));
        plot.show(ui, |plot_ui| {
            self.contains_pointer = plot_ui.response().contains_pointer();
            if plot_ui.response().dragged() && ctrl_pressed {
                println!("ctrl + drag");
                response.set_drag_started();
            }
            for (plot_settings, data_points) in plot_lines {
                plot_ui.line(
                    Line::new(PlotPoints::from(data_points))
                        .color(plot_settings.color)
                        .width(plot_settings.width)
                        .name(&plot_settings.field),
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

    fn update(&mut self, messages: &[TimedMessage]) {
        if !self.cache_valid {
            self.points.clear();
        }

        let MsgSources {
            x_field, y_fields, ..
        } = &self.settings;
        for msg in messages {
            let x: f64 = extract_from_message(&msg.message, [x_field]).unwrap()[0];
            let ys: Vec<f64> = extract_from_message(&msg.message, y_fields).unwrap();
            self.points.push((x, ys));
        }
    }

    fn get_message_subscription(&self) -> Option<u32> {
        Some(self.settings.msg_id)
    }

    fn should_send_message_history(&self) -> bool {
        !self.cache_valid
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
