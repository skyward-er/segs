use super::{path::Path, text::Text};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
pub struct Defs {
    #[serde(rename = "path")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<Path>,

    #[serde(rename = "text")]
    #[serde(default)]
    pub texts: Vec<Text>,
}
