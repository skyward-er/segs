use egui::{
    Color32, DragValue, Frame, Key, KeyboardShortcut, Label, Margin, Modifiers, Response, RichText,
    Sense, Sides, Stroke, Ui, UiBuilder, Widget,
    ahash::{HashSet, HashSetExt},
    response::Flags,
};
use mavlink_bindgen::parser::MavType;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    error::ErrInstrument,
    mavlink::{
        MavHeader, MavMessage, Message,
        reflection::{
            FieldLike, FieldLookup, IndexedField, MAVLINK_PROFILE, MapConvertible, MessageMap,
        },
    },
    ui::{shortcuts::ShortcutHandlerExt, widgets::ShortcutCard},
};

use super::BaseCommand;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigurableCommand {
    pub base: BaseCommand,
    pub(super) selected_fields: HashSet<usize>,
    #[serde(skip)]
    pub parameters_window_visible: bool,
    #[serde(skip)]
    focused_key: Option<Key>,
}

impl ConfigurableCommand {
    pub fn new(id: usize) -> Self {
        Self {
            base: BaseCommand::new(id),
            selected_fields: HashSet::new(),
            parameters_window_visible: false,
            focused_key: None,
        }
    }

    pub fn show_operative_parameters(
        &mut self,
        state: &mut super::state::StateManager,
        messages_to_send: &mut Vec<(MavHeader, MavMessage)>,
        ui: &mut Ui,
    ) {
        let ConfigurableCommand {
            base:
                BaseCommand {
                    name,
                    message,
                    system_id,
                    ..
                },
            selected_fields,
            parameters_window_visible,
            focused_key,
        } = self;
        if let Some(message) = message {
            ui.horizontal(|ui| {
                ui.label(RichText::new(name.as_str()).size(15.0).strong());
                ui.label(RichText::new("- Parameters").size(15.0));
            });
            ui.separator();

            // Short discaimer if the command has no fields selected
            if selected_fields.is_empty() {
                ui.label(RichText::new("No fields selected for configuration").italics());
            }

            let keys = vec![
                Key::Num1,
                Key::Num2,
                Key::Num3,
                Key::Num4,
                Key::Num5,
                Key::Num6,
                Key::Num7,
                Key::Num8,
                Key::Num9,
            ];
            for (field_id, key) in selected_fields.iter().zip(keys) {
                let field = field_id
                    .to_mav_field(message.message_id(), &MAVLINK_PROFILE)
                    .log_unwrap();
                let focus_requested = focused_key.is_some_and(|k| k == key);
                let response = ui
                    .horizontal(|ui| {
                        let shortcut_response = if focused_key.is_some() {
                            shortcut_btn(ui, "", Some(Key::Enter))
                        } else {
                            shortcut_btn(ui, "", Some(key))
                        };
                        if shortcut_response.clicked() {
                            if focus_requested {
                                focused_key.take();
                            } else {
                                focused_key.replace(key);
                            }
                        }
                        ui.label(format!("{}: ", field.field().name.to_uppercase()));
                        field_editor(field, message, ui)
                    })
                    .inner;
                // if the field is focused, set the focus state
                if response.gained_focus() && focused_key.is_none() {
                    focused_key.replace(key);
                }
                if focused_key.is_some_and(|k| k == key) && !response.has_focus() {
                    response.request_focus();
                }
                // if the field looses focus or the key is pressed, update the focus state
                if focused_key.is_none() && !response.lost_focus() {
                    response.surrender_focus();
                }
                if response.lost_focus() && focused_key.is_some() {
                    focused_key.take();
                }
            }
        } else {
            // Show an error if the message is not set
            ui.label(
                RichText::new("No message selected for configuration").color(egui::Color32::RED),
            );
        }

        // Lower buttons
        ui.separator();
        let mut back_invoked = false;
        let mut send_invoked = false;
        Sides::new().show(
            ui,
            |ui| {
                // Back button to close the parameters window
                let key = focused_key.xor(Some(Key::Backspace));
                if shortcut_btn(ui, "BACK", key).clicked() {
                    back_invoked = true;
                }
            },
            |ui| {
                let key = focused_key.xor(Some(Key::Plus));
                if shortcut_btn(ui, "SEND", key).clicked() {
                    send_invoked = true;
                }
            },
        );
        if back_invoked {
            *parameters_window_visible = false;
        }

        if send_invoked {
            if let Some(map) = message {
                // append the message to the list of messages to send
                let header = MavHeader {
                    system_id: *system_id,
                    ..Default::default()
                };
                messages_to_send.push((header, MavMessage::from_map(map.clone()).log_unwrap()));
                // close the command switch window
                state.hide();
            }
            *parameters_window_visible = false;
        }
    }
}

// TODO: convert this into a widget (and remove code duplication)
fn shortcut_btn(ui: &mut Ui, text: &str, key: Option<Key>) -> Response {
    let shortcut = key.map(|key| KeyboardShortcut::new(Modifiers::NONE, key));
    let shortcut_detected = ui
        .ctx()
        .shortcuts()
        .lock()
        .capture_actions(
            ui.id().with("shortcut_lease"),
            Box::new(super::CommandSwitchLease),
            |_| {
                if let Some(key) = key {
                    vec![(Modifiers::NONE, key, true)]
                } else {
                    vec![]
                }
            },
        )
        .unwrap_or_default();
    let mut res = ui
        .scope_builder(UiBuilder::new().id_salt(key).sense(Sense::click()), |ui| {
            let mut visuals = *ui.style().interact(&ui.response());

            // override the visuals if the button is pressed
            if shortcut_detected {
                visuals = ui.visuals().widgets.active;
            }
            let vis = ui.visuals();
            let uvis = ui.style().interact(&ui.response());
            let shortcut_card = shortcut.map(|shortcut| {
                ShortcutCard::new(shortcut)
                    .text_color(vis.strong_text_color())
                    .fill_color(vis.gray_out(uvis.bg_fill))
                    .margin(Margin::symmetric(5, 0))
                    .text_size(12.)
            });

            Frame::canvas(ui.style())
                .inner_margin(Margin::symmetric(4, 2))
                .outer_margin(0)
                .corner_radius(ui.visuals().noninteractive().corner_radius)
                .fill(visuals.bg_fill)
                .stroke(Stroke::new(1., Color32::TRANSPARENT))
                .show(ui, |ui| {
                    ui.set_height(ui.available_height());
                    ui.horizontal_centered(|ui| {
                        ui.set_height(15.);
                        if !text.is_empty() {
                            ui.add_space(1.);
                            Label::new(RichText::new(text).size(14.).color(visuals.text_color()))
                                .selectable(false)
                                .ui(ui);
                        }
                        if let Some(shortcut_card) = shortcut_card {
                            shortcut_card.ui(ui);
                        }
                    });
                });
        })
        .response;

    if shortcut_detected {
        res.flags.insert(Flags::FAKE_PRIMARY_CLICKED);
    }
    res
}

// TODO: convert this into a widget (and remove code duplication)
fn field_editor(field: IndexedField, message_map: &mut MessageMap, ui: &mut Ui) -> Response {
    // show the combo box for enum types
    if let Some(enum_type) = &field.field().enumtype {
        let enum_info = MAVLINK_PROFILE.get_enum(enum_type).log_unwrap();
        // TODO handle enum advanced options
        macro_rules! variant_selector_for {
            ($kind:ty) => {{
                let variant_ix: &mut $kind = message_map.get_mut_field(field).log_unwrap();
                let selected_text = enum_info.entries[*variant_ix as usize].name.clone();
                egui::ComboBox::from_id_salt(ui.id().with("field_selector"))
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for (index, variant) in enum_info.entries.iter().enumerate() {
                            ui.selectable_value(variant_ix, index as $kind, &variant.name);
                        }
                    })
                    .response
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
                ui.response()
            }
        }
    } else {
        // show the drag value for numeric types and text box for char types
        macro_rules! drag_value_with_range {
            ($_type:ty, $min:expr, $max:expr) => {{
                let value: &mut $_type = message_map.get_mut_field(field).log_unwrap();
                ui.add(
                    egui::DragValue::new(value)
                        .range($min..=$max)
                        .clamp_existing_to_range(true),
                )
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
            MavType::Int16 => {
                drag_value_with_range!(i16, i16::MIN, i16::MAX)
            }
            MavType::Int32 => {
                drag_value_with_range!(i32, i32::MIN, i32::MAX)
            }
            MavType::Int64 => {
                drag_value_with_range!(i64, i64::MIN, i64::MAX)
            }
            MavType::Float => {
                drag_value_with_range!(f32, f32::MIN, f32::MAX)
            }
            MavType::Double => {
                drag_value_with_range!(f64, f64::MIN, f64::MAX)
            }
            MavType::Char => {
                let value: &mut char = message_map.get_mut_field(field).log_unwrap();
                let mut buffer = value.to_string();
                let res = ui.add(
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
                res
            }
            MavType::Array(_, _) => {
                warn!("Array types are not supported yet");
                ui.response()
            }
        }
    }
}

pub fn show_command_settings(ui: &mut Ui, command: &mut ConfigurableCommand) {
    let ConfigurableCommand {
        base:
            BaseCommand {
                name,
                system_id,
                message,
                settings_window_visible,
                show_only_tc,
                ..
            },
        selected_fields,
        ..
    } = command;
    // Command text
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(name);
    });

    // add a label for the system ID
    ui.horizontal(|ui| {
        let label = ui.label("System ID:");
        // add a drag value for the system ID
        ui.add(DragValue::new(system_id).range(1..=255))
            .labelled_by(label.id);
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

    // If the message id is changed, update the message and selected fields
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
        selected_fields.clear();
    }

    // For each field in the message, show a checkbox with the field name
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

        let num_checked = selected_fields.len();
        if !settable_fields.is_empty() {
            ui.group(|ui| {
                for field in settable_fields {
                    ui.horizontal(|ui| {
                        // First, the checkbox for selecting the field to configure in operation mode
                        let mut field_present = selected_fields.contains(&field.id());
                        let text = if field_present {
                            field.field().name.to_uppercase()
                        } else {
                            format!("{}: ", field.field().name.to_uppercase())
                        };
                        ui.checkbox(&mut field_present, text);
                        if field_present && num_checked >= 9 {
                            warn!(
                                "Maximum number of fields selected for configuration reached (9)."
                            );
                        } else if field_present {
                            // Add the field to the selected fieldss
                            selected_fields.insert(field.id());
                        } else {
                            // Remove the field from the selected fields
                            selected_fields.remove(&field.id());

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
                                        egui::ComboBox::from_id_salt(
                                            ui.id().with("field_selector"),
                                        )
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
                                    MavType::Int16 => {
                                        drag_value_with_range!(i16, i16::MIN, i16::MAX)
                                    }
                                    MavType::Int32 => {
                                        drag_value_with_range!(i32, i32::MIN, i32::MAX)
                                    }
                                    MavType::Int64 => {
                                        drag_value_with_range!(i64, i64::MIN, i64::MAX)
                                    }
                                    MavType::Float => {
                                        drag_value_with_range!(f32, f32::MIN, f32::MAX)
                                    }
                                    MavType::Double => {
                                        drag_value_with_range!(f64, f64::MIN, f64::MAX)
                                    }
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
                                    MavType::Array(_, _) => {
                                        warn!("Array types are not supported yet")
                                    }
                                }
                            }
                        }
                    });
                }
            });
        }

        ui.label(
            RichText::new("Check the fields you'd like to configure in operation mode").italics(),
        );

        if num_checked >= 9 {
            ui.label(
                RichText::new("Maximum number of fields selected for configuration reached (9). Deselect some fields to continue.")
                    .color(egui::Color32::RED),
            );
        }
    }

    ui.separator();
    if ui.button("⬅ Back").clicked() {
        *settings_window_visible = false;
    }
}
