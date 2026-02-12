use egui::{Context, Response};

pub fn pointer_clicked_outside(ctx: &Context, response: &Response) -> bool {
    // If the pointer clicked this frame, but NOT on our area
    if ctx.input(|i| i.pointer.any_click()) && !response.clicked_by(egui::PointerButton::Primary) {
        // Additionally check if the pointer is actually outside the rect
        if let Some(pos) = ctx.pointer_interact_pos()
            && !response.rect.contains(pos)
        {
            return true;
        }
    }
    false
}
