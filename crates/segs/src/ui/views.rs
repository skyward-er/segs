pub mod configuration;
pub mod operator;

use egui::{Align, Frame, Layout, Margin, Panel, Ui, Vec2};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use super::components::buttons;

/// View represents what the user is currently looking at, imagine this as the
/// index of a document, but instead of pages, we index over possible layouts of
/// the UI. This is useful to keep track of which panels should be visible, and
/// which should not, as well as to keep track of the state of each view.
#[enum_dispatch(ViewTrait)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum View {
    Configuration(configuration::ConfigurationView),
    Operator(operator::OperatorView),
}

#[enum_dispatch]
trait ViewTrait {
    fn top_bar_left_fn(&mut self, _ui: &mut Ui) {
        // Default implementation does nothing
    }

    fn top_bar_middle_fn(&mut self, _ui: &mut Ui) {
        // Default implementation does nothing
    }

    fn top_bar_right_fn(&mut self, _ui: &mut Ui) {
        // Default implementation does nothing
    }

    fn main_view_fn(&mut self, _ui: &mut Ui) {
        // Default implementation does nothing
    }
}

impl View {
    fn show_top_bar(&mut self, ui: &mut Ui) {
        Panel::top("top_panel")
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .inner_margin(Margin::symmetric(4, 3))
                    .fill(ui.style().visuals.panel_fill),
            )
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let width = ui.max_rect().width();
                    let window_controls_width = 75.;
                    let middle_width = 300.;
                    let right_side_width = (width - middle_width) / 2.;
                    let side_width = right_side_width - window_controls_width;

                    ui.add_space(window_controls_width);

                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.set_min_width(side_width);
                        self.top_bar_left_fn(ui);
                    });

                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.set_width(middle_width);
                        self.top_bar_middle_fn(ui);
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.set_min_width(side_width);
                        ui.add_space(3.);

                        // Theme toggle button
                        buttons::theme_toggle(ui);

                        self.top_bar_right_fn(ui);
                    });
                });
            });
    }

    pub fn show_inside(&mut self, ui: &mut Ui) {
        self.show_top_bar(ui);
        self.main_view_fn(ui);
    }
}

impl Default for View {
    fn default() -> Self {
        Self::Configuration(configuration::ConfigurationView::default())
    }
}
