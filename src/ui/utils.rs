use egui::containers::Frame;
use egui::{Shadow, Style, Ui};

use super::panes::{Pane, PaneBehavior};

/// This function wraps a ui into a popup frame intended for the pane that needs
/// to be maximized on screen.
pub fn maximized_pane_ui(ui: &mut Ui, pane: &mut Pane) {
    Frame::popup(&Style::default())
        .fill(egui::Color32::TRANSPARENT)
        .shadow(Shadow::NONE)
        .show(ui, |ui| pane.ui(ui));
}
