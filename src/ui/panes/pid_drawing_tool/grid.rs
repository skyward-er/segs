use egui::Pos2;
use serde::{Deserialize, Serialize};

const DEFAULT_SIZE: f32 = 10.0;
const MIN_SIZE: f32 = 5.0;
const MAX_SIZE: f32 = 50.0;
const SCROLL_DELTA: f32 = 1.0;

pub const LINE_DISTANCE_THRESHOLD: f32 = 5.0;
pub const LINE_THICKNESS: f32 = 0.2;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct GridInfo {
    pub zero_pos: Pos2,
    size: f32,
}

impl Default for GridInfo {
    fn default() -> Self {
        Self {
            zero_pos: Pos2::ZERO,
            size: DEFAULT_SIZE,
        }
    }
}

impl GridInfo {
    pub fn get_size(&self) -> f32 {
        self.size
    }

    pub fn apply_scroll_delta(&mut self, delta: f32, pos: &Pos2) {
        if delta == 0.0 {
            return;
        }

        let old_size = self.size;
        if delta > 0.0 {
            self.size += SCROLL_DELTA;
        } else {
            self.size -= SCROLL_DELTA;
        };

        self.size = self.size.clamp(MIN_SIZE, MAX_SIZE);

        if self.size != old_size {
            let delta_prop = self.size / old_size - 1.0;
            let pos_delta = delta_prop * (self.zero_pos - pos.to_vec2());
            self.zero_pos += pos_delta.to_vec2();
        }
    }
}
