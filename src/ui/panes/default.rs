use super::PaneBehavior;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::ui::{
    composable_view::{PaneAction, PaneResponse},
    utils::{vertically_centered, SizingMemo},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DefaultPane {
    #[serde(skip)]
    centering_memo: SizingMemo,
    #[serde(skip)]
    contains_pointer: bool,
}

impl PartialEq for DefaultPane {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl PaneBehavior for DefaultPane {
    fn ui(&mut self, ui: &mut egui::Ui, tile_id: egui_tiles::TileId) -> PaneResponse {
        let mut response = PaneResponse::default();

        let parent = vertically_centered(ui, &mut self.centering_memo, |ui| {
            ui.vertical_centered(|ui| {
                if ui.button("Vertical Split").clicked() {
                    response.set_action(PaneAction::SplitV);
                    debug!("Vertical Split button clicked");
                }
                if ui.button("Horizontal Split").clicked() {
                    response.set_action(PaneAction::SplitH);
                    debug!("Horizontal Split button clicked");
                }
                if ui.button("Widget Gallery").clicked() {
                    response.set_action(PaneAction::ReplaceThroughGallery(Some(tile_id)));
                }
            })
            .response
        });

        self.contains_pointer = parent.contains_pointer();

        if parent
            .interact(egui::Sense::click_and_drag())
            .on_hover_cursor(egui::CursorIcon::Grab)
            .dragged()
        {
            response.set_drag_started();
        };

        response
    }

    fn contains_pointer(&self) -> bool {
        self.contains_pointer
    }
}
