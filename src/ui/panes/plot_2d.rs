use crate::ui::composable_view::PaneResponse;

use super::PaneBehavior;

use egui_plot::{Line, PlotPoints};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plot2DPane {
    n_points: u32,
    frequency: f64,
    width: f32,
    color: egui::Color32,

    #[serde(skip)]
    pub contains_pointer: bool,

    #[serde(skip)]
    settings_visible: bool,
}

impl Default for Plot2DPane {
    fn default() -> Self {
        Self {
            contains_pointer: false,
            settings_visible: false,
            n_points: 2,
            frequency: 1.0,
            width: 1.0,
            color: egui::Color32::from_rgb(0, 120, 240),
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

        let mut window_visible = self.settings_visible;
        egui::Window::new("Plot Settings")
            .id(ui.id())
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut window_visible)
            .show(ui.ctx(), |ui| self.settings_window(ui));
        self.settings_visible = window_visible;

        let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);

        let plot = egui_plot::Plot::new("plot");
        plot.show(ui, |plot_ui| {
            self.contains_pointer = plot_ui.response().contains_pointer();
            if plot_ui.response().dragged() && ctrl_pressed {
                println!("ctrl + drag");
                response.set_drag_started();
            }
            let points: Vec<[f64; 2]> = (0..self.n_points)
                .map(|i| i as f64 * 100.0 / (self.n_points - 1) as f64)
                .map(|i| [i, (i * std::f64::consts::PI * 2.0 * self.frequency).sin()])
                .collect();
            plot_ui.line(
                Line::new(PlotPoints::from(points))
                    .color(self.color)
                    .width(self.width),
            );
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
    }

    fn settings_window(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new(ui.id())
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                ui.label("Size:");
                ui.add(egui::Slider::new(&mut self.n_points, 2..=1000).text("Points"));
                ui.end_row();

                ui.label("Frequency:");
                ui.add(egui::Slider::new(&mut self.frequency, 0.1..=10.0).text("Hz"));
                ui.end_row();

                ui.label("Color:");
                ui.color_edit_button_srgba(&mut self.color);
                ui.end_row();

                ui.label("Width:");
                ui.add(egui::Slider::new(&mut self.width, 0.1..=10.0).text("pt"));
                ui.end_row();
            });
    }
}
