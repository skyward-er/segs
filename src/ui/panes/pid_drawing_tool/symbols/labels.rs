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
    mavlink_field: IndexedField,
    size: Vec2,

    #[serde(skip)]
    last_value: Option<f32>,
}

impl Default for Label {
    fn default() -> Self {
        Self {
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

        let unit = self.mavlink_field.field().unit.as_deref().unwrap_or("");
        let text = match self.last_value {
            Some(value) => format!("{:.2} {}", value, unit),
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

    fn update(&mut self, message: &MavMessage) {
        if message.message_id() == GSE_TM_DATA::ID {
            let value = self.mavlink_field.extract_as_f64(message).log_unwrap();
            self.last_value = Some(value as f32);
        }
    }

    fn anchor_points(&self) -> Option<Vec<Vec2>> {
        None
    }

    fn size(&self) -> Vec2 {
        self.size
    }
}
