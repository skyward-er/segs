use egui::{Align, Frame, Layout, Margin, Panel, Ui, Vec2};
use segs_ui::style::CtxStyleExt;

use crate::ui::components::buttons;

pub fn show(ui: &mut Ui) {
    let stroke = ui.app_style().main_view_stroke;
    let response = Panel::top("top_panel")
        .show_separator_line(false)
        .frame(
            Frame::new()
                .inner_margin(Margin::symmetric(4, 3))
                .fill(ui.style().visuals.panel_fill),
        )
        .show_inside(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                let width = ui.max_rect().width();
                let window_controls_width = 75.;
                let middle_width = 300.;
                let right_side_width = (width - middle_width) / 2.;
                let side_width = right_side_width - window_controls_width;

                ui.add_space(window_controls_width);

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.set_min_width(side_width);
                });

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.set_width(middle_width);
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.set_min_width(side_width);
                    ui.add_space(3.);

                    // Theme toggle button
                    buttons::theme_toggle(ui);
                });
            });
        });

    let rect = response.response.rect;
    let y = rect.bottom() - stroke.width * 0.5;
    ui.painter().hline(rect.x_range(), y, stroke);
}
