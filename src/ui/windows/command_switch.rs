use egui::{Key, KeyboardShortcut, ModifierNames, Modifiers, RichText, Ui, Vec2};
use itertools::Itertools;
use mavlink_bindgen::parser::MavType;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    error::ErrInstrument,
    mavlink::{
        MavMessage, Message,
        reflection::{FieldLike, FieldLookup, MAVLINK_PROFILE, MapConvertible, MessageMap},
    },
};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandSwitchWindow {
    commands: Vec<Command>,
    #[serde(skip)]
    state: VisibileState,
    #[serde(skip)]
    messages_to_send: Vec<MavMessage>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum VisibileState {
    #[default]
    Hidden,
    CommandCatalog,
    CommandSettings(u8),
    CommandSwitch,
}

impl VisibileState {
    fn switch_command(&mut self) {
        match self {
            VisibileState::Hidden
            | VisibileState::CommandCatalog
            | VisibileState::CommandSettings(_) => {
                *self = VisibileState::CommandSwitch;
            }
            VisibileState::CommandSwitch => {
                *self = VisibileState::Hidden;
            }
        }
    }

    fn switch_catalog(&mut self) {
        match self {
            VisibileState::Hidden
            | VisibileState::CommandSwitch
            | VisibileState::CommandSettings(_) => {
                *self = VisibileState::CommandCatalog;
            }
            VisibileState::CommandCatalog => {
                *self = VisibileState::Hidden;
            }
        }
    }
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
        if self.state == VisibileState::CommandSwitch {
            // Show the command switch window
            show_command_switch_window(ui, self);
        } else {
            show_command_settings_window(ui, self);
        }
    }

    pub fn consume_messages_to_send(&mut self) -> Vec<MavMessage> {
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
    let mut visible = matches!(state, VisibileState::CommandSwitch);
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
                let shortcut_text = cmd.shortcut_comb()[1].format(&ModifierNames::SYMBOLS, is_mac);
                let text = RichText::new(format!("[{}] {}", shortcut_text, &cmd.name)).size(17.0);
                let cmd_btn = ui.add_sized(Vec2::new(300.0, 10.0), egui::Button::new(text));

                // catch called shortcuts
                let shortcut_pressed = ui
                    .ctx()
                    .input_mut(|i| i.consume_shortcut(&cmd.shortcut_comb()[1]));
                let msg = (shortcut_pressed || cmd_btn.clicked())
                    .then(|| cmd.message.clone())
                    .flatten();
                if let Some(map) = msg {
                    messages_to_send.push(MavMessage::from_map(map).log_unwrap());
                }
            }
        });
    if !visible {
        *state = VisibileState::Hidden;
    }
}

fn show_command_settings_window(ui: &mut Ui, window: &mut CommandSwitchWindow) {
    let CommandSwitchWindow {
        state, commands, ..
    } = window;
    let mut visible = matches!(
        state,
        VisibileState::CommandSettings(_) | VisibileState::CommandCatalog
    );
    egui::Window::new("Command Switch")
        .id(ui.id().with("command_switch_settings_window"))
        .max_width(300.0)
        .resizable(false)
        .collapsible(false)
        .open(&mut visible)
        .show(ui.ctx(), |ui| {
            if commands.iter().all(|cmd| !cmd.ui_visible) {
                show_command_overview(ui, commands);
            } else {
                let cmd = commands.iter_mut().find(|cmd| cmd.ui_visible).log_unwrap();
                show_single_command_settings(ui, cmd);
            }
        });
    if !visible {
        *state = VisibileState::Hidden;
    }
}

fn show_command_overview(ui: &mut Ui, commands: &mut Vec<Command>) {
    for cmd in commands.iter_mut() {
        #[cfg(target_os = "macos")]
        let is_mac = true;
        #[cfg(not(target_os = "macos"))]
        let is_mac = false;
        let shortcut_text = cmd
            .shortcut_comb()
            .into_iter()
            .map(|s| s.format(&ModifierNames::SYMBOLS, is_mac))
            .join(" ");
        let text = RichText::new(format!("[{}] {}", shortcut_text, &cmd.name)).size(17.0);
        let cmd_btn = ui.add_sized(Vec2::new(300.0, 10.0), egui::Button::new(text));
        if cmd_btn.clicked() {
            cmd.ui_visible = true;
        }
    }
    if commands.len() < 10 {
        let plus_btn = ui.add_sized(
            Vec2::new(300.0, 10.0),
            egui::Button::new(RichText::new("+").size(17.0)),
        );
        if plus_btn.clicked() {
            commands.push(Command::new(commands.len()));
        }
    }
}

// FIXME: this function was duplicated from `src/ui/panes/command.rs`
fn show_single_command_settings(ui: &mut Ui, command: &mut Command) {
    let Command {
        name,
        message,
        ui_visible,
        show_only_tc,
        ..
    } = command;
    ui.label(RichText::new("Command Settings:").size(15.0));
    ui.separator();

    // Command text
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(name);
    });

    // add a checkbox for filtering sendable messages
    ui.checkbox(show_only_tc, "Show only TC messages");

    // Create a combo box for selecting the message kind
    let mut message_id = message.as_ref().map(|m| m.message_id());
    let selected_text = message_id
        .and_then(|id| MAVLINK_PROFILE.get_msg(id))
        .map(|m| m.name.clone())
        .unwrap_or("Select a Message".to_string());
    egui::ComboBox::from_id_salt(ui.id().with("message_selector"))
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            let mut msgs = MAVLINK_PROFILE.get_sorted_msgs();
            if *show_only_tc {
                msgs.retain(|m| m.name.ends_with("_TC"));
            }
            for msg in msgs {
                ui.selectable_value(&mut message_id, Some(msg.id), &msg.name);
            }
        });

    // If the message id is changed, update the message
    if message
        .as_ref()
        .is_none_or(|m| Some(m.message_id()) != message_id)
    {
        if let Some(id) = message_id {
            *message = Some(
                MavMessage::default_message_from_id(id)
                    .log_unwrap()
                    .as_map(),
            );
        } else {
            *message = None;
        }
    }

    // For each field in the message, show a text box with the field name and value,
    // and update the MessageMap based on the content of these text fields.
    if let Some(message_map) = message.as_mut() {
        let mut settable_fields = (0..message_map.field_map().len())
            .map(|f| {
                f.to_mav_field(message_map.message_id(), &MAVLINK_PROFILE)
                    .log_unwrap()
            })
            .collect::<Vec<_>>();

        // filter out the fields that are not settable
        settable_fields.retain(|f| {
            // skip the timestamp field
            f.field().name.to_lowercase() != "timestamp"
        });

        if !settable_fields.is_empty() {
            ui.group(|ui| {
                for field in settable_fields {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", &field.field().name.to_uppercase()));

                        // show the combo box for enum types
                        if let Some(enum_type) = &field.field().enumtype {
                            let enum_info = MAVLINK_PROFILE.get_enum(enum_type).log_unwrap();
                            // TODO handle enum advanced options
                            macro_rules! variant_selector_for {
                                ($kind:ty) => {{
                                    let variant_ix: &mut $kind =
                                        message_map.get_mut_field(field).log_unwrap();
                                    let selected_text =
                                        enum_info.entries[*variant_ix as usize].name.clone();
                                    egui::ComboBox::from_id_salt(ui.id().with("field_selector"))
                                        .selected_text(selected_text)
                                        .show_ui(ui, |ui| {
                                            for (index, variant) in
                                                enum_info.entries.iter().enumerate()
                                            {
                                                ui.selectable_value(
                                                    variant_ix,
                                                    index as $kind,
                                                    &variant.name,
                                                );
                                            }
                                        });
                                }};
                            }
                            match field.field().mavtype {
                                MavType::UInt8 => variant_selector_for!(u8),
                                MavType::UInt16 => variant_selector_for!(u16),
                                MavType::UInt32 => variant_selector_for!(u32),
                                MavType::UInt64 => variant_selector_for!(u64),
                                _ => {
                                    // TODO handle other enum types
                                    warn!(
                                        "Enum type {} is not supported for field {}",
                                        enum_type,
                                        field.field().name
                                    );
                                }
                            }
                        } else {
                            // show the drag value for numeric types and text box for char types
                            macro_rules! drag_value_with_range {
                                ($_type:ty, $min:expr, $max:expr) => {{
                                    let value: &mut $_type =
                                        message_map.get_mut_field(field).log_unwrap();
                                    ui.add(egui::DragValue::new(value).range($min..=$max));
                                }};
                            }

                            match field.field().mavtype {
                                MavType::UInt8MavlinkVersion | MavType::UInt8 => {
                                    drag_value_with_range!(u8, 0, u8::MAX)
                                }
                                MavType::UInt16 => drag_value_with_range!(u16, 0, u16::MAX),
                                MavType::UInt32 => drag_value_with_range!(u32, 0, u32::MAX),
                                MavType::UInt64 => drag_value_with_range!(u64, 0, u64::MAX),
                                MavType::Int8 => drag_value_with_range!(i8, i8::MIN, i8::MAX),
                                MavType::Int16 => drag_value_with_range!(i16, i16::MIN, i16::MAX),
                                MavType::Int32 => drag_value_with_range!(i32, i32::MIN, i32::MAX),
                                MavType::Int64 => drag_value_with_range!(i64, i64::MIN, i64::MAX),
                                MavType::Float => drag_value_with_range!(f32, f32::MIN, f32::MAX),
                                MavType::Double => drag_value_with_range!(f64, f64::MIN, f64::MAX),
                                MavType::Char => {
                                    let value: &mut char =
                                        message_map.get_mut_field(field).log_unwrap();
                                    let mut buffer = value.to_string();
                                    ui.add(
                                        egui::TextEdit::singleline(&mut buffer)
                                            .hint_text("char")
                                            .char_limit(1),
                                    );
                                    if let Some(c) = buffer.chars().next() {
                                        *value = c;
                                    } else {
                                        warn!("Invalid char input: {}", buffer);
                                        // TODO handle invalid char input (USER ERROR)
                                    }
                                }
                                MavType::Array(_, _) => warn!("Array types are not supported yet"),
                            }
                        }
                    });
                }
            });
        }
    }

    ui.separator();
    if ui.button("⬅ Back").clicked() {
        *ui_visible = false;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Command {
    id: usize,
    name: String,
    message: Option<MessageMap>,

    // UI SETTINGS
    #[serde(skip)]
    ui_visible: bool,
    #[serde(skip)]
    show_only_tc: bool,
}

impl Command {
    fn new(id: usize) -> Self {
        Self {
            id,
            name: String::from("New Command"),
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
