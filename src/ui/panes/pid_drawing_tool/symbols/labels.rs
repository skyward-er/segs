use serde::{Deserialize, Serialize};
use skyward_mavlink::{mavlink::MessageData, orion};

use crate::{
    mavlink::{extract_from_message, MavlinkResult, MessageView, ViewId},
    ui::utils::{egui_to_glam, glam_to_egui},
};

use super::{MavlinkValue, SymbolBehavior};
use egui::{Align2, Color32, FontId, Rounding, Stroke, Theme, Ui};
use glam::Vec2;

const FONT_SIZE: f32 = 2.0;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Label {
    text: String,
    units: String,
    #[serde(skip)]
    show_window: bool,

    last_value: Option<f32>,
    source: MavlinkValue,
    size: Vec2,
}

impl Default for Label {
    fn default() -> Self {
        Self {
            text: "0.00".to_string(),
            units: "".to_string(),
            show_window: false,
            source: MavlinkValue {
                msg_id: orion::GSE_TM_DATA::ID,
                field: "n2o_vessel_pressure".to_string(),
                view_id: ViewId::new(),
            },
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
            Rounding::ZERO,
            Color32::TRANSPARENT,
            Stroke::new(1.0, color),
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

impl MessageView for Label {
    fn view_id(&self) -> ViewId {
        self.source.view_id
    }

    fn id_of_interest(&self) -> u32 {
        self.source.msg_id
    }

    fn is_valid(&self) -> bool {
        self.last_value.is_some()
    }

    fn populate_view(
        &mut self,
        msg_slice: &[crate::mavlink::TimedMessage],
    ) -> crate::mavlink::MavlinkResult<()> {
        self.update_view(msg_slice)
    }

    fn update_view(
        &mut self,
        msg_slice: &[crate::mavlink::TimedMessage],
    ) -> crate::mavlink::MavlinkResult<()> {
        if let Some(msg) = msg_slice.last() {
            let values: MavlinkResult<Vec<Option<f32>>> =
                extract_from_message(&msg.message, [&self.source.field]);
            if let Ok(values) = values {
                if !values.is_empty() {
                    if let Some(value) = values[0] {
                        self.last_value = Some(value);
                        self.text = format!("{:.2}{}", value, self.units);
                    }
                }
            }
        }
        Ok(())
    }
}
