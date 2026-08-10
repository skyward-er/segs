#![allow(unused)]

mod delete_confirmation;
mod grid_settings;
mod save_discard_confirmation;

use egui::{Align2, Area, Frame, Id, Key, Modifiers, Order, Pos2, Ui, UiBuilder, UiKind, Vec2, emath::easing, vec2};

pub use delete_confirmation::DeleteConfirmationPopup;
pub use grid_settings::GridSettingsPopup;
pub use save_discard_confirmation::{SaveDiscardChoice, SaveDiscardConfirmationPopup};

const POPUP_MARGIN: Vec2 = vec2(8., 8.);

pub struct Popup<'a> {
    enabled: &'a mut bool,
    id: Option<Id>,
    pivot_pos: Pos2,
    pivot_align: Align2,
    force_sizing_pass: bool,
}

impl<'a> Popup<'a> {
    pub fn new(enabled: &'a mut bool, pos: Pos2) -> Self {
        Self {
            enabled,
            id: None,
            pivot_pos: pos,
            pivot_align: Align2::LEFT_TOP,
            force_sizing_pass: false,
        }
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn pivot(mut self, align: Align2) -> Self {
        self.pivot_align = align;
        self
    }

    pub fn force_sizing_pass(mut self) -> Self {
        self.force_sizing_pass = true;
        self
    }

    pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
        let mut builder = UiBuilder::new();
        if let Some(id) = self.id {
            builder = builder.id(id)
        }
        ui.scope_builder(builder, |ui| self.show_inner(ui, add_contents));
    }

    fn show_inner(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
        let Self {
            enabled,
            pivot_pos,
            pivot_align,
            force_sizing_pass,
            ..
        } = self;

        let popup_id = ui.id().with("_popup");
        let area_id = ui.id().with("_area");

        // Mirror egui popup memory so modals know a popup owns Escape and pointer input
        if *enabled {
            egui::Popup::open_id(ui.ctx(), popup_id);
        } else if egui::Popup::is_id_open(ui.ctx(), popup_id) {
            egui::Popup::close_id(ui.ctx(), popup_id);
        }

        if *enabled && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape)) {
            *enabled = false;
        }

        let id = ui.id().with("_anim_visible");
        let visible_t = ui
            .ctx()
            .animate_bool_with_time_and_easing(id, *enabled, 0.2, easing::cubic_out);

        let pivot = pivot_pos + visible_t * get_offset_from_align(pivot_align);
        if visible_t > 0.3 {
            // Ignore an opening click until the popup area existed in a previous frame
            let was_open_last_frame = ui.ctx().read_response(area_id).is_some();

            let source_toggled_t = (visible_t - 0.2) / 0.8;
            let style = ui.style();
            let res = Area::new(area_id)
                .kind(UiKind::Popup)
                .order(Order::Foreground)
                .pivot(pivot_align)
                .fixed_pos(pivot)
                .sizing_pass(force_sizing_pass)
                .show(ui.ctx(), |ui| {
                    ui.set_opacity(source_toggled_t);
                    Frame::new()
                        .inner_margin(POPUP_MARGIN)
                        .corner_radius(style.visuals.menu_corner_radius)
                        .shadow(style.visuals.popup_shadow)
                        .fill(style.visuals.window_fill())
                        .stroke(style.visuals.window_stroke())
                        .show(ui, |ui| {
                            add_contents(ui);
                        });
                })
                .response;

            // After a sizing pass, request a discard to avoid showing a frame without the
            // open popup contents
            if force_sizing_pass {
                ui.ctx().request_discard("record popup size after forced sizing pass");
            }
            let pointer_pressed_elsewhere = ui.input(|input| input.pointer.any_pressed())
                && ui
                    .ctx()
                    .pointer_interact_pos()
                    .is_some_and(|pointer| !res.rect.contains(pointer));

            // Hide only after the popup survived its opening frame
            if should_close_popup(was_open_last_frame, res.should_close(), pointer_pressed_elsewhere) {
                *enabled = false;
            }
        }

        if !*enabled && egui::Popup::is_id_open(ui.ctx(), popup_id) {
            egui::Popup::close_id(ui.ctx(), popup_id);
        }
    }
}

/// Returns whether popup interaction should close the popup this frame.
fn should_close_popup(was_open_last_frame: bool, close_requested: bool, pointer_pressed_elsewhere: bool) -> bool {
    close_requested || (was_open_last_frame && pointer_pressed_elsewhere)
}

const AXIS_OFFSET: f32 = 7.;

fn get_offset_from_align(align: Align2) -> Vec2 {
    let (x, y) = match align {
        Align2::LEFT_TOP => (AXIS_OFFSET, AXIS_OFFSET),
        Align2::CENTER_TOP => (0., AXIS_OFFSET),
        Align2::RIGHT_TOP => (-AXIS_OFFSET, AXIS_OFFSET),
        Align2::LEFT_CENTER => (AXIS_OFFSET, 0.),
        Align2::CENTER_CENTER => (0., 0.),
        Align2::RIGHT_CENTER => (-AXIS_OFFSET, 0.),
        Align2::LEFT_BOTTOM => (AXIS_OFFSET, -AXIS_OFFSET),
        Align2::CENTER_BOTTOM => (0., -AXIS_OFFSET),
        Align2::RIGHT_BOTTOM => (-AXIS_OFFSET, -AXIS_OFFSET),
    };
    vec2(x, y)
}

#[cfg(test)]
mod tests {
    use super::should_close_popup;

    #[test]
    fn opening_pointer_press_does_not_immediately_close_popup() {
        assert!(!should_close_popup(false, false, true));
    }

    #[test]
    fn outside_pointer_press_closes_established_popup() {
        assert!(should_close_popup(true, false, true));
    }

    #[test]
    fn inside_pointer_press_keeps_popup_open() {
        assert!(!should_close_popup(true, false, false));
    }

    #[test]
    fn explicit_close_request_closes_popup() {
        assert!(should_close_popup(false, true, false));
    }
}
