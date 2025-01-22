use super::PaneBehavior;
use egui::{Align, Align2, Area, Color32, Frame, Label, Layout, Order, UiKind};
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

                let response = ui.button("Open popup");
                let popup_id = ui.make_persistent_id("my_unique_id");
                if response.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                }

                if ui.memory(|mem| mem.is_popup_open(popup_id)) {
                    let frame = Frame::popup(ui.style()).fill(Color32::DARK_RED);

                    let mut pos = ui.ctx().screen_rect().right_top();
                    pos.x -= 15.0;
                    pos.y += 15.0;
                    let align = Align2::RIGHT_TOP;

                    let response = Area::new(popup_id)
                        .kind(UiKind::Popup)
                        .order(Order::Foreground)
                        .fixed_pos(pos)
                        .default_width(100.0)
                        .pivot(align)
                        .show(ui.ctx(), |ui| {
                            frame
                                .show(ui, |ui| {
                                    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                                        ui.set_min_width(100.0);
                                        ui.style_mut().visuals.override_text_color =
                                            Some(Color32::WHITE);
                                        ui.label("Popup content");
                                    })
                                    .inner
                                })
                                .inner
                        });
                }

                // let popup_id = ui.make_persistent_id("my_unique_id");
                // if response.clicked() {
                //     ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                // }
                // let below = egui::AboveOrBelow::Above;
                // let close_on_click_outside = egui::popup::PopupCloseBehavior::IgnoreClicks;
                // egui::popup::popup_above_or_below_widget(
                //     ui,
                //     popup_id,
                //     &response,
                //     below,
                //     close_on_click_outside,
                //     |ui| {
                //         ui.set_min_width(200.0); // if you want to control the size
                //         ui.label("Some more info, or things you can select:");
                //         ui.label("…");
                //     },
                // );
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
