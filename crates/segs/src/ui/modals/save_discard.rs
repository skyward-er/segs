use egui::{Id, Tooltip, Ui};
use segs_memory::MemoryExt;
use segs_ui::containers::Modal;

use crate::layout::LayoutManager;

const MODAL_ID: &str = "layout_close_confirmation_modal";
const SAVE_ERROR_ID: &str = "layout_close_confirmation_save_error";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveDiscardModalChoice {
    Save,
    Discard,
}

pub struct SaveDiscardModalResponse {
    pub choice: Option<SaveDiscardModalChoice>,
    pub should_close: bool,
}

/// Modal used to resolve unsaved layout changes before closing the application.
pub struct SaveDiscardModal<'a> {
    layouts: &'a mut LayoutManager,
}

impl<'a> SaveDiscardModal<'a> {
    /// Creates a save/discard confirmation for the active layout.
    pub fn new(layouts: &'a mut LayoutManager) -> Self {
        Self { layouts }
    }

    /// Shows the confirmation and reports a successfully resolved choice.
    pub fn show(self, ui: &mut Ui) -> SaveDiscardModalResponse {
        let Self { layouts } = self;
        let mut choice = None;
        let mut save_error: Option<String> = ui.mem().get_temp(Id::new(SAVE_ERROR_ID));
        let response = Modal::new(Id::new(MODAL_ID), "Unsaved Layout Changes").show(ui.ctx(), |ui| {
            ui.set_max_width(420.);
            ui.label("Save or discard the active layout changes before closing?");
            ui.add_space(10.);
            ui.horizontal(|ui| {
                if ui.button("Discard").clicked() {
                    choice = Some(SaveDiscardModalChoice::Discard);
                }

                let save = ui.button("Save");
                if save.clicked() {
                    match layouts.save_active() {
                        Ok(()) => {
                            save_error = None;
                            choice = Some(SaveDiscardModalChoice::Save);
                        }
                        Err(error) => save_error = Some(error.to_string()),
                    }
                }
                if let Some(error) = &save_error {
                    Tooltip::always_open(ui.ctx().clone(), save.layer_id, save.id.with("save_error"), save.rect).show(
                        |ui| {
                            ui.colored_label(ui.visuals().error_fg_color, error);
                        },
                    );
                }
            });
        });

        let should_close = response.should_close();
        if choice.is_some() || should_close {
            ui.mem().remove_temp::<String>(Id::new(SAVE_ERROR_ID));
        } else if let Some(error) = save_error {
            ui.mem().insert_temp(Id::new(SAVE_ERROR_ID), error);
        }

        SaveDiscardModalResponse { choice, should_close }
    }
}
