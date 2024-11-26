use crate::{ui::composable_view::PaneResponse, MAVLINK_PROFILE, MSG_MANAGER};

use super::PaneBehavior;

use egui::{Color32, Vec2b};
use egui_plot::{Legend, Line, PlotPoints};
use serde::{Deserialize, Serialize};
use skyward_mavlink::{
    lyra::{MavMessage, ROCKET_FLIGHT_TM_DATA},
    mavlink::{Message, MessageData},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlotLineSettings {
    field: String,
    width: f32,
    color: Color32,
}

impl Default for PlotLineSettings {
    fn default() -> Self {
        Self {
            field: "".to_owned(),
            width: 1.0,
            color: Color32::BLUE,
        }
    }
}

impl PlotLineSettings {
    fn new(field_y: String) -> Self {
        Self {
            field: field_y,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plot2DPane {
    // UI settings
    #[serde(skip)]
    pub contains_pointer: bool,
    settings_visible: bool,
    sources_visible: bool,
    // Mavlink settings
    msg_id: u32,
    field_x: String,
    plot_lines: Vec<PlotLineSettings>,
    plot_active: bool,
    reverse_data: bool
}

impl Default for Plot2DPane {
    fn default() -> Self {
        Self {
            contains_pointer: false,
            settings_visible: false,
            sources_visible: false,
            msg_id: ROCKET_FLIGHT_TM_DATA::ID,
            field_x: "timestamp".to_owned(),
            plot_lines: vec![],
            plot_active: false,
            reverse_data: false
        }
    }
}

impl PaneBehavior for Plot2DPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let mut response = PaneResponse::default();

        let Self {
            settings_visible,
            sources_visible,
            plot_lines,
            msg_id,
            field_x,
            plot_active,
            reverse_data,
            ..
        } = self;


        // Spawn windows
        egui::Window::new("Plot Settings")
            .id(ui.make_persistent_id("plot_settings"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(settings_visible)
            .show(ui.ctx(), |ui| settings_window(ui, plot_lines));

        egui::Window::new("Plot Sources")
            .id(ui.make_persistent_id("plot_sources"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(sources_visible)
            .show(ui.ctx(), |ui| {
                sources_window(ui, msg_id, field_x, plot_lines, plot_active, reverse_data)
            });

        let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);

        let mut plot_lines = Vec::new();
        if self.plot_active {
            let acc_points = MSG_MANAGER
                .get()
                .unwrap()
                .lock()
                .get_message(*msg_id)
                .map(|msg| {
                    msg.iter()
                        .map(|msg| {
                            let value: serde_json::Value =
                                serde_json::to_value(msg.message.clone()).unwrap();

                            let x = value.get(&*field_x).unwrap();
                            let x = serde_json::from_value::<f64>(x.clone()).unwrap();
                            let mut ys = Vec::new();
                            for field in self.plot_lines.iter() {
                                let y = value.get(field.field.as_str()).unwrap();
                                if self.reverse_data {
                                    ys.push(serde_json::from_value::<f64>(y.clone()).unwrap() * -1.0);
                                } else {
                                    ys.push(serde_json::from_value::<f64>(y.clone()).unwrap());
                                }
                            }
                            (x, ys)
                        })
                        .collect::<Vec<(f64, Vec<f64>)>>()
                })
                .unwrap_or_default();

            if !acc_points.is_empty() {
                for (i, plot_line) in self.plot_lines.iter().enumerate() {
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
                .context_menu(|ui| show_menu(ui, settings_visible, sources_visible));
        });

        response
    }

    fn contains_pointer(&self) -> bool {
        self.contains_pointer
    }
}

fn settings_window(ui: &mut egui::Ui, plot_lines: &mut [PlotLineSettings]) {
    egui::Grid::new(ui.id())
        .num_columns(4)
        .spacing([10.0, 5.0])
        .show(ui, |ui| {
            for plot_line in plot_lines.iter_mut() {
                ui.label(&plot_line.field);
                ui.color_edit_button_srgba(&mut plot_line.color);
                ui.add(
                    egui::DragValue::new(&mut plot_line.width)
                        .speed(0.1)
                        .suffix(" pt"),
                )
                .on_hover_text("Width of the line in points");
                ui.end_row();
            }
        });
}

fn sources_window(
    ui: &mut egui::Ui,
    msg_id: &mut u32,
    field_x: &mut String,
    plot_lines: &mut Vec<PlotLineSettings>,
    plot_active: &mut bool,
    reverse_data: &mut bool,
) {
    // record msg id to check if it has changed
    let old_msg_id = *msg_id;
    // extract the msg name from the id to show it in the combo box
    let msg_name = MAVLINK_PROFILE
        .get_name_from_id(*msg_id)
        .unwrap_or_default();

    // show the first combo box with the message name selection
    egui::ComboBox::from_label("Message Kind")
        .selected_text(msg_name)
        .show_ui(ui, |ui| {
            for msg in MAVLINK_PROFILE.sorted_messages() {
                ui.selectable_value(msg_id, MavMessage::message_id_from_name(msg).unwrap(), msg);
            }
        });

    // reset fields if the message is changed
    if *msg_id != old_msg_id {
        plot_lines.truncate(1);
    }

    // check fields and assing a default field_x and field_y once the msg is changed
    let fields = MAVLINK_PROFILE.get_plottable_fields_by_id(*msg_id);
    // get the first field that is in the list of fields or the previous if valid
    let new_field_x = fields
        .contains(&field_x.as_str())
        .then(|| field_x.to_owned())
        .or(fields.first().map(|s| s.to_string()));

    // if there are no fields, reset the field_x and plot_lines
    let Some(new_field_x) = new_field_x else {
        *field_x = "".to_owned();
        plot_lines.clear();
        *plot_active = false;
        return;
    };
    // update the field_x
    *field_x = new_field_x;

    // if fields are valid, show the combo boxes for the x_axis
    egui::ComboBox::from_label("X Axis")
        .selected_text(field_x.as_str())
        .show_ui(ui, |ui| {
            for msg in fields.iter() {
                ui.selectable_value(field_x, (*msg).to_owned(), *msg);
            }
        });

    // populate the plot_lines with the first field if it is empty and there are more than 1 fields
    if plot_lines.is_empty() && fields.len() > 1 {
        plot_lines.push(PlotLineSettings::new(fields[1].to_string()));
    }

    // check how many fields are left and how many are selected
    // let fields_selected = plot_lines.len() + 1;
    // let fields_left_to_draw = fields.len().saturating_sub(2);
    // fields_left_to_draw.min(fields_selected.saturating_sub(2)) {
    let plot_lines_len = plot_lines.len();
    egui::Grid::new(ui.auto_id_with("y_axis"))
        .num_columns(3)
        .spacing([10.0, 2.5])
        .show(ui, |ui| {
            for (i, line_settings) in plot_lines.iter_mut().enumerate() {
                // let line_settings = &mut plot_lines.get_mut(1 + i).unwrap();
                let PlotLineSettings {
                    field,
                    width,
                    color,
                } = line_settings;
                let widget_label = if plot_lines_len > 1 {
                    format!("Y Axis {}", i + 1)
                } else {
                    "Y Axis".to_owned()
                };
                egui::ComboBox::from_label(widget_label)
                    .selected_text(field.as_str())
                    .show_ui(ui, |ui| {
                        for msg in fields.iter() {
                            ui.selectable_value(field, (*msg).to_owned(), *msg);
                        }
                    });
                ui.color_edit_button_srgba(color);
                ui.add(egui::DragValue::new(width).speed(0.1).suffix(" pt"))
                .on_hover_text("Width of the line in points");
                ui.checkbox( reverse_data, "Reverse Data")
                .on_hover_text("Check to reverse the data");
                
                ui.end_row();
            }
        });

    // if we have fields left, show the add button
    if fields.len().saturating_sub(plot_lines_len + 1) > 0
        && ui
            .button("Add Y Axis")
            .on_hover_text("Add another Y axis")
            .clicked()
    {
        let next_field = fields
            .iter()
            .find(|f| !plot_lines.iter().any(|l| l.field == **f))
            .unwrap();
        plot_lines.push(PlotLineSettings::new(next_field.to_string()));
    }

    // update fields and flag for active plot
    *plot_active = !plot_lines.is_empty();
}

fn show_menu(ui: &mut egui::Ui, settings_visible: &mut bool, sources_visible: &mut bool) {
    ui.set_max_width(200.0); // To make sure we wrap long text

    if ui.button("Settings…").clicked() {
        *settings_visible = true;
        ui.close_menu();
    }

    if ui.button("Sources…").clicked() {
        *sources_visible = true;
        ui.close_menu();
    }
}
