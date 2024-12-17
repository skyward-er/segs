mod source_window;

use egui::{Color32, Vec2b};
use egui_plot::{Legend, Line, PlotPoints};
use serde::{Deserialize, Serialize};
use source_window::{sources_window, SourceSettings};

use crate::{
    error::ErrInstrument,
    mavlink::{
        extract_from_message, MavlinkResult, MessageData, MessageView, TimedMessage,
        ROCKET_FLIGHT_TM_DATA,
    },
    ui::composable_view::PaneResponse,
    MSG_MANAGER,
};

use super::PaneBehavior;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plot2DPane {
    // UI settings
    #[serde(skip)]
    pub contains_pointer: bool,
    #[serde(skip)]
    settings_visible: bool,
    line_settings: Vec<LineSettings>,
    plot_active: bool,
    view: PlotMessageView,
}

impl Plot2DPane {
    pub fn new(id: egui::Id) -> Self {
        Self {
            contains_pointer: false,
            settings_visible: false,
            line_settings: vec![],
            plot_active: false,
            view: PlotMessageView::new(id),
        }
    }
}

impl PartialEq for Plot2DPane {
    fn eq(&self, other: &Self) -> bool {
        self.view.settings == other.view.settings
            && self.line_settings == other.line_settings
            && self.plot_active == other.plot_active
    }
}

impl PaneBehavior for Plot2DPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let mut response = PaneResponse::default();

        let Self {
            line_settings: plot_lines,
            settings_visible,
            plot_active,
            view,
            ..
        } = self;

        let mut settings = SourceSettings::new(&mut view.settings, plot_lines);
        egui::Window::new("Plot Settings")
            .id(ui.auto_id_with("plot_settings")) // TODO: fix this issue with ids
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(settings_visible)
            .show(ui.ctx(), |ui| sources_window(ui, &mut settings));
        // if settings are changed, invalidate the cache
        view.cache_valid = !settings.are_sources_changed();
        // if there are no fields, do not plot
        *plot_active = !settings.fields_empty();

        let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);

        let mut plot_lines = Vec::new();
        if self.plot_active {
            MSG_MANAGER
                .get()
                .unwrap()
                .lock()
                .refresh_view(view)
                .log_expect("MessageView may be invalid");
            let acc_points = &view.points;

            let field_x = &view.settings.x_field;
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
                .context_menu(|ui| show_menu(ui, settings_visible));
        });

        response
    }

    fn contains_pointer(&self) -> bool {
        self.contains_pointer
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlotMessageView {
    // == Settings from the UI ==
    settings: MsgSources,
    // == Data ==
    #[serde(skip)]
    points: Vec<(f64, Vec<f64>)>,
    // == Internal ==
    id: egui::Id,
    #[serde(skip)]
    cache_valid: bool,
}

impl PlotMessageView {
    fn new(id: egui::Id) -> Self {
        Self {
            settings: Default::default(),
            points: Vec::new(),
            id,
            cache_valid: false,
        }
    }
}

impl MessageView for PlotMessageView {
    fn widget_id(&self) -> &egui::Id {
        &self.id
    }

    fn id_of_interest(&self) -> u32 {
        self.settings.msg_id
    }

    fn is_valid(&self) -> bool {
        self.cache_valid
    }

    fn populate_view(&mut self, msg_slice: &[TimedMessage]) -> MavlinkResult<()> {
        self.points.clear();
        let MsgSources {
            x_field, y_fields, ..
        } = &self.settings;
        for msg in msg_slice {
            let x: f64 = extract_from_message(&msg.message, [x_field])?[0];
            let ys: Vec<f64> = extract_from_message(&msg.message, y_fields)?;
            self.points.push((x, ys));
        }
        Ok(())
    }

    fn update_view(&mut self, msg_slice: &[TimedMessage]) -> MavlinkResult<()> {
        let MsgSources {
            x_field, y_fields, ..
        } = &self.settings;
        for msg in msg_slice {
            let x: f64 = extract_from_message(&msg.message, [x_field])?[0];
            let ys: Vec<f64> = extract_from_message(&msg.message, y_fields)?;
            self.points.push((x, ys));
        }
        Ok(())
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
