mod command;
mod configurable;
mod direct;
mod state;

use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use egui::{Color32, Key, ModifierNames, Modifiers, RichText, Ui, Vec2};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    error::ErrInstrument,
    mavlink::{MavHeader, MavMessage, reflection::MapConvertible},
    ui::{
        shortcuts::{ShortcutAppState, ShortcutHandlerExt, ShortcutLease},
        windows::command_switch::command::ReplyState,
    },
};

use command::{BaseCommand, Command};

const MAXIMUM_REPLY_TIMEOUT: Duration = Duration::from_secs(3);

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
            .shortcuts()
            .lock()
            .capture_actions(ui.id().with("command_switch_lease"), Box::new(()), |_| {
                vec![(Modifiers::NONE, Key::Slash, true)]
            })
            .unwrap_or_default();
        if !self.commands.is_empty() && slash_pressed {
            // First reset the reply state of all commands
            for cmd in self.commands.iter_mut() {
                cmd.base_mut().reply_state.reset();
            }
            // Then toggle the visibility of the command switch window
            self.state.switch_command();
        }
        if self.state.is_command_switch() {
            // Update the the state of the expired commands
            for cmd in self.commands.iter_mut() {
                if let ReplyState::WaitingForReply(instant) = cmd.base().reply_state {
                    if instant.elapsed() > MAXIMUM_REPLY_TIMEOUT {
                        cmd.base_mut().reply_state = ReplyState::TimeoutNack;
                    }
                }
            }
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

    pub fn handle_acknowledgements(&mut self, messages: Vec<&MavMessage>) {
        let mut acks_ids = HashSet::new();
        let mut nacks_ids = HashSet::new();
        for message in messages {
            match message {
                MavMessage::ACK_TM(ack) => {
                    acks_ids.insert(ack.recv_msgid as usize);
                }
                MavMessage::NACK_TM(nack) => {
                    nacks_ids.insert(nack.recv_msgid as usize);
                }
                _ => continue,
            }
        }
        for cmd in self.commands.iter_mut() {
            let base = cmd.base_mut();
            if let ReplyState::WaitingForReply(instant) = base.reply_state {
                if acks_ids.contains(&base.id) {
                    base.reply_state = ReplyState::ExplicitAck;
                } else if nacks_ids.contains(&base.id) {
                    base.reply_state = ReplyState::ExplicitNack;
                }
            }
        }
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
            // check if there are any commands with visible parameters window
            let cmd = commands
                .iter_mut()
                .find(|cmd| matches!(cmd, Command::Configurable(c) if c.parameters_window_visible));

            // make sure the state is coherent with individual window visibility
            // settings (since they change from inside interaction)
            if cmd.is_none() {
                state.set_command_switch();
            } else {
                state.set_configurable_command_dialog();
            }

            // show the appropriate ui based
            if let Some(Command::Configurable(cmd)) = cmd {
                cmd.show_operative_parameters(state, messages_to_send, ui);
            } else {
                show_switch_list(state, commands, messages_to_send, ui);
            }
        });
    if !visible {
        state.hide();
    }
}

struct CommandSwitchLease;

impl ShortcutLease for CommandSwitchLease {
    fn once_ended(&mut self, state: &mut ShortcutAppState) {
        state.is_command_switch_active = false;
    }

    fn while_active(&mut self, state: &mut ShortcutAppState) {
        state.is_command_switch_active = true;
    }
}

fn show_switch_list(
    state: &mut state::StateManager,
    commands: &mut [Command],
    messages_to_send: &mut Vec<(MavHeader, MavMessage)>,
    ui: &mut Ui,
) {
    for cmd in commands.iter_mut() {
        #[cfg(target_os = "macos")]
        let is_mac = true;
        #[cfg(not(target_os = "macos"))]
        let is_mac = false;
        let shortcut_text = cmd.base().shortcut_comb()[1].format(&ModifierNames::SYMBOLS, is_mac);
        let text = RichText::new(format!("[{}] {}", shortcut_text, &cmd.base().name)).size(17.0);
        let cmd_btn = ui
            .add_enabled_ui(cmd.base().reply_state.is_enabled(), |ui| {
                let valid_fill = ui
                    .visuals()
                    .widgets
                    .inactive
                    .bg_fill
                    .lerp_to_gamma(Color32::GREEN, 0.3);
                let missing_fill = ui
                    .visuals()
                    .widgets
                    .inactive
                    .bg_fill
                    .lerp_to_gamma(Color32::YELLOW, 0.3);
                let invalid_fill = ui
                    .visuals()
                    .widgets
                    .inactive
                    .bg_fill
                    .lerp_to_gamma(Color32::RED, 0.3);
                let mut btn = egui::Button::new(text);
                btn = match cmd.base().reply_state {
                    ReplyState::ReadyForInvocation | ReplyState::WaitingForReply(_) => btn,
                    ReplyState::ExplicitAck => btn.fill(valid_fill),
                    ReplyState::ExplicitNack => btn.fill(invalid_fill),
                    ReplyState::TimeoutNack => btn.fill(missing_fill),
                };
                ui.add_sized(Vec2::new(300.0, 10.0), btn)
            })
            .inner;

        // catch called shortcuts
        let shortcut_pressed = ui.ctx().shortcuts().lock().capture_actions(
            ui.id().with("shortcut_lease"),
            Box::new(CommandSwitchLease),
            |s| {
                if s.is_operation_mode() && cmd.base().reply_state.is_enabled() {
                    vec![(Modifiers::NONE, cmd.base().shortcut_keys()[1], true)]
                } else {
                    vec![]
                }
            },
        );
        let actionated = shortcut_pressed.unwrap_or_default() || cmd_btn.clicked();

        if actionated {
            match cmd {
                Command::Configurable(cmd) => {
                    // change state to show the configurable command dialog
                    state.set_configurable_command_dialog();
                    cmd.parameters_window_visible = true;
                }
                Command::Direct(cmd) => {
                    let BaseCommand {
                        system_id,
                        message,
                        reply_state,
                        ..
                    } = &mut cmd.base;
                    if let Some(map) = message {
                        // append the message to the list of messages to send
                        let header = MavHeader {
                            system_id: *system_id,
                            ..Default::default()
                        };
                        messages_to_send
                            .push((header, MavMessage::from_map(map.clone()).log_unwrap()));
                        // Update the reply state to waiting for reply
                        *reply_state = ReplyState::WaitingForReply(Instant::now());
                    }
                }
            }
        }
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
            if commands
                .iter()
                .all(|cmd| !cmd.base().settings_window_visible)
            {
                show_catalog_list(ui, commands);
            } else {
                let cmd = commands
                    .iter_mut()
                    .find(|cmd| cmd.base().settings_window_visible)
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
            cmd.base_mut().settings_window_visible = true;
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
