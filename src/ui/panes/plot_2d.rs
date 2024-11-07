use crate::ui::composable_view::PaneResponse;

use super::PaneBehavior;

use egui_plot::{Line, PlotPoints};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Plot2DPane {
    #[serde(skip)]
    pub hovered: bool,
}

impl PaneBehavior for Plot2DPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let modifiers = ui.input(|i| i.modifiers);
        let mut response = PaneResponse::default();

        let plot = egui_plot::Plot::new("plot");
        plot.show(ui, |plot_ui| {
            self.hovered = plot_ui.response().contains_pointer();
            if plot_ui.response().dragged() && modifiers.ctrl {
                println!("ctrl + drag");
                response.set_drag_started();
            }

            plot_ui.line(Line::new(PlotPoints::from(vec![[1.0, 0.0], [2.0, 10.0]])));
        });

        response
    }

    fn tab_title(&self) -> egui::WidgetText {
        "Plot".into()
    }

    fn contains_pointer(&self) -> bool {
        self.hovered
    }
}
