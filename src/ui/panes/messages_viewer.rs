use egui::{vec2, Color32, Frame, Id, ScrollArea, Window, Response}; // Importa i moduli necessari da egui
use serde::{Serialize, Deserialize}; // Importa i moduli per la serializzazione e deserializzazione
use std::collections::HashSet; // Importa HashSet dalla libreria standard
use crate::mavlink::{TimedMessage, extract_from_message}; // Importa moduli specifici dal crate mavlink
use crate::mavlink::reflection::ReflectionContext; // Importa ReflectionContext dal crate mavlink
use crate::ui::panes::{PaneBehavior, PaneResponse}; // Importa i comportamenti e le risposte del pannello

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
    selected_message: Option<String>, // Messaggio selezionato
    #[serde(skip)]
    selected_fields: HashSet<String>, // Campi selezionati
    #[serde(skip)]
    message_log: Vec<String>, // Registro dei messaggi
    sampling_frequency: f32, // Frequenza di campionamento
}

impl Default for MessagesViewerPane {
    fn default() -> Self {
        Self {
            contains_pointer: false, // Imposta contains_pointer su false
            settings_visible: false, // Imposta settings_visible su false
            items: vec![], // Inizializza items come un vettore vuoto
            settings: MsgSources::default(), // Imposta settings con il valore predefinito di MsgSources
            /*
             * Reflection permette di aggiornare in tempo reale quali sono i messaggi
             * e i fields dei pacchetti che riceviamo, il tutto in maniera automatica.
             */
            available_messages: ReflectionContext::new() //Questo carica il profilo MAVLink con tutti i messaggi disponibili.
                .sorted_messages() //Questo restituisce un Vec<&str> con i nomi dei messaggi, come ["ATTITUDE", "GPS_RAW_INT", "SYS_STATUS"].
                .iter()
                .map(|&s| s.to_string())
                .collect(), // Inizializza available_messages con i messaggi ordinati
                //.iter().map(|&s| s.to_string()).collect()
                //Trasforma ogni &str in String, perché il Vec finale deve contenere Vec<String>.
            seen_message_types: HashSet::new(), //Non permette duplicati. Se provi ad aggiungere due volte lo stesso elemento, lo tiene solo una volta.
            sampling_frequency: 10.0, // Imposta la frequenza di campionamento su 10.0
            selected_message: None, //Memorizza il nome del messaggio selezionato dall’utente. DEVO IMPOSTARLO A NONE IL PRIMO?
            selected_fields: HashSet::new(), //Memorizza i campi del messaggio selezionato che l’utente vuole visualizzare.
            message_log: Vec::new(), // Inizializza message_log come un vettore vuoto
        }
    }
}

impl PaneBehavior for MessagesViewerPane {
    fn ui(&mut self, ui: &mut egui::Ui, _tile_id: egui_tiles::TileId) -> PaneResponse {
        let response = PaneResponse::default(); // Crea una risposta predefinita del pannello
        ui.heading("Messages Viewer"); // Aggiunge un'intestazione al pannello
        if ui.button("Open Message Filter").clicked() {
            self.settings_visible = true; // Mostra le impostazioni se il pulsante è cliccato
        }
        
        if self.settings_visible {
            // Controlla se le impostazioni sono visibili
            Window::new("Messages Viewer Settings")
                // Crea una nuova finestra con il titolo "Messages Viewer Settings"
                .collapsible(false)
                // Impedisce alla finestra di essere collassabile
                .resizable(false)
                // Impedisce alla finestra di essere ridimensionabile
                .open(&mut self.settings_visible)
                // Imposta la visibilità della finestra in base a self.settings_visible
                .show(ui.ctx(), |ui| {
                    // Mostra la finestra nel contesto dell'interfaccia utente
                    egui::ComboBox::new("message_kind", "Message Kind")
                        // Crea una nuova combo box con l'etichetta "Message Kind"
                        .selected_text(self.selected_message.clone().unwrap_or("Select a message".to_string()))
                        // Imposta il testo selezionato nella combo box, o "Select a message" se nessun messaggio è selezionato
                        .show_ui(ui, |ui| {
                            // Mostra la combo box nell'interfaccia utente
                            for message_type in &self.available_messages { //available_messages si popola con ReflectionContext
                                // Itera su ciascun tipo di messaggio disponibile
                                if ui.selectable_label(self.selected_message.as_deref() == Some(message_type), message_type).clicked() {
                                    //Crea un’etichetta selezionabile (selectable_label) per ogni messaggio disponibile.
                                    //Controlla se ci ha cliccato sopra (clicked()).
                                    //Se clicca, cambia il messaggio selezionato.
                                    self.selected_message = Some(message_type.clone());
                                    // Imposta il messaggio selezionato
                                    self.selected_fields.clear();
                                    // Pulisce i campi selezionati quando si seleziona un nuovo messaggio
                                }
                            }
                        });

                    if let Some(ref selected_msg) = self.selected_message {
                        ui.label("Select Fields:");
                        ScrollArea::both()
                        .auto_shrink([false, true]) // Impedisce la riduzione automatica in larghezza
                        .max_width(300.0) // Imposta una larghezza massima più ampia
                        .max_height(100.0)
                        .show(ui, |ui| {
                            let mut select_all = false;
                            let mut deselect_all = false;
                            

                            /*ReflectionContext*
                             * è una struttura o un oggetto che fornisce funzionalità di riflessione per i messaggi MAVLink. 
                             * La riflessione è una tecnica che consente a un programma di ispezionare e manipolare la struttura 
                             * dei dati a runtime. In questo contesto, ReflectionContext viene utilizzato per ottenere informazioni 
                             * sui campi di un messaggio MAVLink specifico.
                             */
                            
                            if let Ok(fields) = ReflectionContext::new().get_fields_by_name(selected_msg) {
                                //restituisce i fields del messaggio selezionato
                                // Ottiene i campi del messaggio selezionato utilizzando ReflectionContext
                                if fields.len() > 1 {
                                    // Se il numero di campi è maggiore di 1
                                    ui.horizontal(|ui| {
                                        // Dispone gli elementi orizzontalmente
                                        ui.checkbox(&mut select_all, "Select All");
                                        // Crea una checkbox per selezionare tutti i campi
                                        ui.checkbox(&mut deselect_all, "Deselect All");
                                        // Crea una checkbox per deselezionare tutti i campi
                                    });
                                }
                            }
                            if select_all {
                                if let Ok(fields) = ReflectionContext::new().get_fields_by_name(selected_msg) {
                                    for field in &fields {
                                        self.selected_fields.insert(field.to_string()); // Seleziona tutti i campi
                                    }
                                }
                            }
                            
                            if deselect_all {
                                self.selected_fields.clear(); // Deseleziona tutti i campi
                            }
                            
                            if let Ok(fields) = ReflectionContext::new().get_fields_by_name(selected_msg) {
                                // Ottiene i campi del messaggio selezionato utilizzando ReflectionContext
                                for field in fields {
                                    // Itera su ciascun campo ottenuto
                                    let mut selected = self.selected_fields.contains(field);
                                    // Verifica se il campo è già selezionato
                                    let response: Response = ui.checkbox(&mut selected, field);
                                    // Crea una checkbox per il campo e aggiorna lo stato di selezione
                                    if response.clicked() {
                                        // Se la checkbox è stata cliccata
                                        if selected {
                                            self.selected_fields.insert(field.to_string());
                                            // Aggiunge il campo selezionato all'insieme dei campi selezionati
                                        } else {
                                            self.selected_fields.remove(field);
                                            // Rimuove il campo deselezionato dall'insieme dei campi selezionati
                                        }
                                    }
                            
                                    if response.drag_started() {
                                        ui.memory_mut(|mem| mem.set_dragged_id(Id::new(field)));
                                        // Imposta l'ID del campo trascinato nella memoria di egui
                                    }
                                    if let Some(dragged) = ui.memory(|mem| mem.dragged_id()) {
                                        // Verifica se c'è un campo trascinato
                                        if dragged == Id::new(field) {
                                            ui.label("Dragging...");
                                            // Mostra un'etichetta durante il trascinamento del campo
                                        }
                                    }
                                }
                            }
                        });
                    }
                    ui.add_space(7.0); // Aggiunge spazio tra le righe
                    ui.label("Sampling Frequency (Hz):");
                    ui.add(egui::Slider::new(&mut self.sampling_frequency, 1.0..=100.0).text("Hz")); // Aggiunge uno slider per la frequenza di campionamento
                });
        }

        // Drag and drop per riordinare i messaggi
        let mut from: Option<usize> = None; // Indice di partenza del trascinamento
        let mut to: Option<usize> = None; // Indice di destinazione del trascinamento

        let frame = Frame::default().fill(Color32::DARK_GRAY); // Crea un frame con sfondo grigio scuro
        let (_, dropped_payload) = ui.dnd_drop_zone::<usize, ()>(frame, |ui| {
            ui.set_min_size(vec2(150.0, 200.0)); // Imposta la dimensione minima della zona di drop

            for (row_idx, item) in self.message_log.iter().enumerate() {
                let item_id = Id::new(("drag_and_drop", row_idx)); // Crea un ID per l'elemento
                let response = ui.dnd_drag_source(item_id, row_idx, |ui| {
                    ui.label(item); // Mostra l'etichetta dell'elemento
                }).response;

                if let (Some(pointer), Some(hovered_payload)) = (
                    ui.input(|i| i.pointer.interact_pos()), // Ottiene la posizione del puntatore
                    response.dnd_hover_payload::<usize>(), // Ottiene il payload del trascinamento
                ) {
                    let rect = response.rect; // Ottiene il rettangolo dell'elemento
                    let stroke = egui::Stroke::new(1.0, Color32::WHITE); // Crea una linea bianca
                    let insert_row_idx = if *hovered_payload == row_idx {
                        ui.painter().hline(rect.x_range(), rect.center().y, stroke); // Disegna una linea orizzontale al centro
                        row_idx
                    } else if pointer.y < rect.center().y {
                        ui.painter().hline(rect.x_range(), rect.top(), stroke); // Disegna una linea orizzontale in alto
                        row_idx
                    } else {
                        ui.painter().hline(rect.x_range(), rect.bottom(), stroke); // Disegna una linea orizzontale in basso
                        row_idx + 1
                    };

                    if let Some(dragged_payload) = response.dnd_release_payload() {
                        from = Some(*dragged_payload); // Imposta l'indice di partenza
                        to = Some(insert_row_idx); // Imposta l'indice di destinazione
                    }
                }
            }
        });

        if let Some(dragged_payload) = dropped_payload {
            from = Some(*dragged_payload); // Imposta l'indice di partenza se c'è un payload rilasciato
            to = Some(usize::MAX); // Imposta l'indice di destinazione al massimo valore
        }

        if let (Some(from), Some(mut to)) = (from, to) {
            to -= (from < to) as usize; // Regola l'indice di destinazione
            let item = self.message_log.remove(from); // Rimuove l'elemento dall'indice di partenza
            to = to.min(self.message_log.len()); // Limita l'indice di destinazione alla lunghezza del registro
            self.message_log.insert(to, item); // Inserisce l'elemento all'indice di destinazione
        }

        response // Restituisce la risposta del pannello
    }
    
    fn contains_pointer(&self) -> bool {
        self.contains_pointer // Restituisce se il puntatore è contenuto nel pannello
    }
    
    fn update(&mut self, messages: &[TimedMessage]) {
        //let message = messages.into_iter().last()
        //if let Some (message) = message 

        for msg in messages {
            let msg_type = format!("{:?}", msg.message); // Formatta il tipo di messaggio come stringa
            if self.seen_message_types.insert(msg_type.clone()) {
                self.available_messages.push(msg_type.clone()); // Aggiunge il tipo di messaggio ai messaggi disponibili
                self.available_messages.sort(); // Ordina i messaggi disponibili
            }
            if self.selected_message.as_deref() == Some(&msg_type) {
                let mut log_entry = format!("[{}] {}", msg.time.elapsed().as_secs(), msg_type); // Crea una voce di registro con il tempo trascorso e il tipo di messaggio
                let fields: Vec<_> = self.selected_fields.iter().take(5).collect(); // Ottiene i primi 5 campi selezionati
                for field in fields {
                    if let Ok(values) = extract_from_message::<_, String>(&msg.message, vec![field.to_string()]) {
                        if !values.is_empty() {
                            log_entry.push_str(&format!(" | {}: {:?}", field, values[0])); // Aggiunge i valori dei campi alla voce di registro
                        }
                    }
                }
                self.message_log.push(log_entry); // Aggiunge la voce di registro al registro dei messaggi
                if self.message_log.len() > 1000 {
                    self.message_log.remove(0); // Rimuove la voce più vecchia se il registro supera 1000 voci
                }
            }
        }
    }
    
    fn get_message_subscription(&self) -> Option<u32> {
        None // Non restituisce alcuna sottoscrizione ai messaggi
    }
    
    fn should_send_message_history(&self) -> bool {
        false // Non invia la cronologia dei messaggi
    }
}

// La macro #[derive(Clone, Debug, Serialize, Deserialize)] genera automaticamente le implementazioni
// per i tratti Clone, Debug, Serialize e Deserialize per la struttura MsgSources.
// E la mia struttura di come sono fatti i dati dei messaggi
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MsgSources {
    msg_id: u32, // ID del messaggio
    fields_with_checkbox: Vec<(String, bool)>, // Campi con checkbox
}

// Implementazione del tratto Default per MsgSources
impl Default for MsgSources {
    fn default() -> Self {
        Self {
            //come default devo far vedere un messaggio in particolare ?
            msg_id: 0, // Imposta l'ID del messaggio su 0
            fields_with_checkbox: Vec::new(), // Inizializza fields_with_checkbox come un vettore vuoto
        }
    }
}

// Implementazione del tratto PartialEq per MsgSources
impl PartialEq for MsgSources {
    fn eq(&self, other: &Self) -> bool {
        self.msg_id == other.msg_id
            && self.fields_with_checkbox == other.fields_with_checkbox // Confronta l'uguaglianza dei campi
    }
}