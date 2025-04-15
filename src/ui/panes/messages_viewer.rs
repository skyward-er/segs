use egui::{ScrollArea, Window, Response}; // Importa i moduli necessari da egui
use serde::{Serialize, Deserialize}; // Importa i moduli per la serializzazione e deserializzazione
use std::collections::{HashSet, HashMap}; // Importa HashSet e HashMap dalla libreria standard
use crate::mavlink::{TimedMessage, extract_from_message}; // Importa moduli specifici dal crate mavlink
use crate::mavlink::reflection::ReflectionContext; // Importa ReflectionContext dal crate mavlink
use crate::ui::panes::{PaneBehavior, PaneResponse}; // Importa i comportamenti e le risposte del pannello
use crate::mavlink::Message;
use crate::MAVLINK_PROFILE;


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
    available_messages: Vec<String>, // Elenco dei messaggi disponibili
    #[serde(skip)]
    seen_message_types: HashSet<String>, // Tipi di messaggi visti
    #[serde(skip)]
    field_map: HashMap<String, Option<String>>, // Mappa dei campi
    #[serde(skip)]
    selected_message: Option<u32>, // Messaggio selezionato
    #[serde(skip)]
    selected_fields: HashSet<String>, // Campi selezionati
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
            available_messages: ReflectionContext::new()
                .sorted_messages()
                .iter()
                .map(|&s| s.to_string())
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
    fn ui(&mut self, ui: &mut egui::Ui, _tile_id: egui_tiles::TileId) -> PaneResponse {
        let response = PaneResponse::default(); // Crea una risposta predefinita del pannello
        ui.heading("Messages Viewer");
        if ui.button("Open Message Filter").clicked() {
            self.settings_visible = true; // Mostra le impostazioni se il pulsante è cliccato
        }
        
        if self.settings_visible {
            // Finestra di configurazione
            Window::new("Messages Viewer Settings")
                .collapsible(false)
                .resizable(false)
                .open(&mut self.settings_visible)
                .show(ui.ctx(), |ui| {
                    egui::ComboBox::new("message_kind", "Message Kind")
                        .selected_text(self.selected_message.clone().unwrap_or("Select a message".to_string()))
                        .show_ui(ui, |ui| {
                            for message_type in &self.available_messages {
                                if ui.selectable_label(self.selected_message.as_deref() == Some(message_type), message_type).clicked() {
                                    self.selected_message = Some(message_type.clone());
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

                            if let Ok(fields) = MAVLINK_PROFILE.get_fields_by_name(selected_msg) {
                                if fields.len() > 1 {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut select_all, "Select All");
                                        ui.checkbox(&mut deselect_all, "Deselect All");
                                    });
                                }
                            }

                            if select_all {
                                if let Ok(fields) = MAVLINK_PROFILE.get_fields_by_name(selected_msg) {
                                    for field in &fields {
                                        self.selected_fields.insert(field.to_string());
                                    }
                                }
                            }
                            
                            if deselect_all {
                                self.selected_fields.clear();
                            }

                            if let Ok(fields) = ReflectionContext::new().get_fields_by_name(selected_msg) {
                                for field in fields {
                                    let mut selected = self.selected_fields.contains(field);
                                    let response: Response = ui.checkbox(&mut selected, field);
                                    if response.clicked() {
                                        if selected {
                                            self.selected_fields.insert(field.to_string());
                                        } else {
                                            self.selected_fields.remove(field);
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
        egui::Grid::new("message_viewer").show(ui, |ui| {
            if let Some(selected_msg) = &self.selected_message {
                if let Ok(fields) = MAVLINK_PROFILE.get_fields_by_name(selected_msg) {
                    for field in fields {
                        // Usa field come &str per il controllo
                        if self.selected_fields.contains(field) {
                            let value = self.field_map
                                .get(field)
                                .and_then(|v| v.as_deref())
                                .unwrap_or("N/A");
                            
                            ui.label(field);
                            ui.label(value);
                            ui.end_row();
                        }
                    }
                }
            } else {
                ui.label("No message selected");
            }
        });



        response
    }
    
    fn contains_pointer(&self) -> bool {
        self.contains_pointer
    }
    

fn update(&mut self, messages: &[TimedMessage]) {
    for msg in messages {
        if let Some(selected_msg) = &self.selected_message {
            
                if let Ok(fields) = MAVLINK_PROFILE.get_fields(selected_msg) { 
                    for field in fields {
                        if self.selected_fields.contains(field) {
                            if let Ok(values) = extract_from_message(&msg.message, &[field]) {
                                if !values.is_empty() {
                                    let value: f64 = values[0];
                                    self.field_map.insert(field.to_string(), Some(value.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
    
}

    fn get_message_subscription(&self) -> Option<u32> {
        None
    }

    fn should_send_message_history(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MsgSources {
    msg_id: u32,
    fields_with_checkbox: Vec<(String, bool)>,
}

impl Default for MsgSources {
    fn default() -> Self {
        Self {
            msg_id: 0,
            fields_with_checkbox: Vec::new(),
        }
    }
}

impl PartialEq for MsgSources {
    fn eq(&self, other: &Self) -> bool {
        self.msg_id == other.msg_id && self.fields_with_checkbox == other.fields_with_checkbox
    }
}