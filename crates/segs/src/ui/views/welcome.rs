use egui::{Align, CentralPanel, Label, Layout, RichText};

use crate::{
    app::AppContext,
    ui::{layout, views::ViewTrait},
};

#[derive(Default)]
pub struct WelcomeView;

impl ViewTrait for WelcomeView {
    fn show_main_view(&mut self, ui: &mut egui::Ui, appctx: &mut AppContext) {
        CentralPanel::default().show_inside(ui, |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space((ui.available_height() * 0.35).max(32.));
                ui.add(Label::new(RichText::new("No layout selected").size(24.)).selectable(false));
                ui.add(Label::new("Create a layout or select an existing one to get started.").selectable(false));
                ui.add_space(12.);
                if ui.button("Open Layout Manager").clicked() {
                    layout::request_open_manager(ui, &appctx.layouts);
                }
            });
        });
    }
}
