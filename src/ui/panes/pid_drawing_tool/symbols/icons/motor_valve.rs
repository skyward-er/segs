use egui::{RichText, Ui, Window};
use serde::{Deserialize, Serialize};

use crate::{
    error::ErrInstrument,
    mavlink::reflection::MAVLINK_PROFILE,
    mavlink::{MavMessage, Message, reflection::IndexedField},
    ui::cache::ChangeTracker,
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct MotorValve {
    subscribed_field: Option<IndexedField>,

    /// false = closed, true = open
    #[serde(skip)]
    pub last_value: Option<bool>,
    #[serde(skip)]
    pub is_subs_window_visible: bool,
}

impl MotorValve {
    pub fn update(&mut self, msg: &MavMessage, subscribed_msg_ids: &[u32]) {
        // Reset field if msg_id has changed
        if let Some(inner_field) = &self.subscribed_field {
            if !subscribed_msg_ids.contains(&inner_field.msg_id()) {
                self.subscribed_field = None;
            }
        }

        if let Some(field) = &self.subscribed_field {
            if field.msg_id() == msg.message_id() {
                let value = field.extract_as_f64(msg).log_unwrap();
                self.last_value = Some(value != 0.0);
            }
        }
    }

    pub fn reset_subscriptions(&mut self) {
        self.subscribed_field = None;
        self.last_value = None;
    }

    pub fn subscriptions_ui(&mut self, ui: &mut Ui, mavlink_ids: &[u32]) {
        let change_tracker = ChangeTracker::record_initial_state(&self.subscribed_field);
        Window::new("Subscriptions")
            .id(ui.auto_id_with("subs_settings"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut self.is_subs_window_visible)
            .show(ui.ctx(), |ui| {
                subscription_window(ui, mavlink_ids, &mut self.subscribed_field)
            });
        // reset last_value if the subscribed field has changed
        if change_tracker.has_changed(&self.subscribed_field) {
            self.last_value = None;
        }
    }
}

fn subscription_window(ui: &mut Ui, msg_ids: &[u32], field: &mut Option<IndexedField>) {
    let mut current_msg_id = field.as_ref().map(|f| f.msg_id()).unwrap_or(msg_ids[0]);

    // extract the msg name from the id to show it in the combo box
    let msg_name = MAVLINK_PROFILE
        .get_msg(current_msg_id)
        .map(|m| m.name.clone())
        .unwrap_or_default();

    // show the first combo box with the message name selection
    let msg_digest = ChangeTracker::record_initial_state(current_msg_id);
    egui::ComboBox::from_label("Message Kind")
        .selected_text(msg_name)
        .show_ui(ui, |ui| {
            for msg in MAVLINK_PROFILE
                .get_sorted_msgs()
                .into_iter()
                .filter(|msg| msg_ids.contains(&msg.id))
            {
                ui.selectable_value(&mut current_msg_id, msg.id, &msg.name);
            }
        });
    // reset field if the message is changed
    if msg_digest.has_changed(&current_msg_id) {
        *field = None;
    }

    // Get all fields available for subscription
    let fields = MAVLINK_PROFILE
        .get_all_state_fields(current_msg_id)
        .log_unwrap();

    // If no fields available for subscription
    if fields.is_empty() {
        ui.label(
            RichText::new("No fields available for subscription")
                .underline()
                .strong(),
        );
        return;
    };

    // Otherwise, select the first field available
    let field = field.get_or_insert(fields[0].to_owned());
    egui::ComboBox::from_label("field")
        .selected_text(&field.field().name)
        .show_ui(ui, |ui| {
            for msg in fields.iter() {
                ui.selectable_value(field, msg.to_owned(), &msg.field().name);
            }
        });
}
