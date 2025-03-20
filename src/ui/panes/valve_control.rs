use egui_tiles::TileId;
use serde::{Deserialize, Serialize};

use crate::ui::app::PaneResponse;

use super::PaneBehavior;

mod enums;

#[derive(Clone, PartialEq, Default, Serialize, Deserialize, Debug)]
pub struct ValveControlPane {
    // Temporary Internal state
    #[serde(skip)]
    contains_pointer: bool,
}

impl PaneBehavior for ValveControlPane {
    fn ui(&mut self, ui: &mut egui::Ui, tile_id: TileId) -> PaneResponse {
        todo!()
    }

    fn contains_pointer(&self) -> bool {
        self.contains_pointer
    }
}
