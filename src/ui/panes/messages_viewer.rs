use egui::Label;
use serde::{Deserialize, Serialize};

use crate::ui::composable_view::PaneResponse;

use super::PaneBehavior;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MessagesViewerPane {
    #[serde(skip)]
    contains_pointer: bool,
}

impl PartialEq for MessagesViewerPane {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl PaneBehavior for MessagesViewerPane {
    #[profiling::function]
    fn ui(&mut self, ui: &mut egui::Ui, _tile_id: egui_tiles::TileId) -> PaneResponse {
        let mut response = PaneResponse::default();
        let label = ui.add_sized(ui.available_size(), Label::new("This is a label"));
        self.contains_pointer = label.contains_pointer();
        if label.drag_started() {
            response.set_drag_started();
        }
        response
    }

    fn contains_pointer(&self) -> bool {
        self.contains_pointer
    }

    fn update(&mut self, _messages: &[crate::mavlink::TimedMessage]) {}

    fn get_message_subscription(&self) -> Option<u32> {
        None
    }

    fn should_send_message_history(&self) -> bool {
        false
    }
}
