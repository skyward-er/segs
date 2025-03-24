use egui::{Label, Ui};
use serde::{Deserialize, Serialize};

use crate::ui::{app::PaneResponse, shortcuts::ShortcutHandler};

use super::PaneBehavior;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MessagesViewerPane;

impl PaneBehavior for MessagesViewerPane {
    #[profiling::function]
    fn ui(&mut self, ui: &mut Ui, _shortcut_handler: &mut ShortcutHandler) -> PaneResponse {
        let mut response = PaneResponse::default();
        let label = ui.add_sized(ui.available_size(), Label::new("This is a label"));
        if label.drag_started() {
            response.set_drag_started();
        }
        response
    }
}
