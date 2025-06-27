use serde::{Deserialize, Serialize};

use egui::{
    Align2, Color32, CornerRadius, FontId, Stroke, StrokeKind, TextEdit, Theme, Ui, Window,
};
use glam::Vec2;

use crate::{
    mavlink::MavMessage,
    ui::utils::{egui_to_glam, glam_to_egui},
};

use super::SymbolBehavior;

const FONT_SIZE: f32 = 2.0;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Label {
    text: String,
    size: Vec2,

    #[serde(skip)]
    is_window_visible: bool,
}

impl Default for Label {
    fn default() -> Self {
        Self {
            text: "Label".to_string(),
            size: Vec2::new(FONT_SIZE * 0.6 * 4.0, FONT_SIZE),
            is_window_visible: false,
        }
    }
}

impl SymbolBehavior for Label {
    fn update(&mut self, _message: &MavMessage, _subscribed_msg_ids: &[u32]) {}

    fn reset_subscriptions(&mut self) {}

    fn paint(&mut self, ui: &mut Ui, theme: Theme, pos: Vec2, size: f32, _: f32) {
        let painter = ui.painter();
        let color = match theme {
            Theme::Light => Color32::BLACK,
            Theme::Dark => Color32::WHITE,
        };

        let text_rect = painter.text(
            glam_to_egui(pos).to_pos2(),
            Align2::LEFT_TOP,
            &self.text,
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

    fn subscriptions_ui(&mut self, ui: &mut Ui, _mavlink_ids: &[u32]) {
        Window::new("Label Customization")
            .id(ui.auto_id_with("customization_settings"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut self.is_window_visible)
            .show(ui.ctx(), |ui| subscription_window(ui, &mut self.text));
    }

    fn context_menu(&mut self, ui: &mut Ui) {
        if ui.button("Customize text…").clicked() {
            self.is_window_visible = true;
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

fn subscription_window(ui: &mut Ui, text: &mut String) {
    TextEdit::singleline(text)
        .hint_text("Label text")
        .desired_width(200.0)
        .show(ui);
}
