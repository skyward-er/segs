use super::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
pub struct Defs {
    #[serde(rename = "path")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<Path>,
}
