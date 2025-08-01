use super::PaneBehavior;
use egui::Ui;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    mavlink::TimedMessage,
    ui::{
        app::{PaneAction, PaneResponse},
        utils::{SizingMemo, vertically_centered},
    },
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
    #[profiling::function]
    fn ui(&mut self, ui: &mut Ui) -> PaneResponse {
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
                    response.set_action(PaneAction::ReplaceThroughGallery);
                }
            })
            .response
        });

        self.contains_pointer = parent.contains_pointer();

        if parent.interact(egui::Sense::click_and_drag()).dragged() {
            response.set_drag_started();
        };

        response
    }

    fn update(&mut self, _messages: &[&TimedMessage]) {}

    fn get_message_subscriptions(&self) -> Box<dyn Iterator<Item = u32>> {
        Box::new(None.into_iter())
    }

    fn should_send_message_history(&self) -> bool {
        false
    }
}
