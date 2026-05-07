mod command;
mod default;
mod messages_viewer;
mod pid_drawing_tool;
mod plot;

use egui::Ui;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use strum_macros::{self, EnumIter, EnumMessage};

use crate::{
    mavlink::{CommandPacket, TimedMessage},
    utils::id::PaneId,
};

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
    fn ui(&mut self, ui: &mut Ui) -> PaneResponse;

    /// Updates the pane with the latest telemetry message, if any.
    fn update(&mut self, _message: Option<&TimedMessage>) {}

    /// Returns true when the pane needs all historical messages replayed
    /// (e.g. after plot settings change).
    fn needs_full_history(&self) -> bool {
        false
    }

    /// Drains outgoing commands from the pane.
    fn drain_outgoing_commands(&mut self) -> Vec<CommandPacket> {
        Vec::new()
    }

    /// Initializes the pane with the given pane ID.
    fn init(&mut self, _pane_id: PaneId) {}
}

impl PaneBehavior for Pane {
    fn ui(&mut self, ui: &mut Ui) -> PaneResponse {
        self.pane.ui(ui)
    }

    fn update(&mut self, message: Option<&TimedMessage>) {
        self.pane.update(message)
    }

    fn needs_full_history(&self) -> bool {
        self.pane.needs_full_history()
    }

    fn drain_outgoing_commands(&mut self) -> Vec<CommandPacket> {
        self.pane.drain_outgoing_commands()
    }

    fn init(&mut self, pane_id: PaneId) {
        self.pane.init(pane_id);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, EnumMessage, EnumIter)]
#[enum_dispatch]
pub enum PaneKind {
    Default(default::DefaultPane),

    #[strum(message = "Messages Viewer")]
    MessagesViewer(messages_viewer::MessagesViewerPane),

    #[strum(message = "Command")]
    CommandPane(command::CommandPane),

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
