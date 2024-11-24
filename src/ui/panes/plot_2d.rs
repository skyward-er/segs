use crate::{ui::composable_view::PaneResponse, MAVLINK_PROFILE, MSG_MANAGER};

use super::PaneBehavior;

use egui::Color32;
use egui_plot::{Line, PlotPoints};
use serde::{Deserialize, Serialize};
use skyward_mavlink::{
    lyra::{MavMessage, ROCKET_FLIGHT_TM_DATA},
    mavlink::{Message, MessageData},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlotLineSettings {
    field_y: String,
    width: f32,
    color: Color32,
}

impl Default for PlotLineSettings {
    fn default() -> Self {
        Self {
            field_y: "".to_owned(),
            width: 1.0,
            color: Color32::BLUE,
        }
    }
}

impl PlotLineSettings {
    fn new(field_y: String) -> Self {
        Self {
            field_y,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plot2DPane {
    // UI settings
    #[serde(skip)]
    pub contains_pointer: bool,
    #[serde(skip)]
    settings_visible: bool,
    sources_visible: bool,
    // Mavlink settings
    msg_id: u32,
    field_x: String,
    plot_lines: Vec<PlotLineSettings>,
    plot_active: bool,
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
        }
    }
}

impl PartialEq for Plot2DPane {
    fn eq(&self, other: &Self) -> bool {
        self.n_points == other.n_points
            && self.frequency == other.frequency
            && self.width == other.width
            && self.color == other.color
    }
}

impl PaneBehavior for Plot2DPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let mut response = PaneResponse::default();

        // Spawn windows
        let mut settings_window_visible = self.settings_visible;
        egui::Window::new("Plot Settings")
            .id(ui.make_persistent_id("plot_settings"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut settings_window_visible)
            .show(ui.ctx(), |ui| self.settings_window(ui));
        self.settings_visible = settings_window_visible;

        let mut sources_window_visible = self.sources_visible;
        egui::Window::new("Plot Sources")
            .id(ui.make_persistent_id("plot_sources"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut sources_window_visible)
            .show(ui.ctx(), |ui| self.sources_window(ui));
        self.sources_visible = sources_window_visible;

        let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);

        let mut plot_lines = Vec::new();
        if self.plot_active {
            let acc_points = MSG_MANAGER
                .get()
                .unwrap()
                .lock()
                .get_message(self.msg_id)
                .map(|msg| {
                    msg.iter()
                        .map(|msg| {
                            let value: serde_json::Value =
                                serde_json::to_value(msg.message.clone()).unwrap();

                            let x = value.get(&self.field_x).unwrap();
                            let x = serde_json::from_value::<f64>(x.clone()).unwrap();
                            let mut ys = Vec::new();
                            for field in self.plot_lines.iter() {
                                let y = value.get(field.field_y.as_str()).unwrap();
                                ys.push(serde_json::from_value::<f64>(y.clone()).unwrap());
                            }
                            (x, ys)
                        })
                        .collect::<Vec<(f64, Vec<f64>)>>()
                })
                .unwrap_or_default();

            if !acc_points.is_empty() {
                for (i, plot_line) in self.plot_lines.iter().enumerate() {
                    let points: Vec<[f64; 2]> = acc_points
                        .iter()
                        .map(|(timestamp, acc)| [{ *timestamp }, acc[i]])
                        .collect();
                    plot_lines.push((plot_line.clone(), points));
                }
            }
        }

        let plot = egui_plot::Plot::new("plot").auto_bounds([true, true].into());
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
                        .width(plot_settings.width),
                );
            }
            plot_ui.response().context_menu(|ui| self.menu(ui));
        });

        response
    }

    fn contains_pointer(&self) -> bool {
        self.contains_pointer
    }
}

impl Plot2DPane {
    fn menu(&mut self, ui: &mut egui::Ui) {
        ui.set_max_width(200.0); // To make sure we wrap long text

        if ui.button("Settings…").clicked() {
            self.settings_visible = true;
            ui.close_menu();
        }

        if ui.button("Sources…").clicked() {
            self.sources_visible = true;
            ui.close_menu();
        }
    }

    fn settings_window(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new(ui.id())
            .num_columns(4)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                for plot_line in self.plot_lines.iter_mut() {
                    ui.label(&plot_line.field_y);
                    ui.color_edit_button_srgba(&mut plot_line.color);
                    ui.label("Width:");
                    ui.add(egui::Slider::new(&mut plot_line.width, 0.1..=10.0).text("pt"));
                    ui.end_row();
                }
            });
    }

    fn sources_window(&mut self, ui: &mut egui::Ui) {
        let old_msg_id = self.msg_id;
        let msg_name = MAVLINK_PROFILE
            .get_name_from_id(self.msg_id)
            .unwrap_or_default();
        egui::ComboBox::from_label("Message Kind")
            .selected_text(msg_name)
            .show_ui(ui, |ui| {
                for msg in MAVLINK_PROFILE.messages() {
                    ui.selectable_value(
                        &mut self.msg_id,
                        MavMessage::message_id_from_name(msg).unwrap(),
                        msg,
                    );
                }
            });

        // reset fields if the message is changed
        if self.msg_id != old_msg_id {
            self.plot_lines.truncate(1);
        }

        // check fields and assing a default field_x and field_y once the msg is changed
        let fields = MAVLINK_PROFILE.get_plottable_fields_by_id(self.msg_id);
        // get the first field that is in the list of fields or the previous if valid
        let mut field_x = fields
            .contains(&self.field_x.as_str())
            .then(|| self.field_x.clone())
            .or(fields.first().map(|s| s.to_string()));
        // get the second field that is in the list of fields or the previous if valid
        let mut field_y = self
            .plot_lines
            .first()
            .and_then(|s| {
                fields
                    .contains(&s.field_y.as_str())
                    .then_some(s.field_y.to_owned())
            })
            .or(fields.get(1).map(|s| s.to_string()));

        // if fields are valid, show the combo boxes for the x_axis
        if field_x.is_some() {
            let field_x = field_x.as_mut().unwrap();
            egui::ComboBox::from_label("X Axis")
                .selected_text(field_x.as_str())
                .show_ui(ui, |ui| {
                    for msg in fields.iter() {
                        ui.selectable_value(field_x, (*msg).to_owned(), *msg);
                    }
                });
        }
        // if fields are more than 1, show the combo boxes for the y_axis
        if field_y.is_some() {
            let field_y = field_y.as_mut().unwrap();
            let widget_label = if self.plot_lines.len() > 1 {
                "Y Axis 1"
            } else {
                "Y Axis"
            };
            egui::ComboBox::from_label(widget_label)
                .selected_text(field_y.as_str())
                .show_ui(ui, |ui| {
                    for msg in fields.iter() {
                        ui.selectable_value(field_y, (*msg).to_owned(), *msg);
                    }
                });
        }
        // check how many fields are left and how many are selected
        let fields_selected = self.plot_lines.len() + 1;
        let fields_left_to_draw = fields.len().saturating_sub(2);
        for i in 0..fields_left_to_draw.min(fields_selected.saturating_sub(2)) {
            let field = &mut self.plot_lines.get_mut(1 + i).unwrap().field_y;
            let widget_label = format!("Y Axis {}", i + 2);
            egui::ComboBox::from_label(widget_label)
                .selected_text(field.as_str())
                .show_ui(ui, |ui| {
                    for msg in fields.iter() {
                        ui.selectable_value(field, (*msg).to_owned(), *msg);
                    }
                });
            self.plot_lines[1 + i].field_y = field.clone();
        }

        // if we have fields left, show the add button
        let fields_left_to_draw = fields.len().saturating_sub(fields_selected);
        if fields_left_to_draw > 0
            && ui
                .button("Add Y Axis")
                .on_hover_text("Add another Y axis")
                .clicked()
        {
            self.plot_lines
                .push(PlotLineSettings::new(fields[fields_selected].to_string()));
        }

        // update fields and flag for active plot
        self.field_x = field_x.unwrap_or_default();
        if field_y.is_some() {
            if self.plot_lines.first().is_none() {
                self.plot_lines
                    .push(PlotLineSettings::new(field_y.unwrap()));
            } else {
                self.plot_lines[0].field_y = field_y.unwrap();
            }
            self.plot_active = true;
        } else {
            self.plot_active = false;
        }
    }
}
