use std::collections::{HashMap, HashSet};

use egui::{Response, ScrollArea, Sense, UiBuilder, Window};
use mavlink_bindgen::parser::MavType;
use serde::{Deserialize, Serialize};

use crate::{
    error::ErrInstrument,
    mavlink::{
        MavMessage, MessageData, ROCKET_FLIGHT_TM_DATA, TimedMessage,
        reflection::{IndexedField, MAVLINK_PROFILE},
    },
    ui::panes::{PaneBehavior, PaneResponse},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MessagesViewerPane {
    // == PANE RELATED ==
    selected_message: Option<u32>,
    selected_fields: HashSet<usize>,

    // == TEMP VALUES ==
    #[serde(skip)]
    field_map: HashMap<usize, Option<String>>,

    // == UI RELATED ==
    #[serde(skip)]
    settings_visible: bool,
}

impl PaneBehavior for MessagesViewerPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let mut pane_response = PaneResponse::default(); // Crea una risposta predefinita del pannello

        if self.settings_visible {
            let msg_name = self
                .selected_message
                .and_then(|id| MAVLINK_PROFILE.get_msg(id))
                .map(|msg| msg.name.clone())
                .unwrap_or_else(|| "Select a message".to_string());
            // Finestra di configurazione
            Window::new("Messages Viewer Settings")
                .id(ui.id().with("messages_viewer_settings"))
                .collapsible(false)
                .resizable(false)
                .open(&mut self.settings_visible)
                .show(ui.ctx(), |ui| {
                    egui::ComboBox::new("message_kind", "Message Kind")
                        .selected_text(msg_name)
                        .show_ui(ui, |ui| {
                            for msg in MAVLINK_PROFILE.get_sorted_msgs() {
                                let mut current =
                                    self.selected_message.unwrap_or(ROCKET_FLIGHT_TM_DATA::ID);
                                ui.selectable_value(&mut current, msg.id, &msg.name);
                                self.selected_message = Some(current);
                            }
                        });

                    if let Some(ref selected_msg) = self.selected_message {
                        ui.label("Select Fields:");
                        ScrollArea::both()
                            .auto_shrink([false, true])
                            .max_width(300.0)
                            .max_height(300.0)
                            .show(ui, |ui| {
                                let mut select_all = false;
                                let mut deselect_all = false;

                                if let Some(fields) = MAVLINK_PROFILE.get_fields(*selected_msg) {
                                    if fields.len() > 1 {
                                        ui.horizontal(|ui| {
                                            ui.checkbox(&mut select_all, "Select All");
                                            ui.checkbox(&mut deselect_all, "Deselect All");
                                        });
                                    }
                                }

                                if select_all {
                                    if let Some(fields) = MAVLINK_PROFILE.get_fields(*selected_msg)
                                    {
                                        for field in &fields {
                                            self.selected_fields.insert(field.id());
                                        }
                                    }
                                }

                                if deselect_all {
                                    self.selected_fields.clear();
                                }

                                if let Some(fields) = MAVLINK_PROFILE.get_fields(*selected_msg) {
                                    for field in fields {
                                        let mut selected =
                                            self.selected_fields.contains(&field.id());
                                        let response: Response =
                                            ui.checkbox(&mut selected, field.field().name.clone());
                                        if response.clicked() {
                                            if selected {
                                                self.selected_fields.insert(field.id());
                                            } else {
                                                self.selected_fields.remove(&field.id());
                                            }
                                        }
                                    }
                                }
                            });
                    }
                });
        }

        let max_rect = ui.max_rect().shrink(8.);
        let res = ui.scope_builder(
            UiBuilder::new()
                .max_rect(max_rect)
                .sense(Sense::click_and_drag()),
            |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.scope_builder(UiBuilder::new().sense(Sense::click_and_drag()), |ui| {
                            egui::Grid::new("message_viewer").show(ui, |ui| {
                                if let Some(selected_msg) = &self.selected_message {
                                    if let Some(fields) = MAVLINK_PROFILE.get_fields(*selected_msg)
                                    {
                                        for field in fields {
                                            // Usa field come &str per il controllo
                                            if self.selected_fields.contains(&field.id()) {
                                                let value = self
                                                    .field_map
                                                    .get(&field.id())
                                                    .and_then(|v| v.as_deref())
                                                    .unwrap_or("N/A");

                                                ui.label(field.field().name.clone());
                                                ui.label(value);
                                                ui.end_row();
                                            }
                                        }
                                    }
                                } else {
                                    ui.label("No message selected");
                                }
                            });

                            // FIll the remaining space
                            ui.allocate_space(ui.available_size());
                        })
                        .response
                    })
                    .inner
            },
        );

        let res = res.inner.union(res.response);

        // Show the menu when the user right-clicks the pane
        res.context_menu(|ui| {
            if ui.button("Open settings…").clicked() {
                self.settings_visible = true;
                ui.close_menu();
            }
        });

        // Check if the user started dragging the pane
        if res.drag_started() {
            pane_response.set_drag_started();
        }

        pane_response
    }

    fn update(&mut self, messages: &[&TimedMessage]) {
        for msg in messages {
            if let Some(selected_msg) = &self.selected_message {
                if let Some(fields) = MAVLINK_PROFILE.get_fields(*selected_msg) {
                    for field in fields {
                        if self.selected_fields.contains(&field.id()) {
                            let value = field.format(&msg.message);
                            self.field_map.insert(field.id(), Some(value));
                        }
                    }
                }
            }
        }
    }

    fn get_message_subscriptions(&self) -> Box<dyn Iterator<Item = u32>> {
        Box::new(self.selected_message.into_iter())
    }

    fn should_send_message_history(&self) -> bool {
        false
    }
}

trait MessageViewerFormatter {
    fn format(&self, msg: &MavMessage) -> String;
}

impl MessageViewerFormatter for IndexedField {
    fn format(&self, msg: &MavMessage) -> String {
        match &self.field().mavtype {
            MavType::UInt8MavlinkVersion | MavType::UInt8 => {
                self.extract_as_u8(msg).log_unwrap().to_string()
            }
            MavType::UInt16 => self.extract_as_u16(msg).log_unwrap().to_string(),
            MavType::UInt32 => self.extract_as_u32(msg).log_unwrap().to_string(),
            MavType::UInt64 => self.extract_as_u64(msg).log_unwrap().to_string(),
            MavType::Int8 => self.extract_as_i8(msg).log_unwrap().to_string(),
            MavType::Int16 => self.extract_as_i16(msg).log_unwrap().to_string(),
            MavType::Int32 => self.extract_as_i32(msg).log_unwrap().to_string(),
            MavType::Int64 => self.extract_as_i64(msg).log_unwrap().to_string(),
            MavType::Char => self.extract_as_char(msg).log_unwrap().to_string(),
            MavType::Float => format!("{:.5}", self.extract_as_f32(msg).log_unwrap()),
            MavType::Double => format!("{:.5}", self.extract_as_f64(msg).log_unwrap()),
            MavType::Array(_, _) => self.extract_as_string(msg).log_unwrap(),
        }
    }
}
