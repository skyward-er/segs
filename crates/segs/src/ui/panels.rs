use egui::{Align, Context, Frame, Layout, Ui};
use segs_ui::{containers::ResizablePanel, style::CtxStyleExt};

/* pub fn bottom_controls_bar(ctx: &Context, controls: &mut BottomBarControls) {
    egui::TopBottomPanel::bottom("bottom_panel")
        .show_separator_line(false)
        .frame(Frame::new().fill(ctx.style().visuals.panel_fill))
        .show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let temp_id = ui.id().with("bottom_bar_sources_toggle");
                    let mut source_toggled: bool = ui.mem().get_temp_or_default(temp_id);

                    let icon = if source_toggled {
                        icons::Antenna::solid()
                    } else {
                        icons::Antenna::outline()
                    };
                    let btn = UnpaddedBottomBarButton::default()
                        .add_space(10.)
                        .add_icon(icon)
                        .add_space(5.)
                        .add_text("Sources")
                        .add_space(5.);
                    let res = ui.add(btn);

                    let res = res.on_hover_cursor(egui::CursorIcon::PointingHand);
                    if res.clicked() {
                        source_toggled = !source_toggled;
                    }

                    popups::ConnectionPopup::new(&mut source_toggled, res.rect.left_top(), Align2::LEFT_BOTTOM)
                        .show(ui);

                    ui.mem().insert_temp(temp_id, source_toggled);
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(2., 0.);
                    let icon = if controls.notifications_active {
                        icons::Bell::solid()
                    } else {
                        icons::Bell::outline()
                    };
                    let btn = UnpaddedBottomBarButton::default()
                        .add_space(5.)
                        .add_icon(icon)
                        .add_space(10.);
                    if ui.add(btn).clicked() {
                        controls.notifications_active = !controls.notifications_active;
                    }

                    let btn = UnpaddedBottomBarButton::default()
                        .padded()
                        .add_icon(icons::Error)
                        .add_text("0")
                        .add_icon(icons::Warning)
                        .add_text("0");
                    ui.add(btn);

                    let btn = UnpaddedBottomBarButton::default()
                        .padded()
                        .add_icon(icons::Lightning)
                        .add_text("Quick Commands");
                    ui.add(btn);
                });
            });
        });
} */

/* pub fn main_view(
    ctx: &Context,
    panel_controls: &mut PanelsControls,
    add_contents_left: impl FnOnce(&mut Ui),
    add_contents_right: impl FnOnce(&mut Ui),
    add_contents_bottom: impl FnOnce(&mut Ui),
    add_contents_main: impl FnOnce(&mut Ui),
) {
    let app_style = ctx.app_style();
    let visuals = &ctx.style().visuals;
    let back_frame = Frame::new().fill(visuals.panel_fill);
    let front_frame = Frame::new().corner_radius(5.).fill(app_style.main_panels_fill);
    egui::CentralPanel::default().frame(back_frame).show(ctx, |ui| {
        // Define collapse state based on visibility
        let mut collapsed_left = !panel_controls.left_panel_visible;
        let mut collapsed_right = !panel_controls.right_panel_visible;
        let mut collapsed_bottom = !panel_controls.bottom_panel_visible;

        let visuals = ctx.app_style();
        // Outer frames are for the hierarchical ResizablePanel structure, just fill
        // color
        let panel_outer_frame = Frame::new().corner_radius(5.).fill(visuals.main_panels_fill);
        // Inner ones use margin to create spacing between panels and content
        let panel_inner_frame = Frame::new().inner_margin(10.);
        // let panel_inner_frame = Frame::NONE;
        let main_inner_frame = panel_inner_frame
            .corner_radius(5.)
            .fill(visuals.main_view_fill)
            .stroke(visuals.main_view_stroke);

        let left = ResizablePanel::horizontal_left()
            .collapsed(&mut collapsed_left)
            .inactive_separator_width(0.)
            .left_frame(panel_outer_frame);
        let right = ResizablePanel::horizontal_right()
            .collapsed(&mut collapsed_right)
            .inactive_separator_width(0.)
            .right_frame(panel_outer_frame);
        let bottom = ResizablePanel::vertical_bottom()
            .collapsed(&mut collapsed_bottom)
            .inactive_separator_width(0.)
            .top_frame(panel_outer_frame)
            .bottom_frame(panel_outer_frame);

        let layout = Layout::top_down(Align::Min);

        front_frame.show(ui, |ui| {
            left.show(ui, |panel| {
                panel
                    .show_left(|ui| {
                        panel_inner_frame.show(ui, |ui| {
                            ui.set_min_size(ui.available_size());
                            ui.with_layout(layout, add_contents_left);
                        });
                    })
                    .show_right(|ui| {
                        right.show(ui, |panel| {
                            panel
                                .show_right(|ui| {
                                    panel_inner_frame.show(ui, |ui| {
                                        ui.set_min_size(ui.available_size());
                                        ui.with_layout(layout, add_contents_right);
                                    });
                                })
                                .show_left(|ui| {
                                    bottom.show(ui, |panel| {
                                        panel
                                            .show_bottom(|ui| {
                                                panel_inner_frame.show(ui, |ui| {
                                                    ui.set_min_size(ui.available_size());
                                                    ui.with_layout(layout, add_contents_bottom);
                                                });
                                            })
                                            .show_top(|ui| {
                                                main_inner_frame.show(ui, |ui| {
                                                    ui.set_min_size(ui.available_size());
                                                    ui.with_layout(layout, add_contents_main);
                                                });
                                            });
                                    });
                                });
                        });
                    });
            });
        });

        // Update visibility state based on collapses
        panel_controls.left_panel_visible = !collapsed_left;
        panel_controls.right_panel_visible = !collapsed_right;
        panel_controls.bottom_panel_visible = !collapsed_bottom;
    });
}
 */
