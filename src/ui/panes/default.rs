use super::{plot::Plot2DPane, Pane, PaneBehavior, PaneKind};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::ui::{
    composable_view::{PaneAction, PaneResponse},
    utils::{vertically_centered, SizingMemo},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DefaultPane {
    occupied: f32,
    fixed: bool,
    #[serde(skip)]
    centering_memo: SizingMemo,
    #[serde(skip)]
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

impl PartialEq for DefaultPane {
    fn eq(&self, other: &Self) -> bool {
        self.occupied == other.occupied && self.fixed == other.fixed
    }
}

impl PaneBehavior for DefaultPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
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
                if ui.button("Plot").clicked() {
                    response.set_action(PaneAction::Replace(Pane::boxed(PaneKind::Plot2D(
                        Plot2DPane::new(ui.auto_id_with("plot_2d")),
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
