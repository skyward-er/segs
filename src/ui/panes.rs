mod default;
mod messages_viewer;
mod pid_drawing_tool;
pub mod plot;

use egui_tiles::TileId;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use strum_macros::{self, EnumIter, EnumMessage};

use crate::mavlink::{MavMessage, TimedMessage};

use super::app::PaneResponse;

#[derive(Clone, PartialEq, Default, Serialize, Deserialize, Debug)]
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
    /// Renders the UI of the pane.
    fn ui(&mut self, ui: &mut egui::Ui, tile_id: TileId) -> PaneResponse;

    /// Whether the pane contains the pointer.
    fn contains_pointer(&self) -> bool;

    /// Updates the pane state. This method is called before `ui` to allow the
    /// pane to update its state based on the messages received.
    fn update(&mut self, _messages: &[TimedMessage]) {}

    /// Returns the ID of the messages this pane is interested in, if any.
    fn get_message_subscription(&self) -> Option<u32> {
        None
    }

    /// Checks whether the full message history should be sent to the pane.
    fn should_send_message_history(&self) -> bool {
        false
    }

    /// Drains the outgoing messages from the pane.
    fn drain_outgoing_messages(&mut self) -> Vec<MavMessage> {
        Vec::new()
    }
}

impl PaneBehavior for Pane {
    fn ui(&mut self, ui: &mut egui::Ui, tile_id: TileId) -> PaneResponse {
        self.pane.ui(ui, tile_id)
    }

    fn contains_pointer(&self) -> bool {
        self.pane.contains_pointer()
    }

    fn update(&mut self, messages: &[TimedMessage]) {
        self.pane.update(messages)
    }

    fn get_message_subscription(&self) -> Option<u32> {
        self.pane.get_message_subscription()
    }

    fn should_send_message_history(&self) -> bool {
        self.pane.should_send_message_history()
    }

    fn drain_outgoing_messages(&mut self) -> Vec<MavMessage> {
        self.pane.drain_outgoing_messages()
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

    #[strum(message = "Pid")]
    Pid(pid_drawing_tool::PidPane),
}

impl Default for PaneKind {
    fn default() -> Self {
        PaneKind::Default(default::DefaultPane::default())
    }
}
