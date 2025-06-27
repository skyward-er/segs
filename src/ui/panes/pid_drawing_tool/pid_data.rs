use std::path::Path;

use egui::ahash::HashMap;
use serde::{Deserialize, Serialize};

use super::{connections::Connection, elements::Element};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PidData {
    pub elements: HashMap<u32, Element>,
    pub connections: Vec<Connection>,
    pub message_subscription_ids: Vec<u32>,
}

impl PidData {
    pub fn to_file(&self, file_path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let file = std::fs::File::create(file_path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    pub fn from_file(file_path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(file_path)?;
        let data: Self = serde_json::from_reader(file)?;
        Ok(data)
    }
}
