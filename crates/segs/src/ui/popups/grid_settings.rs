use egui::{Align2, DragValue, Id, Pos2, Ui};

use crate::ui::grid::GridSettings;

use super::Popup;

const POPUP_ID: &str = "grid_settings_popup";
const MIN_GRID_VALUE: u32 = 1;
const MAX_GRID_VALUE: u32 = 100;

/// Popup controls for configuring the layout grid.
pub struct GridSettingsPopup<'a> {
    enabled: &'a mut bool,
    settings: &'a mut GridSettings,
    pivot_pos: Pos2,
}

impl<'a> GridSettingsPopup<'a> {
    /// Creates a popup anchored at `pivot_pos`.
    pub fn new(enabled: &'a mut bool, settings: &'a mut GridSettings, pivot_pos: Pos2) -> Self {
        Self {
            enabled,
            settings,
            pivot_pos,
        }
    }

    /// Shows the popup and applies valid edits directly to the grid settings.
    pub fn show(self, ui: &mut Ui) {
        let Self {
            enabled,
            settings,
            pivot_pos,
        } = self;

        Popup::new(enabled, pivot_pos)
            .id(Id::new(POPUP_ID))
            .pivot(Align2::RIGHT_TOP)
            .show(ui, |ui| show_contents(ui, settings));
    }
}

fn show_contents(ui: &mut Ui, settings: &mut GridSettings) {
    clamp_settings(settings);
    egui::Grid::new("grid_settings_values")
        .num_columns(2)
        .spacing([8., 4.])
        .show(ui, |ui| {
            ui.label("Columns");
            ui.add(DragValue::new(&mut settings.cols).range(MIN_GRID_VALUE..=MAX_GRID_VALUE));
            ui.end_row();

            ui.label("Rows");
            ui.add(DragValue::new(&mut settings.rows).range(MIN_GRID_VALUE..=MAX_GRID_VALUE));
            ui.end_row();
        });
    clamp_settings(settings);
}

fn clamp_count(value: u32) -> u32 {
    value.clamp(MIN_GRID_VALUE, MAX_GRID_VALUE)
}

fn clamp_settings(settings: &mut GridSettings) {
    settings.cols = clamp_count(settings.cols);
    settings.rows = clamp_count(settings.rows);
}
