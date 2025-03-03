use egui::Context;
use egui_tiles::TileId;
use strum::{EnumMessage, IntoEnumIterator};

use super::{
    app::PaneAction,
    panes::{Pane, PaneKind},
};

#[derive(Default)]
pub struct WidgetGallery {
    pub open: bool,
    tile_id: Option<TileId>,
}

impl WidgetGallery {
    pub fn replace_tile(&mut self, tile_id: TileId) {
        self.tile_id = Some(tile_id);
        self.open = true;
    }

    pub fn show(&mut self, ctx: &Context) -> Option<PaneAction> {
        let mut window_visible = self.open;
        let resp = egui::Window::new("Widget Gallery")
            .collapsible(false)
            .open(&mut window_visible)
            .show(ctx, |ui| {
                for pane in PaneKind::iter() {
                    if let PaneKind::Default(_) = pane {
                        continue;
                    } else if let Some(message) = pane.get_message() {
                        if ui.button(message).clicked() {
                            if let Some(tile_id) = self.tile_id {
                                return Some(PaneAction::Replace(tile_id, Pane::boxed(pane)));
                            }
                        }
                    }
                }
                None
            });
        self.open = window_visible;

        let action = resp.and_then(|resp| resp.inner).flatten();

        // If an action was taken, always close the window
        if action.is_some() {
            self.open = false;
        }

        action
    }
}
