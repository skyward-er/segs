mod adapter_status;
pub use adapter_status::ConnectionPopup;

use egui::{Align2, Area, Frame, Id, Pos2, Ui, UiBuilder, Vec2, emath::easing, vec2};

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

        let id = ui.id().with("_anim_visible");
        let visible_t = ui
            .ctx()
            .animate_bool_with_time_and_easing(id, *enabled, 0.2, easing::cubic_out);

        let pivot = pivot_pos + visible_t * get_offset_from_align(pivot_align);
        if visible_t > 0.3 {
            let id = ui.id().with("_area");

            let source_toggled_t = (visible_t - 0.2) / 0.8;
            let style = ui.style();
            let res = Area::new(id)
                .pivot(pivot_align)
                .fixed_pos(pivot)
                .sizing_pass(force_sizing_pass)
                .show(ui.ctx(), |ui| {
                    ui.set_opacity(source_toggled_t);
                    Frame::new()
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

            // Hide the popup if the user clicks outside of it
            if res.clicked_elsewhere() {
                *enabled = false;
            }
        }
    }
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
