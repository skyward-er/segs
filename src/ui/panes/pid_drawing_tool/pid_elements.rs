use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PidElement {
    pub x: i32,
    pub y: i32,
    pub size: i32,
}

impl PidElement {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        (x >= self.x && x < (self.x + self.size)) && (y >= self.y && y < (self.y + self.size))
    }
}
