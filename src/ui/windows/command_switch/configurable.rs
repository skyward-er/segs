use egui::{
    DragValue, RichText, Ui,
    ahash::{HashSet, HashSetExt},
};
use mavlink_bindgen::parser::MavType;
use serde::{Deserialize, Serialize};
use skyward_mavlink::mavlink::Message;
use tracing::warn;

use crate::{
    error::ErrInstrument,
    mavlink::{
        MavMessage,
        reflection::{FieldLike, FieldLookup, MAVLINK_PROFILE, MapConvertible},
    },
};

use super::BaseCommand;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigurableCommand {
    pub base: BaseCommand,
    pub(super) selected_fields: HashSet<usize>,
}

impl ConfigurableCommand {
    pub fn new(id: usize) -> Self {
        Self {
            base: BaseCommand::new(id),
            selected_fields: HashSet::new(),
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
                ui_visible,
                show_only_tc,
                ..
            },
        selected_fields,
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
                        if field_present {
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
    }

    ui.separator();
    if ui.button("⬅ Back").clicked() {
        *ui_visible = false;
    }
}
