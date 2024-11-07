mod default;
mod messages_viewer;
mod plot_2d;

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use super::composable_view::PaneResponse;

#[enum_dispatch(Pane)]
pub trait PaneBehavior {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse;
    fn tab_title(&self) -> egui::WidgetText;
    fn contains_pointer(&self) -> bool;
}

// An enum to represent the different widgets available to the user.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[enum_dispatch]
pub enum Pane {
    Default(default::DefaultPane),
    MessagesViewer(messages_viewer::MessagesViewerPane),
    Plot2D(plot_2d::Plot2DPane),
}

impl Default for Pane {
    fn default() -> Self {
        Pane::Default(default::DefaultPane::default())
    }
}
