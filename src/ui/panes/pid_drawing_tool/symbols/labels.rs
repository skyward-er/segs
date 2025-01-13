use serde::{Deserialize, Serialize};

use crate::ui::utils::glam_to_egui;

use super::SymbolBehavior;
use egui::{Align2, Color32, FontId, Rounding, Stroke, Theme, Ui};
use glam::Vec2;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Label {
    last_value: Option<f32>,
    format_string: String,
    #[serde(skip)]
    show_window: bool,
}

impl SymbolBehavior for Label {
    fn paint(&mut self, ui: &Ui, theme: Theme, pos: Vec2, size: f32, _: f32) {
        let painter = ui.painter();
        let color = match theme {
            Theme::Light => Color32::BLACK,
            Theme::Dark => Color32::WHITE,
        };

        painter.text(
            glam_to_egui(pos).to_pos2(),
            Align2::LEFT_TOP,
            &self.format_string,
            FontId::monospace(self.size().y * size),
            color,
        );
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
                ui.text_edit_singleline(&mut self.format_string);
            });
        self.show_window = show_window;
    }

    fn anchor_points(&self) -> Option<Vec<Vec2>> {
        None
    }

    fn size(&self) -> Vec2 {
        let font_size = 2.0;
        Vec2::new(font_size * 0.6 * self.format_string.len() as f32, font_size)
    }

    fn context_menu(&mut self, ui: &mut Ui) {
        if ui.button("Edit").clicked() {
            self.show_window = true;
            ui.close_menu();
        }
    }
}
