use egui::Pos2;
use serde::{Deserialize, Serialize};

use super::grid::GridInfo;

#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn add_size(&self, size: i32) -> Self {
        Self {
            x: self.x + size,
            y: self.y + size,
        }
    }

    pub fn to_pos2(&self, grid: &GridInfo) -> Pos2 {
        Pos2 {
            x: self.x as f32 * grid.size + grid.zero_pos.x,
            y: self.y as f32 * grid.size + grid.zero_pos.y,
        }
    }

    pub fn to_relative_pos2(&self, grid: &GridInfo) -> Pos2 {
        Pos2 {
            x: self.x as f32 * grid.size,
            y: self.y as f32 * grid.size,
        }
    }

    pub fn from_pos2(grid: &GridInfo, pos: &Pos2) -> Self {
        Self {
            x: ((pos.x - grid.zero_pos.x) / grid.size) as i32,
            y: ((pos.y - grid.zero_pos.y) / grid.size) as i32,
        }
    }

    pub fn distance(&self, grid: &GridInfo, pos: &Pos2) -> f32 {
        let me = self.to_pos2(grid);

        ((me.x - pos.x).powi(2) + (me.y - pos.y).powi(2)).sqrt()
    }
}
