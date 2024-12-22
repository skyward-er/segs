use egui::{Context, Id};
use egui_tiles::TileId;

use super::{
    composable_view::PaneAction,
    panes::{plot::Plot2DPane, Pane, PaneKind},
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
                if ui.button("Plot").clicked() {
                    if let Some(tile_id) = self.tile_id {
                        Some(PaneAction::Replace(
                            tile_id,
                            Pane::boxed(PaneKind::Plot2D(Plot2DPane::new(Id::new("plot")))),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
        self.open = window_visible;

        let action = resp.map(|resp| resp.inner.flatten()).flatten();

        // If an action was taken, always close the window
        if action.is_some() {
            self.open = false;
        }

        action
    }
}
