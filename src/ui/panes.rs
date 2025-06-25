mod command;
mod default;
mod messages_viewer;
mod pid_drawing_tool;
mod plot;
mod valve_control;

use egui::Ui;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use skyward_mavlink::mavlink::MavHeader;
use strum_macros::{self, EnumIter, EnumMessage};

use crate::{
    mavlink::{MavMessage, TimedMessage},
    utils::id::PaneId,
};

use super::{app::PaneResponse, shortcuts::ShortcutHandler};

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
    fn ui(&mut self, ui: &mut Ui, shortcut_handler: &mut ShortcutHandler) -> PaneResponse;

    /// Updates the pane state. This method is called before `ui` to allow the
    /// pane to update its state based on the messages received.
    fn update(&mut self, _messages: &[&TimedMessage]) {}

    /// Returns the ID of the messages this pane is interested in, if any.
    fn get_message_subscriptions(&self) -> Box<dyn Iterator<Item = u32>> {
        Box::new(None.into_iter())
    }

    /// Checks whether the full message history should be sent to the pane.
    fn should_send_message_history(&self) -> bool {
        false
    }

    /// Drains the outgoing messages from the pane.
    fn drain_outgoing_messages(&mut self) -> Vec<(MavHeader, MavMessage)> {
        Vec::new()
    }

    /// Initializes the pane with the given pane ID. This is called when the pane is inserted into the layout.
    fn init(&mut self, _pane_id: PaneId) {}
}

impl PaneBehavior for Pane {
    fn ui(&mut self, ui: &mut Ui, shortcut_handler: &mut ShortcutHandler) -> PaneResponse {
        self.pane.ui(ui, shortcut_handler)
    }

    fn update(&mut self, messages: &[&TimedMessage]) {
        self.pane.update(messages)
    }

    fn get_message_subscriptions(&self) -> Box<dyn Iterator<Item = u32>> {
        self.pane.get_message_subscriptions()
    }

    fn should_send_message_history(&self) -> bool {
        self.pane.should_send_message_history()
    }

    fn drain_outgoing_messages(&mut self) -> Vec<(MavHeader, MavMessage)> {
        self.pane.drain_outgoing_messages()
    }

    fn init(&mut self, pane_id: PaneId) {
        self.pane.init(pane_id);
    }
}

// An enum to represent the diffent kinds of widget available to the user.
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

    #[strum(message = "Valve Control")]
    ValveControl(valve_control::ValveControlPane),
}

impl Default for PaneKind {
    fn default() -> Self {
        PaneKind::Default(default::DefaultPane::default())
    }
}
