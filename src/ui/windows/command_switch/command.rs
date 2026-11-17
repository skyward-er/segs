use std::time::Instant;

use crate::mavlink::reflection::MessageMap;
use egui::{Key, RichText, Ui};
use serde::{Deserialize, Serialize};

use super::{
    configurable::{self},
    direct::{self},
};

/// Symbols used as shortcut of the commands (after `/`): the command with id `n`
/// uses the `n`-th symbol. Append new symbols at the end, so that the shortcuts
/// of the commands already saved in the layouts do not change.
const SHORTCUT_SYMBOLS: &str = "1234567890.-+*";

/// Maximum number of commands, one for each shortcut symbol
pub const MAX_COMMANDS: usize = SHORTCUT_SYMBOLS.len();

/// Shortcut of a command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandShortcut {
    /// Shortcut on an egui [`Key`]
    Key(Key),
    /// Shortcut on a character that has no egui [`Key`] (e.g. `*` from the numpad)
    Text(&'static str),
}

impl CommandShortcut {
    fn from_symbol(symbol: char) -> Self {
        match symbol {
            '0' => Self::Key(Key::Num0),
            '1' => Self::Key(Key::Num1),
            '2' => Self::Key(Key::Num2),
            '3' => Self::Key(Key::Num3),
            '4' => Self::Key(Key::Num4),
            '5' => Self::Key(Key::Num5),
            '6' => Self::Key(Key::Num6),
            '7' => Self::Key(Key::Num7),
            '8' => Self::Key(Key::Num8),
            '9' => Self::Key(Key::Num9),
            '.' => Self::Key(Key::Period),
            '-' => Self::Key(Key::Minus),
            '+' => Self::Key(Key::Plus),
            '*' => Self::Text("*"),
            _ => unreachable!("Invalid shortcut symbol: {}", symbol),
        }
    }
}

/// Command Base on which all commands are built upon, containing common fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseCommand {
    pub id: usize,
    pub name: String,
    pub system_id: u8,
    pub message: Option<MessageMap>,

    // UI SETTINGS
    #[serde(skip)]
    pub reply_state: ReplyState,
    #[serde(skip)]
    pub settings_window_visible: bool,
    #[serde(skip)]
    pub show_only_tc: bool,
}

impl BaseCommand {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            name: String::from("New Command"),
            system_id: 1,
            message: None,
            reply_state: ReplyState::default(),
            settings_window_visible: false,
            show_only_tc: false,
        }
    }

    /// Symbol of the shortcut that invokes the command once the command switch
    /// is open, `None` if the id has no shortcut
    pub fn shortcut_symbol(&self) -> Option<char> {
        self.id
            .checked_sub(1)
            .and_then(|index| SHORTCUT_SYMBOLS.chars().nth(index))
    }

    /// Shortcut that invokes the command once the command switch is open
    pub fn shortcut(&self) -> Option<CommandShortcut> {
        self.shortcut_symbol().map(CommandShortcut::from_symbol)
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum ReplyState {
    #[default]
    ReadyForInvocation,
    WaitingForReply(Instant),
    ExplicitAck,
    TimeoutNack,
    ExplicitNack,
}

impl ReplyState {
    pub fn reset(&mut self) {
        *self = ReplyState::ReadyForInvocation;
    }

    pub fn is_enabled(&self) -> bool {
        !matches!(self, ReplyState::WaitingForReply(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Command {
    Configurable(configurable::ConfigurableCommand),
    Direct(direct::DirectCommand),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CommandKind {
    Configurable,
    Direct,
}

impl CommandKind {
    fn from_command(command: &Command) -> Self {
        match command {
            Command::Configurable(_) => CommandKind::Configurable,
            Command::Direct(_) => CommandKind::Direct,
        }
    }
}

impl Command {
    pub fn direct(id: usize) -> Self {
        Command::Direct(direct::DirectCommand {
            base: BaseCommand::new(id),
        })
    }

    pub fn configurable(id: usize) -> Self {
        Command::Configurable(configurable::ConfigurableCommand::new(id))
    }

    pub fn base(&self) -> &BaseCommand {
        match self {
            Command::Configurable(cmd) => &cmd.base,
            Command::Direct(cmd) => &cmd.base,
        }
    }

    pub fn base_mut(&mut self) -> &mut BaseCommand {
        match self {
            Command::Configurable(cmd) => &mut cmd.base,
            Command::Direct(cmd) => &mut cmd.base,
        }
    }

    pub fn show_settings(&mut self, ui: &mut Ui) {
        // Common title and separator
        ui.label(RichText::new("Command Settings:").size(15.0));
        ui.separator();

        // Radio buttons to select the command type
        let current_kind = CommandKind::from_command(self);
        let mut command_kind = current_kind;
        ui.horizontal(|ui| {
            ui.radio_value(&mut command_kind, CommandKind::Configurable, "Configurable");
            ui.radio_value(&mut command_kind, CommandKind::Direct, "Direct");
        });
        // If the command kind is changed, update the command
        if command_kind != current_kind {
            let mut new_cmd = match command_kind {
                CommandKind::Configurable => Self::configurable(self.base().id),
                CommandKind::Direct => Self::direct(self.base().id),
            };
            *new_cmd.base_mut() = self.base().clone();
            *self = new_cmd;
        }

        match self {
            Command::Configurable(cmd) => configurable::show_command_settings(ui, cmd),
            Command::Direct(cmd) => direct::show_command_settings(ui, cmd),
        }
    }
}
