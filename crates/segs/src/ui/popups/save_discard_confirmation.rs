use egui::{Align2, Id, Pos2, Tooltip, Ui};

use super::Popup;

const POPUP_ID: &str = "save_discard_confirmation_popup";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveDiscardChoice {
    Save,
    Discard,
}

/// Popup used to resolve unsaved layout changes before another action.
pub struct SaveDiscardConfirmationPopup<'a> {
    enabled: &'a mut bool,
    pivot_pos: Pos2,
    pivot_align: Align2,
    error: Option<&'a str>,
}

impl<'a> SaveDiscardConfirmationPopup<'a> {
    /// Creates a confirmation popup anchored at `pivot_pos`.
    pub fn new(enabled: &'a mut bool, pivot_pos: Pos2) -> Self {
        Self {
            enabled,
            pivot_pos,
            pivot_align: Align2::LEFT_TOP,
            error: None,
        }
    }

    /// Sets which popup corner is anchored to the pivot position.
    pub fn pivot(mut self, align: Align2) -> Self {
        self.pivot_align = align;
        self
    }

    /// Associates a save failure with the Save button.
    pub fn error(mut self, error: Option<&'a str>) -> Self {
        self.error = error;
        self
    }

    /// Shows the popup and returns the selected resolution.
    pub fn show(self, ui: &mut Ui) -> Option<SaveDiscardChoice> {
        let Self {
            enabled,
            pivot_pos,
            pivot_align,
            error,
        } = self;
        let mut choice = None;
        Popup::new(enabled, pivot_pos)
            .id(Id::new(POPUP_ID))
            .pivot(pivot_align)
            .show(ui, |ui| {
                ui.label("Save or discard the active layout changes?");
                ui.horizontal(|ui| {
                    if ui.button("Discard").clicked() {
                        choice = Some(SaveDiscardChoice::Discard);
                        ui.close();
                    }
                    let save = ui.button("Save");
                    if save.clicked() {
                        choice = Some(SaveDiscardChoice::Save);
                        ui.close();
                    }
                    if let Some(error) = error {
                        Tooltip::always_open(ui.ctx().clone(), save.layer_id, save.id.with("save_error"), save.rect)
                            .show(|ui| {
                                ui.colored_label(ui.visuals().error_fg_color, error);
                            });
                    }
                });
            });
        choice
    }
}
