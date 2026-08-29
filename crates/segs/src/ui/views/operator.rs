use crate::{
    app::AppContext,
    ui::{components::widget_renderer::show_widgets, grid::Grid, views::ViewTrait},
};
use egui::{CentralPanel, Frame};
use segs_ui::style::CtxStyleExt;

/// View state for operating the active layout.
#[derive(Default)]
pub struct OperatorView;

impl ViewTrait for OperatorView {
    fn show_main_view(&mut self, ui: &mut egui::Ui, appctx: &mut AppContext) {
        let app_style = ui.app_style();

        CentralPanel::default()
            .frame(Frame::new().fill(app_style.main_panels_fill))
            .show_inside(ui, |ui| {
                let rect = ui.available_rect_before_wrap();

                let Some(layout) = appctx.layouts.active() else {
                    return;
                };
                let grid = Grid::new(rect, layout.grid_settings);
                show_widgets(ui, &layout.widgets, &grid, &mut appctx.data_store);
            });
    }
}
