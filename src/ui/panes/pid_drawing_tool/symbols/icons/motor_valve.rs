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
    pub fn update(&mut self, msg: &MavMessage, subscribed_msg_id: u32) {
        // Reset field if msg_id has changed
        if let Some(inner_field) = &self.subscribed_field {
            if inner_field.msg_id() != subscribed_msg_id {
                self.subscribed_field = None;
            }
        }

        if let Some(field) = &self.subscribed_field {
            if msg.message_id() == subscribed_msg_id {
                let value = field.extract_as_f64(msg).log_unwrap();
                self.last_value = Some(value != 0.0);
            }
        }
    }

    pub fn reset_subscriptions(&mut self) {
        self.subscribed_field = None;
        self.last_value = None;
    }

    pub fn subscriptions_ui(&mut self, ui: &mut Ui, mavlink_id: u32) {
        let change_tracker = ChangeTracker::record_initial_state(&self.subscribed_field);
        Window::new("Subscriptions")
            .id(ui.auto_id_with("subs_settings"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut self.is_subs_window_visible)
            .show(ui.ctx(), |ui| {
                subscription_window(ui, mavlink_id, &mut self.subscribed_field)
            });
        // reset last_value if the subscribed field has changed
        if change_tracker.has_changed(&self.subscribed_field) {
            self.last_value = None;
        }
    }
}

fn subscription_window(ui: &mut Ui, msg_id: u32, field: &mut Option<IndexedField>) {
    // Get all fields available for subscription
    let fields = MAVLINK_PROFILE.get_all_state_fields(msg_id).log_unwrap();

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
