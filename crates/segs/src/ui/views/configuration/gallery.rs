use egui::Ui;

pub fn show(ui: &mut Ui) {
    for i in 0..40 {
        ui.label(format!("Widget {i}"));
        ui.add_space(2.);
    }
}
