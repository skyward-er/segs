use egui::{Align2, Id, Pos2, Tooltip, Ui};

use super::Popup;

const POPUP_ID: &str = "delete_confirmation_popup";

/// Popup used to confirm permanent layout deletion.
pub struct DeleteConfirmationPopup<'a> {
    enabled: &'a mut bool,
    pivot_pos: Pos2,
    pivot_align: Align2,
    error: Option<&'a str>,
}

impl<'a> DeleteConfirmationPopup<'a> {
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

    /// Associates a persistence failure with the Delete button.
    pub fn error(mut self, error: Option<&'a str>) -> Self {
        self.error = error;
        self
    }

    /// Shows the popup and returns whether deletion was confirmed.
    pub fn show(self, ui: &mut Ui) -> bool {
        let Self {
            enabled,
            pivot_pos,
            pivot_align,
            error,
        } = self;
        let mut confirmed = false;
        Popup::new(enabled, pivot_pos)
            .id(Id::new(POPUP_ID))
            .pivot(pivot_align)
            .show(ui, |ui| {
                ui.label("Confirm layout deletion?");
                ui.horizontal(|ui| {
                    let delete = ui.button("Delete");
                    if delete.clicked() {
                        confirmed = true;
                        ui.close();
                    }
                    if let Some(error) = error {
                        Tooltip::always_open(
                            ui.ctx().clone(),
                            delete.layer_id,
                            delete.id.with("delete_error"),
                            delete.rect,
                        )
                        .show(|ui| {
                            ui.colored_label(ui.visuals().error_fg_color, error);
                        });
                    }
                });
            });
        confirmed
    }
}
