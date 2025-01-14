use super::symbols::Symbol;
use super::{grid::GridInfo, pos::Pos};
use egui::{Pos2, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Element {
    /// Anchor postion in the grid, symbol center
    pub position: Pos,

    /// Size in grid units
    pub size: i32,

    /// Rotation in radiants
    pub rotation: f32,

    /// Symbol to be displayed
    pub symbol: Symbol,
}

impl Element {
    pub fn new(pos: Pos, symbol: Symbol) -> Self {
        Self {
            position: pos,
            size: 10,
            rotation: 0.0,
            symbol,
        }
    }

    pub fn contains(&self, grid: &GridInfo, pos: &Pos2) -> bool {
        let start = self.position.add_size(-self.size / 2).to_pos2(grid);
        let end = self.position.add_size(self.size / 2).to_pos2(grid);

        (start.x <= pos.x && pos.x < end.x) && (start.y <= pos.y && pos.y < end.y)
    }

    pub fn get_anchor(&self, grid: &GridInfo, idx: usize) -> Pos2 {
        let anchor = self.symbol.get_anchor_points()[idx];
        let anchor = Vec2::from(anchor) * self.size as f32 * grid.get_size();

        self.position.to_pos2(grid) + anchor
    }
}
