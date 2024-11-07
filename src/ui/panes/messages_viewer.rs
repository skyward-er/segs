use egui::Label;
use serde::{Deserialize, Serialize};

use crate::ui::composable_view::PaneResponse;

use super::PaneBehavior;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MessagesViewerPane {
    #[serde(skip)]
    contains_pointer: bool,
}

impl PaneBehavior for MessagesViewerPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let mut response = PaneResponse::default();
        let label = ui.add_sized(ui.available_size(), Label::new("This is a label"));
        self.contains_pointer = label.contains_pointer();
        if label.drag_started() {
            response.set_drag_started();
        }
        response
    }

    fn tab_title(&self) -> egui::WidgetText {
        "Messages".into()
    }

    fn contains_pointer(&self) -> bool {
        self.contains_pointer
    }
}
