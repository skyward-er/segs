use egui::{Button, RichText, Sense, Ui};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    mavlink::{CommandPacket, MAVLINK_PROFILE, TimedMessage},
    ui::app::PaneResponse,
};

use super::PaneBehavior;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandPane {
    /// Button label shown to the user.
    text: String,
    text_size: f32,
    /// Name of the selected command, if any.
    selected_command: Option<String>,
    /// Current parameter values (one per non-hidden param in the command def).
    param_values: Vec<u64>,

    #[serde(skip)]
    settings_visible: bool,
    #[serde(skip)]
    commands_to_send: Vec<CommandPacket>,
}

impl Default for CommandPane {
    fn default() -> Self {
        Self {
            text: "Command".to_string(),
            text_size: 16.0,
            selected_command: None,
            param_values: Vec::new(),
            settings_visible: false,
            commands_to_send: Vec::new(),
        }
    }
}

impl PartialEq for CommandPane {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.text_size == other.text_size
            && self.selected_command == other.selected_command
            && self.param_values == other.param_values
    }
}

impl PaneBehavior for CommandPane {
    #[profiling::function]
    fn ui(&mut self, ui: &mut Ui) -> PaneResponse {
        let mut response = PaneResponse::default();

        let parent = ui
            .scope(|ui| {
                let btn_text = RichText::new(&self.text).size(self.text_size).strong();
                let btn = Button::new(btn_text).sense(egui::Sense::click());

                ui.allocate_rect(ui.max_rect(), Sense::click());
                let btn_rect = ui.max_rect().shrink(2.0);
                let btn_res = ui.put(btn_rect, btn);

                btn_res.context_menu(|ui| command_menu(ui, self));

                if btn_res.clicked() {
                    if let Some(ref name) = self.selected_command.clone() {
                        if let Some(registry) = MAVLINK_PROFILE.get() {
                            if let Some(def) = registry.get_command_by_name(name) {
                                info!("Sending command {}", name);
                                let cmd = CommandPacket {
                                    command_id: def.command_id,
                                    param_values: self.param_values.clone(),
                                };
                                self.commands_to_send.push(cmd);
                            }
                        }
                    }
                }
            })
            .response;

        if parent.interact(egui::Sense::click_and_drag()).dragged() {
            response.set_drag_started();
        }

        let mut window_visible = self.settings_visible;
        egui::Window::new("Command Settings")
            .id(ui.auto_id_with("command_settings"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut window_visible)
            .show(ui.ctx(), |ui| command_settings(ui, self));
        self.settings_visible = window_visible;

        response
    }

    fn update(&mut self, _message: Option<&TimedMessage>) {}

    fn drain_outgoing_commands(&mut self) -> Vec<CommandPacket> {
        self.commands_to_send.drain(..).collect()
    }
}

fn command_menu(ui: &mut Ui, pane: &mut CommandPane) {
    if ui.button("Settings…").clicked() {
        pane.settings_visible = true;
        ui.close_menu();
    }
}

fn command_settings(ui: &mut Ui, pane: &mut CommandPane) {
    ui.set_min_width(220.0);

    ui.horizontal(|ui| {
        ui.label("Label:");
        ui.text_edit_singleline(&mut pane.text);
    });
    ui.horizontal(|ui| {
        ui.label("Text Size:");
        ui.add(egui::Slider::new(&mut pane.text_size, 11.0..=25.0));
    });

    ui.separator();

    let Some(registry) = MAVLINK_PROFILE.get() else {
        ui.label("Registry not loaded.");
        return;
    };

    // Command selector dropdown
    let selected_text = pane
        .selected_command
        .as_deref()
        .unwrap_or("Select a command");
    let prev_cmd = pane.selected_command.clone();

    egui::ComboBox::from_id_salt(ui.id().with("command_selector"))
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            for cmd in &registry.commands {
                ui.selectable_value(
                    &mut pane.selected_command,
                    Some(cmd.name.clone()),
                    &cmd.name,
                );
            }
        });

    // Reset param_values when command changes
    if pane.selected_command != prev_cmd {
        if let Some(ref name) = pane.selected_command {
            if let Some(def) = registry.get_command_by_name(name) {
                pane.param_values = def.default_param_values();
            }
        } else {
            pane.param_values.clear();
        }
    }

    // Show parameter editors
    if let Some(ref name) = pane.selected_command.clone() {
        if let Some(def) = registry.get_command_by_name(name) {
            if !def.params.is_empty() {
                ui.separator();
                ui.label("Parameters:");
                ui.group(|ui| {
                    for (i, param) in def.params.iter().enumerate() {
                        if i >= pane.param_values.len() {
                            break;
                        }
                        ui.horizontal(|ui| {
                            ui.label(format!("{}:", param.name.to_uppercase()));
                            if !param.states.is_empty() {
                                // Enum-like parameter: show a combo box with state names
                                let cur_val = pane.param_values[i];
                                let cur_label = param
                                    .states
                                    .iter()
                                    .find(|s| {
                                        matches!(
                                            &s.value,
                                            crate::cosmos::StateValue::Exact(v) if *v == cur_val
                                        )
                                    })
                                    .map(|s| s.name.as_str())
                                    .unwrap_or("?");
                                egui::ComboBox::from_id_salt(ui.id().with(i))
                                    .selected_text(cur_label)
                                    .show_ui(ui, |ui| {
                                        for state in &param.states {
                                            if let crate::cosmos::StateValue::Exact(v) =
                                                &state.value
                                            {
                                                ui.selectable_value(
                                                    &mut pane.param_values[i],
                                                    *v,
                                                    &state.name,
                                                );
                                            }
                                        }
                                    });
                            } else {
                                // Numeric parameter: drag value
                                let max_val = match param.bit_size {
                                    8 => u8::MAX as u64,
                                    16 => u16::MAX as u64,
                                    32 => u32::MAX as u64,
                                    _ => u64::MAX,
                                };
                                ui.add(
                                    egui::DragValue::new(&mut pane.param_values[i])
                                        .range(0..=max_val),
                                );
                            }
                        });
                    }
                });
            }
        }
    }
}
