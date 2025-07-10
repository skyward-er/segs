mod configurable;
mod direct;
mod state;

use egui::{Key, KeyboardShortcut, ModifierNames, Modifiers, RichText, Ui, Vec2};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    error::ErrInstrument,
    mavlink::{
        MavHeader, MavMessage,
        reflection::{MapConvertible, MessageMap},
    },
    ui::windows::command_switch::configurable::ConfigurableCommand,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandSwitchWindow {
    commands: Vec<Command>,
    #[serde(skip)]
    state: state::StateManager,
    #[serde(skip)]
    messages_to_send: Vec<(MavHeader, MavMessage)>,
}

impl CommandSwitchWindow {
    pub fn toggle_open_state(&mut self) {
        self.state.switch_catalog();
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let slash_pressed = ui
            .ctx()
            .input_mut(|i| i.consume_key(Modifiers::NONE, Key::Slash));
        if !self.commands.is_empty() && slash_pressed {
            // If the slash key is pressed, toggle the visibility of the command switch window
            self.state.switch_command();
        }
        if self.state.is_command_switch() {
            // Show the command switch window
            show_command_switch_window(ui, self);
        } else {
            show_command_catalog(ui, self);
        }
    }

    pub fn consume_messages_to_send(&mut self) -> Vec<(MavHeader, MavMessage)> {
        if self.messages_to_send.is_empty() {
            return vec![];
        }
        std::mem::take(&mut self.messages_to_send)
    }
}

fn show_command_switch_window(ui: &mut Ui, window: &mut CommandSwitchWindow) {
    let CommandSwitchWindow {
        state,
        commands,
        messages_to_send,
    } = window;
    let mut visible = state.is_command_switch();
    egui::Window::new("Command Switch")
        .id(ui.id().with("command_switch_window"))
        .max_width(300.0)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut visible)
        .show(ui.ctx(), |ui| {
            for cmd in commands.iter_mut() {
                #[cfg(target_os = "macos")]
                let is_mac = true;
                #[cfg(not(target_os = "macos"))]
                let is_mac = false;
                let shortcut_text =
                    cmd.base().shortcut_comb()[1].format(&ModifierNames::SYMBOLS, is_mac);
                let text =
                    RichText::new(format!("[{}] {}", shortcut_text, &cmd.base().name)).size(17.0);
                let cmd_btn = ui.add_sized(Vec2::new(300.0, 10.0), egui::Button::new(text));

                // catch called shortcuts
                let shortcut_pressed = ui
                    .ctx()
                    .input_mut(|i| i.consume_shortcut(&cmd.base().shortcut_comb()[1]));
                let actionated = shortcut_pressed || cmd_btn.clicked();
                let msg = actionated.then(|| cmd.base().message.clone()).flatten();
                if let Some(map) = msg {
                    let header = MavHeader {
                        system_id: cmd.base().system_id,
                        ..Default::default()
                    };
                    messages_to_send.push((header, MavMessage::from_map(map).log_unwrap()));
                }
                if actionated {
                    state.hide();
                }
            }
        });
    if !visible {
        state.hide();
    }
}

fn show_command_catalog(ui: &mut Ui, window: &mut CommandSwitchWindow) {
    let CommandSwitchWindow {
        state, commands, ..
    } = window;
    let mut visible = state.is_catalog();
    egui::Window::new("Command Switch")
        .id(ui.id().with("command_switch_settings_window"))
        .max_width(300.0)
        .resizable(false)
        .collapsible(false)
        .open(&mut visible)
        .show(ui.ctx(), |ui| {
            if commands.iter().all(|cmd| !cmd.base().ui_visible) {
                show_catalog_list(ui, commands);
            } else {
                let cmd = commands
                    .iter_mut()
                    .find(|cmd| cmd.base().ui_visible)
                    .log_unwrap();
                cmd.show_settings(ui);
            }
        });
    if !visible {
        state.hide();
    }
}

fn show_catalog_list(ui: &mut Ui, commands: &mut Vec<Command>) {
    for cmd in commands.iter_mut() {
        #[cfg(target_os = "macos")]
        let is_mac = true;
        #[cfg(not(target_os = "macos"))]
        let is_mac = false;
        let shortcut_text = cmd
            .base()
            .shortcut_comb()
            .into_iter()
            .map(|s| s.format(&ModifierNames::SYMBOLS, is_mac))
            .join(" ");
        let text = RichText::new(format!("[{}] {}", shortcut_text, &cmd.base().name)).size(17.0);
        let cmd_btn = ui.add_sized(Vec2::new(300.0, 10.0), egui::Button::new(text));
        if cmd_btn.clicked() {
            cmd.base_mut().ui_visible = true;
        }
    }
    if commands.len() < 9 {
        let plus_btn = ui.add_sized(
            Vec2::new(300.0, 10.0),
            egui::Button::new(RichText::new("+").size(17.0)),
        );
        if plus_btn.clicked() {
            commands.push(Command::direct(commands.len() + 1));
        }
    }
}

/// Command Base on which all commands are built upon, containing common fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct BaseCommand {
    id: usize,
    name: String,
    system_id: u8,
    message: Option<MessageMap>,

    // UI SETTINGS
    #[serde(skip)]
    ui_visible: bool,
    #[serde(skip)]
    show_only_tc: bool,
}

impl BaseCommand {
    fn new(id: usize) -> Self {
        Self {
            id,
            name: String::from("New Command"),
            system_id: 1,
            message: None,
            ui_visible: false,
            show_only_tc: false,
        }
    }

    fn shortcut_comb(&self) -> Vec<KeyboardShortcut> {
        let key = match self.id {
            0 => Key::Num0,
            1 => Key::Num1,
            2 => Key::Num2,
            3 => Key::Num3,
            4 => Key::Num4,
            5 => Key::Num5,
            6 => Key::Num6,
            7 => Key::Num7,
            8 => Key::Num8,
            9 => Key::Num9,
            _ => panic!("Command ID must be between 0 and 9"),
        };
        vec![
            KeyboardShortcut::new(Modifiers::NONE, Key::Slash),
            KeyboardShortcut::new(Modifiers::NONE, key),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Command {
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
    fn direct(id: usize) -> Self {
        Command::Direct(direct::DirectCommand {
            base: BaseCommand::new(id),
        })
    }

    fn configurable(id: usize) -> Self {
        Command::Configurable(configurable::ConfigurableCommand::new(id))
    }

    fn base(&self) -> &BaseCommand {
        match self {
            Command::Configurable(cmd) => &cmd.base,
            Command::Direct(cmd) => &cmd.base,
        }
    }

    fn base_mut(&mut self) -> &mut BaseCommand {
        match self {
            Command::Configurable(cmd) => &mut cmd.base,
            Command::Direct(cmd) => &mut cmd.base,
        }
    }

    fn show_settings(&mut self, ui: &mut Ui) {
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
            *self = match command_kind {
                CommandKind::Configurable => Command::Configurable(ConfigurableCommand {
                    base: self.base().clone(),
                    ..ConfigurableCommand::new(self.base().id)
                }),
                CommandKind::Direct => Command::Direct(direct::DirectCommand {
                    base: self.base().clone(),
                }),
            };
        }

        match self {
            Command::Configurable(cmd) => configurable::show_command_settings(ui, cmd),
            Command::Direct(cmd) => direct::show_command_settings(ui, cmd),
        }
    }
}
