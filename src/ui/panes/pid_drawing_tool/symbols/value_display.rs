use serde::{Deserialize, Serialize};

use egui::{
    Align2, Color32, CornerRadius, FontId, RichText, Stroke, StrokeKind, Theme, Ui, Window,
};
use glam::Vec2;

use crate::{
    error::ErrInstrument,
    mavlink::reflection::MAVLINK_PROFILE,
    mavlink::{MavMessage, Message, reflection::IndexedField},
    ui::{
        cache::ChangeTracker,
        utils::{egui_to_glam, glam_to_egui},
    },
};

use super::SymbolBehavior;

const FONT_SIZE: f32 = 2.0;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ValueDisplay {
    subscribed_field: Option<IndexedField>,
    size: Vec2,

    #[serde(skip)]
    last_value: Option<f32>,
    #[serde(skip)]
    is_subs_window_visible: bool,
}

impl Default for ValueDisplay {
    fn default() -> Self {
        Self {
            subscribed_field: None,
            last_value: Some(0.0),
            size: Vec2::new(FONT_SIZE * 0.6 * 4.0, FONT_SIZE),
            is_subs_window_visible: false,
        }
    }
}

impl SymbolBehavior for ValueDisplay {
    fn update(&mut self, message: &MavMessage, subscribed_msg_ids: &[u32]) {
        // Reset field if msg_id has changed
        if let Some(inner_field) = &self.subscribed_field {
            if !subscribed_msg_ids.contains(&inner_field.msg_id()) {
                self.subscribed_field = None;
            }
        }

        if let Some(subscribed_field) = &self.subscribed_field {
            if message.message_id() == subscribed_field.msg_id() {
                let value = subscribed_field.extract_as_f64(message).log_unwrap();
                self.last_value = Some(value as f32);
            }
        }
    }

    fn reset_subscriptions(&mut self) {
        self.subscribed_field = None;
        self.last_value = None;
    }

    fn paint(&mut self, ui: &mut Ui, theme: Theme, pos: Vec2, size: f32, _: f32) {
        let painter = ui.painter();
        let color = match theme {
            Theme::Light => Color32::BLACK,
            Theme::Dark => Color32::WHITE,
        };

        let unit = self
            .subscribed_field
            .as_ref()
            .and_then(|f| f.field().unit.as_deref())
            .unwrap_or("");
        let text = match self.last_value {
            Some(value) => format!("{:.5} {}", value, unit),
            None => "N/A".to_string(),
        };
        let text_rect = painter.text(
            glam_to_egui(pos).to_pos2(),
            Align2::LEFT_TOP,
            text,
            FontId::monospace(FONT_SIZE * size),
            color,
        );
        self.size = egui_to_glam(text_rect.size()) / size;
        painter.rect(
            egui::Rect::from_min_size(
                glam_to_egui(pos).to_pos2(),
                glam_to_egui(self.size()) * size,
            ),
            CornerRadius::ZERO,
            Color32::TRANSPARENT,
            Stroke::NONE,
            StrokeKind::Middle,
        );
    }

    fn subscriptions_ui(&mut self, ui: &mut Ui, mavlink_ids: &[u32]) {
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

    fn context_menu(&mut self, ui: &mut Ui) {
        if ui.button("Label subscription settings…").clicked() {
            self.is_subs_window_visible = true;
            ui.close_menu();
        }
    }

    fn anchor_points(&self) -> Option<Vec<Vec2>> {
        None
    }

    fn size(&self) -> Vec2 {
        self.size
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
        .get_plottable_fields(current_msg_id)
        .log_expect("Invalid message id");

    // If no fields available for subscription
    if fields.is_empty() {
        ui.label(
            RichText::new("No fields available for subscription")
                .underline()
                .strong(),
        );
        return;
    }

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
