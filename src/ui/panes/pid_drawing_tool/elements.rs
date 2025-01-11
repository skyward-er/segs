use super::pos::Pos;
use super::symbols::Symbol;
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
    pub fn contains(&self, pos: &Pos) -> bool {
        self.position <= *pos && *pos < self.position.add_size(self.size)
    }
}
