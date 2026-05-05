mod adapter_config;

pub use adapter_config::ADAPTER_CONFIG_MODAL_ID;
pub use adapter_config::AdapterConfigModal;

use egui::{Id, Ui};

/// A simple reusable modal overlay component.
pub struct Modal<'a> {
    enabled: &'a mut bool,
    id: Option<Id>,
}

impl<'a> Modal<'a> {
    pub fn new(enabled: &'a mut bool) -> Self {
        Self { enabled, id: None }
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Show the modal. `add_contents` draws the user-provided content inside the modal body.
    pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
        if !*self.enabled {
            return;
        }

        let id = self.id.unwrap_or_else(|| ui.id().with("_modal"));
        let modal_response = egui::Modal::new(id).show(ui.ctx(), |ui| add_contents(ui));
        if modal_response.should_close() {
            *self.enabled = false;
        }
    }
}
