use egui::{Response, Ui};

#[derive(Debug, Default, Clone)]
pub struct SizingMemo {
    occupied_height: f32,
    sizing_pass_done: bool,
}

pub fn vertically_centered(
    ui: &mut Ui,
    memo: &mut SizingMemo,
    add_contents: impl FnOnce(&mut Ui) -> Response,
) -> egui::Response {
    if !memo.sizing_pass_done {
        let r = add_contents(ui);
        memo.occupied_height = r.rect.height();
        memo.sizing_pass_done = true;
        ui.ctx()
            .request_discard("horizontally_centered requires a sizing pass");
        r
    } else {
        let spacing = (ui.available_height() - memo.occupied_height) / 2.0;
        ui.vertical_centered(|ui| {
            ui.add_space(spacing);
            add_contents(ui);
            ui.add_space(spacing);
        })
        .response
    }
}
