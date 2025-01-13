use serde::{Deserialize, Serialize};

pub const LINE_DISTANCE_THRESHOLD: f32 = 5.0; // Pixels

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct GridInfo {
    pub size: f32,
}
