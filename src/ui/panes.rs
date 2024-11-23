mod default;
mod messages_viewer;
mod plot_2d;

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use super::composable_view::PaneResponse;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Pane {
    pub pane: PaneKind,
}

impl Pane {
    pub fn boxed(pane: PaneKind) -> Box<Self> {
        Box::new(Self { pane })
    }
}

#[enum_dispatch(PaneKind)]
pub trait PaneBehavior {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse;
    fn contains_pointer(&self) -> bool;
}

impl PaneBehavior for Pane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        self.pane.ui(ui)
    }

    fn contains_pointer(&self) -> bool {
        self.pane.contains_pointer()
    }
}

// An enum to represent the diffent kinds of widget available to the user.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[enum_dispatch]
pub enum PaneKind {
    Default(default::DefaultPane),
    MessagesViewer(messages_viewer::MessagesViewerPane),
    Plot2D(plot_2d::Plot2DPane),
}

impl Default for PaneKind {
    fn default() -> Self {
        PaneKind::Default(default::DefaultPane::default())
    }
}
