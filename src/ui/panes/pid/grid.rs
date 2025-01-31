use core::f32;

use egui::{Color32, Pos2, Theme, Ui, Vec2};
use serde::{Deserialize, Serialize};

const DEFAULT_SIZE: f32 = 10.0;
const MIN_SIZE: f32 = 5.0;
const MAX_SIZE: f32 = 50.0;
const SCROLL_DELTA: f32 = 1.0;

pub const CONNECTION_LINE_THRESHOLD: f32 = 5.0; // Pixels
pub const CONNECTION_LINE_THICKNESS: f32 = 0.2; // Grid units
pub const CONNECTION_POINT_SIZE: f32 = 1.0; // Grid units

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Grid {
    pub zero_pos: Vec2,
    size: f32,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            zero_pos: Vec2::ZERO,
            size: DEFAULT_SIZE,
        }
    }
}

impl Grid {
    pub fn from_size(size: f32) -> Self {
        Self {
            zero_pos: Vec2::ZERO,
            size,
        }
    }

    /// Returns the grid size
    pub fn size(&self) -> f32 {
        self.size
    }

    /// Applies the scroll delta at the given position (in screen coordinates)
    pub fn apply_scroll_delta(&mut self, delta: f32, pos_s: Vec2) {
        if delta == 0.0 || delta == f32::NAN {
            return;
        }

        let old_size = self.size;
        let delta = delta.signum() * SCROLL_DELTA;
        self.size = (self.size + delta).clamp(MIN_SIZE, MAX_SIZE);

        if self.size != old_size {
            self.zero_pos += (delta / old_size) * (self.zero_pos - pos_s);
        }
    }

    /// Grid to screen coordinates transformation
    pub fn grid_to_screen(&self, p_g: Pos2) -> Pos2 {
        p_g * self.size + self.zero_pos
    }

    /// Screen to grid coordinates transformation
    pub fn screen_to_grid(&self, p_s: Pos2) -> Pos2 {
        (p_s - self.zero_pos) / self.size
    }

    fn dots_color(theme: Theme) -> Color32 {
        match theme {
            Theme::Dark => Color32::DARK_GRAY,
            Theme::Light => Color32::BLACK,
        }
    }

    pub fn draw(&self, ui: &Ui, theme: Theme) {
        let painter = ui.painter();
        let window_rect = ui.max_rect();
        let dot_color = Self::dots_color(theme);

        let offset_x = (self.zero_pos.x % self.size()) as i32;
        let offset_y = (self.zero_pos.y % self.size()) as i32;

        let start_x = (window_rect.min.x / self.size()) as i32 * self.size() as i32 + offset_x;
        let end_x = (window_rect.max.x / self.size() + 2.0) as i32 * self.size() as i32 + offset_x;
        let start_y = (window_rect.min.y / self.size()) as i32 * self.size() as i32 + offset_y;
        let end_y = (window_rect.max.y / self.size() + 2.0) as i32 * self.size() as i32 + offset_y;

        for x in (start_x..end_x).step_by(self.size() as usize) {
            for y in (start_y..end_y).step_by(self.size() as usize) {
                let rect = egui::Rect::from_min_size(
                    egui::Pos2::new(x as f32, y as f32),
                    egui::Vec2::new(1.0, 1.0),
                );
                painter.rect_filled(rect, 0.0, dot_color);
            }
        }
    }
}
