use egui::containers::Frame;
use egui::{Response, Shadow, Stroke, Style, Ui};
use egui_tiles::TileId;

use super::panes::{Pane, PaneBehavior};

/// This function wraps a ui into a popup frame intended for the pane that needs
/// to be maximized on screen.
pub fn maximized_pane_ui(ui: &mut Ui, tile_id: TileId, pane: &mut Pane) {
    Frame::popup(&Style::default())
        .fill(egui::Color32::TRANSPARENT)
        .shadow(Shadow::NONE)
        .stroke(Stroke::NONE)
        .show(ui, |ui| pane.ui(ui, tile_id));
}

#[derive(Debug, Default, Clone)]
pub struct SizingMemo {
    occupied_height: f32,
    sizing_pass_done: bool,
}

pub fn vertically_centered(
    ui: &mut Ui,
    memo: &mut SizingMemo,
    add_contents: impl FnOnce(&mut Ui) -> Response,
) -> egui::Response {
    if !memo.sizing_pass_done {
        let r = add_contents(ui);
        memo.occupied_height = r.rect.height();
        memo.sizing_pass_done = true;
        ui.ctx()
            .request_discard("horizontally_centered requires a sizing pass");
        r
    } else {
        let spacing = (ui.available_height() - memo.occupied_height) / 2.0;
        ui.vertical_centered(|ui| {
            ui.add_space(spacing);
            add_contents(ui);
            ui.add_space(spacing);
        })
        .response
    }
}
