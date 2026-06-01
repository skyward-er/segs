use egui::{Align, Frame, Id, Layout, Margin, Panel, Ui, Vec2};
use segs_memory::MemoryExt;

use crate::ui::components::{
    buttons,
    mode_toggle::{Mode, ModeToggle},
};

pub fn show(ui: &mut Ui) {
    Panel::top("top_panel")
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
                    top_bar_left_fn(ui);
                });

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.set_width(middle_width);
                    top_bar_middle_fn(ui);
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.set_min_width(side_width);
                    ui.add_space(3.);

                    // Theme toggle button
                    buttons::theme_toggle(ui);
                });
            });
        });
}

fn top_bar_left_fn(ui: &mut Ui) {
    let id = Id::new("left_panel_visible");
    let mut left_panel_visible: bool = ui.mem().get_perm_or_default(id);

    buttons::left_panel_toggle(ui, &mut left_panel_visible);

    ui.mem().insert_perm(id, left_panel_visible);
}

fn top_bar_middle_fn(ui: &mut Ui) {
    let id = Id::new("current_mode");
    let mut mode: Mode = ui.mem().get_temp_or_default(id);

    ModeToggle::new(&mut mode).with_height(22.).with_width(300.).show(ui);

    ui.mem().insert_temp(id, mode);
}
