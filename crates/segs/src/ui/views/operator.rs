use serde::{Deserialize, Serialize};

use crate::{app::AppContext, ui::views::ViewTrait};

/// View subtype representing the different operator views available when the
/// user is in the Operator mode.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorView {
    selected_layout: String,
}

impl ViewTrait for OperatorView {
    fn show_activities(&mut self, _ui: &mut egui::Ui, _appctx: &mut AppContext) {}

    fn show_left_panel(&mut self, _ui: &mut egui::Ui, _appctx: &mut AppContext) {}

    fn show_main_view(&mut self, _ui: &mut egui::Ui, _appctx: &mut AppContext) {}
}
