use super::{plot_2d::Plot2DPane, Pane, PaneBehavior, PaneKind};
use serde::{Deserialize, Serialize};

use crate::ui::{
    composable_view::{PaneAction, PaneResponse},
    utils::{vertically_centered, SizingMemo},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DefaultPane {
    #[serde(skip)]
    centering_memo: SizingMemo,
    contains_pointer: bool,
}

impl Default for DefaultPane {
    fn default() -> Self {
        DefaultPane {
            centering_memo: SizingMemo::default(),
            contains_pointer: false,
        }
    }
}

impl PaneBehavior for DefaultPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let mut response = PaneResponse::default();

        let parent = vertically_centered(ui, &mut self.centering_memo, |ui| {
            ui.vertical_centered(|ui| {
                if ui.button("Vertical Split").clicked() {
                    response.set_action(PaneAction::SplitV);
                    log::debug!("Vertical Split button clicked");
                }
                if ui.button("Horizontal Split").clicked() {
                    response.set_action(PaneAction::SplitH);
                    log::debug!("Horizontal Split button clicked");
                }
                if ui.button("Plot").clicked() {
                    response.set_action(PaneAction::Replace(Pane::boxed(PaneKind::Plot2D(
                        Plot2DPane::default(),
                    ))));
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