use serde::{Deserialize, Serialize};

use egui::{Align2, Color32, CornerRadius, FontId, Stroke, StrokeKind, Theme, Ui};
use glam::Vec2;

use crate::{
    MAVLINK_PROFILE,
    error::ErrInstrument,
    mavlink::{
        GSE_TM_DATA, MavMessage, Message, MessageData,
        reflection::{FieldLike, IndexedField},
    },
    ui::utils::{egui_to_glam, glam_to_egui},
};

use super::SymbolBehavior;

const FONT_SIZE: f32 = 2.0;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Label {
    text: String,
    units: String,
    #[serde(skip)]
    show_window: bool,

    last_value: Option<f32>,
    mavlink_field: IndexedField,
    size: Vec2,
}

impl Default for Label {
    fn default() -> Self {
        Self {
            text: "0.00".to_string(),
            units: "".to_string(),
            show_window: false,
            mavlink_field: 6
                .to_mav_field(GSE_TM_DATA::ID, &MAVLINK_PROFILE)
                .log_unwrap(), // n2_vessel_1_pressure for GSE_TM_DATA
            last_value: Some(0.0),
            size: Vec2::new(FONT_SIZE * 0.6 * 4.0, FONT_SIZE),
        }
    }
}

impl SymbolBehavior for Label {
    fn paint(&mut self, ui: &Ui, theme: Theme, pos: Vec2, size: f32, _: f32) {
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

        println!("Drawing label edit window {}", self.show_window);
        let mut show_window = self.show_window;
        egui::Window::new("Label")
            .id(ui.id())
            .auto_sized()
            .collapsible(false)
            .movable(true)
            .open(&mut show_window)
            .show(ui.ctx(), |ui| {
                ui.text_edit_singleline(&mut self.units);
            });
        self.show_window = show_window;
    }

    fn update(&mut self, message: &MavMessage) {
        if message.message_id() == GSE_TM_DATA::ID {
            let value = self.mavlink_field.extract_as_f64(message).log_unwrap();
            self.last_value = Some(value as f32);
            self.text = format!("{:.2}{}", value, self.units);
        }
    }

    fn anchor_points(&self) -> Option<Vec<Vec2>> {
        None
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn context_menu(&mut self, ui: &mut Ui) {
        println!("Label context menu");
        if ui.button("Edit").clicked() {
            self.show_window = true;
            ui.close_menu();
        }
    }
}
