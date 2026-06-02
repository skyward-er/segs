use egui::Ui;

use crate::{app::AppContext, ui::widgets::WidgetTrait};

pub struct ValueDisplayWidget {
    pub value: String,
}

impl WidgetTrait for ValueDisplayWidget {
    fn show(&self, ui: &mut Ui, _appctx: &mut AppContext) {
        ui.centered_and_justified(|ui| {
            ui.label(&self.value);
        });
    }
}
