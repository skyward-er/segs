use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, PartialOrd, Default)]
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
}
