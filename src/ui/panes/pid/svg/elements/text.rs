use crate::ui::panes::pid::svg::{
    attributes::transform::Transform,
    utils::{is_default, is_zero},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Text {
    #[serde(rename = "@font-size")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub font_size: f32,

    #[serde(rename = "@font-family")]
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub font_family: String,

    #[serde(rename = "@transform")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub transform: Transform,

    #[serde(rename = "@segs-format")]
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub format: String,

    #[serde(rename = "$text")]
    pub text: String,
}

impl Text {
    pub fn new(text: String, size: f32) -> Self {
        Self {
            font_size: size,
            font_family: "monospace".to_string(),
            transform: Transform::default(),
            format: "TODO".to_string(),
            text,
        }
    }
}
