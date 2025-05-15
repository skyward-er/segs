use std::collections::HashMap;

use egui::{Button, RichText, Sense, Ui, Vec2};
use egui_extras::{Size, Strip, StripBuilder};
use mavlink_bindgen::parser::MavType;
use serde::{Deserialize, Serialize, de::IntoDeserializer};
use tracing::warn;

use crate::{
    error::ErrInstrument,
    mavlink::{
        COMMAND_TC_DATA, MavMessage, Message, MessageData, TimedMessage,
        reflection::{FieldLike, FieldLookup, MAVLINK_PROFILE, MapConvertible, MessageMap},
    },
    ui::{app::PaneResponse, shortcuts::ShortcutHandler},
};

use super::PaneBehavior;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandPane {
    message: Option<MessageMap>,
    text: String,
    text_size: f32,

    #[serde(skip)]
    settings_visible: bool,
}

impl Default for CommandPane {
    fn default() -> Self {
        Self {
            message: None,
            text: String::from("Customize"),
            text_size: 16.0,
            settings_visible: false,
        }
    }
}

impl PartialEq for CommandPane {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
            && self.text == other.text
            && self.text_size == other.text_size
    }
}

impl PaneBehavior for CommandPane {
    #[profiling::function]
    fn ui(&mut self, ui: &mut Ui, _shortcut_handler: &mut ShortcutHandler) -> PaneResponse {
        let mut response = PaneResponse::default();

        let parent = ui
            .scope(|ui| {
                let btn_text = RichText::new(&self.text).size(self.text_size).strong();
                let btn = Button::new(btn_text).sense(egui::Sense::click());

                // Clever way to add padding to the button
                ui.allocate_rect(ui.max_rect(), Sense::click());
                let btn_rect = ui.max_rect().shrink(2.0);
                let btn_res = ui.put(btn_rect, btn);

                // open the menu on right click on button
                btn_res.context_menu(|ui| command_menu(ui, self));
            })
            .response;

        if parent.interact(egui::Sense::click_and_drag()).dragged() {
            response.set_drag_started();
        };

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

    fn update(&mut self, _messages: &[&TimedMessage]) {}

    fn get_message_subscriptions(&self) -> Box<dyn Iterator<Item = u32>> {
        Box::new(None.into_iter())
    }
}

fn command_menu(ui: &mut Ui, pane: &mut CommandPane) {
    if ui.button("Settings…").clicked() {
        pane.settings_visible = true;
        ui.close_menu();
    }
}

fn command_settings(ui: &mut Ui, pane: &mut CommandPane) {
    ui.set_max_width(200.0);
    ui.horizontal(|ui| {
        ui.label("Text:");
        ui.text_edit_singleline(&mut pane.text);
    });
    ui.horizontal(|ui| {
        ui.label("Text Size:");
        ui.add(egui::Slider::new(&mut pane.text_size, 11.0..=25.0));
    });

    ui.separator();

    // Create a combo box for selecting the message kind
    let mut message_id = pane.message.as_ref().map(|m| m.message_id());
    let selected_text = message_id
        .and_then(|id| MAVLINK_PROFILE.get_msg(id))
        .map(|m| m.name.clone())
        .unwrap_or("Select a Message".to_string());
    egui::ComboBox::from_id_salt(ui.id().with("message_selector"))
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            for msg in MAVLINK_PROFILE.get_sorted_msgs() {
                ui.selectable_value(&mut message_id, Some(msg.id), &msg.name);
            }
        });

    // If the message id is changed, update the message
    if pane
        .message
        .as_ref()
        .is_none_or(|m| Some(m.message_id()) != message_id)
    {
        if let Some(id) = message_id {
            pane.message = Some(
                MavMessage::default_message_from_id(id)
                    .log_unwrap()
                    .as_map(),
            );
        } else {
            pane.message = None;
        }
    }

    // For each field in the message, show a text box with the field name and value,
    // and update the MessageMap based on the content of these text fields.
    if let Some(message_map) = pane.message.as_mut() {
        ui.group(|ui| {
            for i in 0..message_map.field_map().len() {
                let field = i
                    .to_mav_field(message_map.message_id(), &MAVLINK_PROFILE)
                    .log_unwrap();

                ui.horizontal(|ui| {
                    ui.label(format!("{}:", &field.field().name.to_uppercase()));
                    macro_rules! drag_value_with_range {
                        ($_type:ty, $min:expr, $max:expr) => {{
                            let value: &mut $_type = message_map.get_mut_field(field).log_unwrap();
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
                            let value: &mut char = message_map.get_mut_field(field).log_unwrap();
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
                        MavType::Array(_, _) => warn!("Array types are not supported yet"), // TODO handle array types
                    }
                });
            }
        });
    }
}
