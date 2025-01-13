use super::symbols::Symbol;
use super::{grid::GridInfo, pos::Pos};
use egui::Pos2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Element {
    /// Ancor postion in the grid, symbol center
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
        let start = self.position.add_size(-self.size / 2).into_pos2(grid);
        let end = self.position.add_size(self.size / 2).into_pos2(grid);

        (start.x <= pos.x && pos.x < end.x) && (start.y <= pos.y && pos.y < end.y)
    }
}
