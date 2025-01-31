use super::super::{
    attributes::transform::Transform,
    utils::{is_default, is_zero},
};
use egui::{InputState, Pos2};
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Use {
    #[serde(rename = "@href", with = "href")]
    pub href: String,

    #[serde(rename = "@width")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub width: f32,

    #[serde(rename = "@height")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub height: f32,

    #[serde(rename = "@transform")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub transform: Transform,
}

impl Use {
    pub fn handle_click(&mut self, _pos: Pos2) {}
    pub fn handle_double_click(&mut self, _pos: Pos2) {}

    /// Consumes any shortcut the element should respond to
    pub fn handle_shortcuts(&mut self, _input: &mut InputState) {}

    pub fn hovered(&self, pos: Pos2) -> bool {
        let pos = self.transform.to_local_frame(pos).to_vec2();

        // The bounding box in the elemen's frame is defined by the size. But
        // width and height can be negative. This allows to represent where the
        // position anchor point is (for paths is the top-left, for texts
        // is the bottom-left)
        let size = Vec2::new(self.width, self.height);
        let min = Vec2::ZERO.min(size);
        let max = Vec2::ZERO.max(size);

        // Check if the point is in the bounding box
        min.x <= pos.x && pos.x <= max.x && min.y <= pos.y && pos.y <= max.y
    }

    /// Whether the elemen can be moved
    pub fn draggable(&self) -> bool {
        false
    }

    /// Whether a window to edit the element's configuration is available
    pub fn editable(&self) -> bool {
        false
    }

    pub fn who_am_i(&self) -> String {
        format!(
            "{} at x={} y={} width={} height={}",
            self.href,
            self.transform.translate.x,
            self.transform.translate.y,
            self.width,
            self.height
        )
    }
}

mod href {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(id: &str, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        format!("#{id}").serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut id = String::deserialize(deserializer)?;
        id.remove(0);
        Ok(id)
    }
}
