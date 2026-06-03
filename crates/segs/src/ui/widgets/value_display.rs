use egui::Ui;

use crate::{dataflow::DataStore, ui::widgets::WidgetTrait};

pub struct ValueDisplayWidget {
    pub value: String,
}

impl WidgetTrait for ValueDisplayWidget {
    fn show(&self, ui: &mut Ui, _data_store: &mut DataStore) {
        ui.centered_and_justified(|ui| {
            ui.label(&self.value);
        });
    }
}
