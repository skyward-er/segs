use std::f32::consts::FRAC_PI_2;

use egui::{
    Align, Align2, Area, Color32, CornerRadius, Frame, Id, Layout, Rect, Response, Sense, Separator, Ui, UiBuilder,
    Vec2, vec2,
};
use segs_assets::{
    Font,
    fonts::Figtree,
    icons::{self, Icon},
};
use segs_memory::MemoryExt;
use segs_ui::{containers::ResizablePanel, style::CtxStyleExt};
use serde::{Deserialize, Serialize};

use crate::ui::components::buttons;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataflowEditorView;

impl super::ConfigurationViewTrait for DataflowEditorView {
    fn top_bar_left_fn(&mut self, ui: &mut Ui) {
        let id = Id::new("composition_left_panel_visible");
        let mut left_panel_visible: bool = ui.mem().get_perm_or_default(id);

        buttons::left_panel_toggle(ui, &mut left_panel_visible);

        ui.mem().insert_perm(id, left_panel_visible);
    }
}

fn dataflow(ctx: &egui::Context, left_panel_visible: &mut bool) {
    let app_style = ctx.app_style();
    let visuals = &ctx.style().visuals;
    let back_frame = Frame::new().fill(visuals.panel_fill);
    let mut cr = CornerRadius::same(5);
    // Only round the left corners
    cr.ne = 0;
    cr.se = 0;
    let front_frame = Frame::new()
        .corner_radius(cr)
        .fill(app_style.main_panels_fill)
        .stroke(app_style.main_view_stroke);
    egui::CentralPanel::default().frame(back_frame).show(ctx, |ui| {
        // Define collapse state based on visibility
        let mut collapsed_left = !*left_panel_visible;

        let visuals = ctx.app_style();
        let panel_outer_frame = Frame::new().corner_radius(5.).fill(visuals.main_panels_fill);
        let panel_inner_frame = Frame::NONE;
        let main_inner_frame = panel_inner_frame.corner_radius(5.).fill(visuals.main_panels_fill);

        let left = ResizablePanel::horizontal_left()
            .set_minimum_size(180.)
            .inactive_separator_stroke(visuals.main_view_stroke)
            .left_frame(panel_outer_frame)
            .collapsed(&mut collapsed_left);

        let layout = Layout::top_down(Align::Min);

        front_frame.show(ui, |ui| {
            left.show(ui, |panel| {
                panel
                    .show_left(|ui| {
                        panel_inner_frame.show(ui, |ui| {
                            ui.set_min_size(ui.available_size());
                            ui.set_clip_rect(ui.max_rect());
                            ui.with_layout(layout, dataflow_left);
                        });
                    })
                    .show_right(|ui| {
                        main_inner_frame.show(ui, |ui| {
                            ui.set_min_size(ui.available_size());
                            ui.with_layout(layout, dataflow_right);
                        });
                    });
            });
        });

        // Update visibility state based on collapses
        *left_panel_visible = !collapsed_left;
    });
}

fn dataflow_left(ui: &mut egui::Ui) {
    section_selector(ui);
    add_separator(ui);
    section_controls(ui);
    // add_separator(ui);
    // panel_content(ui);
}

fn dataflow_right(ui: &mut egui::Ui) {
    Frame::new().inner_margin(8.).show(ui, |ui| {
        ui.label("Dataflow main panel");
    });
}

fn add_separator(ui: &mut Ui) {
    ui.visuals_mut().widgets.noninteractive.bg_stroke = ui.ctx().app_style().main_view_stroke;
    ui.add(Separator::default().spacing(0.));
}

fn section_selector(ui: &mut Ui) -> Response {
    let size = vec2(ui.available_width(), 30.);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    let painter = ui.painter();

    let text = "Data Input Schemas";
    let text_color = ui.visuals().text_color();
    let galley = painter.layout_no_wrap(text.to_owned(), Figtree::medium().sized(14.), text_color);

    let mut cr = CornerRadius::ZERO;
    cr.nw = 5;
    let color = Color32::from_rgb(26, 26, 28);
    painter.rect_filled(rect, cr, color);

    let (icon_rect, text_rect) = rect.split_left_right_at_x(rect.left() + 20.);

    let id_toggled = ui.id().with("toggled");
    let mut toggled: bool = ui.mem().get_temp_or_default(id_toggled);

    if response.clicked() {
        toggled = !toggled;
        ui.mem().insert_temp(id_toggled, toggled);
    }

    if toggled {
        let id = ui.id().with("section_selector_area");
        Area::new(id)
            .pivot(Align2::LEFT_TOP)
            .fixed_pos(rect.left_bottom() + Vec2::splat(5.))
            .show(ui.ctx(), |ui| {
                let style = ui.style();
                Frame::new()
                    .corner_radius(style.visuals.menu_corner_radius)
                    .shadow(style.visuals.popup_shadow)
                    .fill(style.visuals.window_fill())
                    .stroke(style.visuals.window_stroke())
                    .show(ui, |ui| {
                        // Frame::new().inner_margin(vec2(10., 5.)).show(ui, |ui| {
                        //     asfas(ui);
                        // });
                        // ui.add(Separator::default().spacing(0.));
                        Frame::new().inner_margin(vec2(5., 5.)).show(ui, |ui| {
                            ui.set_min_width(rect.width() - 22.);
                            ui.spacing_mut().item_spacing = Vec2::splat(7.);
                            ui.label("Option 1");
                            ui.label("Option 2");
                            ui.label("Option 3");
                        });
                    })
            });
    }

    let id = ui.id().with("active_animation");
    let active_t = ui.ctx().animate_bool_with_time(id, toggled, 0.1);

    painter.galley(
        text_rect.left_center() - vec2(0., galley.size().y / 2.),
        galley,
        text_color,
    );

    let icon_rot = (1. - active_t) * -FRAC_PI_2;
    let icon = if toggled {
        icons::CaretDown::solid()
    } else {
        icons::CaretDown::outline()
    };
    let icon_rect = Rect::from_center_size(icon_rect.center(), vec2(10., 10.));
    let icon_color = ui.app_style().left_bar.icon_active_color;
    icon.to_image()
        .tint(icon_color)
        .fit_to_exact_size(icon_rect.size())
        .rotate(icon_rot, Vec2::splat(0.5))
        .paint_at(ui, icon_rect);

    response
}

fn section_controls(ui: &mut Ui) -> Response {
    let size = vec2(ui.available_width(), 25.);
    let (rect, response) = ui.allocate_exact_size(size, Sense::empty());

    if response.hovered() {
        let painter = ui.painter();
        let shadow_color = ui.app_style().left_bar.shadow_color_hover;
        painter.rect_filled(rect.shrink2(vec2(0., 3.)), 0., shadow_color);
    }

    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(4.);
            ui.add_space(5.);
            ribbon_control(ui);
            let pos = ui.cursor().left_center();
            let text_color = ui.app_style().left_bar.icon_inactive_color;
            ui.painter().text(
                pos,
                Align2::LEFT_CENTER,
                "Add repository...".to_owned(),
                Figtree::medium().sized(13.),
                text_color,
            )
        },
    );

    response
}

fn ribbon_control(ui: &mut Ui) -> Response {
    let size = vec2(20., 20.);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(17.));
    let icon_color = ui.app_style().left_bar.icon_inactive_color;
    icons::Cloud::plus()
        .to_image()
        .tint(icon_color)
        .fit_to_exact_size(icon_rect.size())
        .paint_at(ui, icon_rect);

    response
}
