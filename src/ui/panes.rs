mod default;
mod messages_viewer;
mod pid_drawing_tool;
pub mod plot;
mod plot;

use egui_tiles::TileId;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use strum_macros::{self, EnumIter, EnumMessage};

use super::composable_view::PaneResponse;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
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
    fn ui(&mut self, ui: &mut egui::Ui, tile_id: TileId) -> PaneResponse;
    fn contains_pointer(&self) -> bool;
}

impl PaneBehavior for Pane {
    fn ui(&mut self, ui: &mut egui::Ui, tile_id: TileId) -> PaneResponse {
        self.pane.ui(ui, tile_id)
    }

    fn contains_pointer(&self) -> bool {
        self.pane.contains_pointer()
    }
}

// An enum to represent the diffent kinds of widget available to the user.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, EnumMessage, EnumIter)]
#[enum_dispatch]
pub enum PaneKind {
    Default(default::DefaultPane),

    #[strum(message = "Messages Viewer")]
    MessagesViewer(messages_viewer::MessagesViewerPane),

    #[strum(message = "Plot 2D")]
    Plot2D(plot::Plot2DPane),
    PidDrawingTool(pid_drawing_tool::PidPane),
}

impl Default for PaneKind {
    fn default() -> Self {
        PaneKind::Default(default::DefaultPane::default())
    }
}
