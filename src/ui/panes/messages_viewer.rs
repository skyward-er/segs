use crate::MAVLINK_PROFILE;
use crate::error::ErrInstrument;
use crate::mavlink::TimedMessage; // Importa moduli specifici dal crate mavlink
use crate::ui::panes::{PaneBehavior, PaneResponse}; // Importa i comportamenti e le risposte del pannello
use crate::ui::shortcuts::ShortcutHandler;
use egui::{Response, ScrollArea, Sense, UiBuilder, Window}; // Importa i moduli necessari da egui
use serde::{Deserialize, Serialize}; // Importa i moduli per la serializzazione e deserializzazione
use std::collections::{HashMap, HashSet}; // Importa HashSet e HashMap dalla libreria standard

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MessagesViewerPane {
    #[serde(skip)]
    contains_pointer: bool, // Indica se il puntatore è contenuto nel pannello
    #[serde(skip)]
    items: Vec<String>, // Elenco degli elementi visualizzati
    settings: MsgSources, // Impostazioni del messaggio
    #[serde(skip)]
    settings_visible: bool, // Indica se le impostazioni sono visibili
    #[serde(skip)]
    available_messages: Vec<u32>, // Elenco dei messaggi disponibili
    #[serde(skip)]
    seen_message_types: HashSet<u32>, // Tipi di messaggi visti
    #[serde(skip)]
    field_map: HashMap<usize, Option<String>>, // Mappa dei campi
    #[serde(skip)]
    selected_message: Option<u32>, // Messaggio selezionato
    #[serde(skip)]
    selected_fields: HashSet<usize>, // Campi selezionati
    #[serde(skip)]
    message_log: Vec<String>, // Registro dei messaggi
    sampling_frequency: f32, // Frequenza di campionamento
}

impl Default for MessagesViewerPane {
    fn default() -> Self {
        Self {
            contains_pointer: false,
            settings_visible: false,
            items: vec![],
            settings: MsgSources::default(),
            available_messages: MAVLINK_PROFILE
                .get_sorted_msgs()
                .iter()
                .map(|&s| s.id)
                .collect(),
            seen_message_types: HashSet::new(),
            sampling_frequency: 10.0,
            selected_message: None,
            selected_fields: HashSet::new(),
            message_log: Vec::new(),
            field_map: HashMap::new(),
        }
    }
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
                            for message_type in &self.available_messages {
                                let message_name = MAVLINK_PROFILE
                                    .get_msg(*message_type)
                                    .map(|msg| msg.name.clone())
                                    .log_unwrap();
                                if ui
                                    .selectable_label(
                                        self.selected_message.is_some_and(|v| v == *message_type),
                                        message_name,
                                    )
                                    .clicked()
                                {
                                    self.selected_message = Some(*message_type);
                                    self.selected_fields.clear();
                                }
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

                    ui.add_space(7.0);
                    ui.label("Sampling Frequency (Hz):");
                    ui.add(egui::Slider::new(&mut self.sampling_frequency, 1.0..=100.0).text("Hz"));
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

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct MsgSources {
    msg_id: u32,
    fields_with_checkbox: Vec<(String, bool)>,
}

impl PartialEq for MsgSources {
    fn eq(&self, other: &Self) -> bool {
        self.msg_id == other.msg_id && self.fields_with_checkbox == other.fields_with_checkbox
    }
}
