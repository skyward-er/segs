use core::f32;

use glam::Vec2;
use serde::{Deserialize, Serialize};

const DEFAULT_SIZE: f32 = 10.0;
const MIN_SIZE: f32 = 5.0;
const MAX_SIZE: f32 = 50.0;
const SCROLL_DELTA: f32 = 1.0;

pub const CONNECTION_LINE_THRESHOLD: f32 = 5.0; // Pixels
pub const CONNECTION_LINE_THICKNESS: f32 = 0.2; // Grid units
pub const CONNECTION_POINT_SIZE: f32 = 1.0; // Grid units

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct GridInfo {
    /// Grid's zero position on screen
    pub zero_pos: Vec2,
    size: f32,
}

impl Default for GridInfo {
    fn default() -> Self {
        Self {
            zero_pos: Vec2::ZERO,
            size: DEFAULT_SIZE,
        }
    }
}

impl GridInfo {
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
    pub fn grid_to_screen(&self, p_g: Vec2) -> Vec2 {
        p_g * self.size + self.zero_pos
    }

    /// Screen to grid coordinates transformation
    pub fn screen_to_grid(&self, p_s: Vec2) -> Vec2 {
        (p_s - self.zero_pos) / self.size
    }
}
