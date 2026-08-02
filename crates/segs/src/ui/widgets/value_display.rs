use egui::{Label, Sense, Ui};

use crate::{dataflow::DataStore, ui::widgets::WidgetTrait};

#[derive(Clone)]
pub struct ValueDisplayWidget {
    pub value: String,
}

impl Default for ValueDisplayWidget {
    /// Creates the gallery's default value display.
    fn default() -> Self {
        Self {
            value: "123.456".to_owned(),
        }
    }
}

impl WidgetTrait for ValueDisplayWidget {
    fn show(&self, ui: &mut Ui, _data_store: &mut DataStore) {
        ui.centered_and_justified(|ui| {
            ui.add(Label::new(&self.value).sense(Sense::empty()));
        });
    }

    /// Returns the widget's gallery name.
    fn display_name(&self) -> &'static str {
        "Value display"
    }
}
