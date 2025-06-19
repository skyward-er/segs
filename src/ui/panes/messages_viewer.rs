use crate::mavlink::{TimedMessage, reflection::MAVLINK_PROFILE};
use crate::ui::panes::{PaneBehavior, PaneResponse};
use crate::ui::shortcuts::ShortcutHandler;
use egui::{Response, ScrollArea, Sense, UiBuilder, Window};
use serde::{Deserialize, Serialize};
use skyward_mavlink::mavlink::MessageData;
use skyward_mavlink::orion::ROCKET_FLIGHT_TM_DATA;
use std::collections::{HashMap, HashSet};

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
    fn ui(&mut self, ui: &mut egui::Ui, _shortcut_handler: &mut ShortcutHandler) -> PaneResponse {
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
                            .max_height(100.0)
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

        let res = ui
            .scope_builder(UiBuilder::new().sense(Sense::click_and_drag()), |ui| {
                egui::Grid::new("message_viewer").show(ui, |ui| {
                    if let Some(selected_msg) = &self.selected_message {
                        if let Some(fields) = MAVLINK_PROFILE.get_fields(*selected_msg) {
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
            .response;

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
                            if let Ok(value) = field.extract_as_f64(&msg.message) {
                                self.field_map.insert(field.id(), Some(value.to_string()));
                            }
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
