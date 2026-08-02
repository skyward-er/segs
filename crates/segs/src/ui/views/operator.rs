use egui::{CentralPanel, Frame};
use segs_ui::style::CtxStyleExt;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContext,
    ui::{components::widget_renderer::show_widgets, grid::Grid, views::ViewTrait},
};

/// View subtype representing the different operator views available when the
/// user is in the Operator mode.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorView {
    pub layout: String,
}

impl ViewTrait for OperatorView {
    fn show_main_view(&mut self, ui: &mut egui::Ui, appctx: &mut AppContext) {
        let app_style = ui.app_style();

        CentralPanel::default()
            .frame(Frame::new().fill(app_style.main_panels_fill))
            .show_inside(ui, |ui| {
                let rect = ui.available_rect_before_wrap();

                let grid = Grid::new(rect, appctx.layout.grid_settings);
                show_widgets(ui, &appctx.layout.widgets, &grid, &mut appctx.data_store);
            });
    }
}
