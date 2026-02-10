use egui::{Align, Color32, Context, Frame, Layout, Margin, Stroke, Vec2};
use segs_assets::icons;
use segs_ui::{
    UiComponentExt,
    containers::ResizablePanel,
    widgets::buttons::{BottomBarButton, UnpaddedBottomBarButton},
};

#[derive(Debug, Clone, Default)]
pub struct TopBarControls {
    pub lock_mode_active: bool,
    pub focus_mode_active: bool,
    pub panels_controls: PanelsControls,
}

pub fn top_controls_bar(ctx: &Context, controls: &mut TopBarControls) {
    let TopBarControls {
        lock_mode_active,
        focus_mode_active,
        panels_controls:
            PanelsControls {
                left_panel_visible,
                right_panel_visible,
                bottom_panel_visible,
            },
    } = controls;
    egui::TopBottomPanel::top("top_panel")
        .show_separator_line(false)
        .frame(
            Frame::new()
                .inner_margin(Margin::same(4))
                .fill(ctx.style().visuals.panel_fill),
        )
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Theme toggle button
                ui.theme_toggle_btn();

                // Panel toggle buttons
                ui.right_panel_toggle_btn(right_panel_visible);
                ui.bottom_panel_toggle_btn(bottom_panel_visible);
                ui.left_panel_toggle_btn(left_panel_visible);

                // Lock & Focus mode toggle button
                ui.focus_mode_toggle(focus_mode_active);
                ui.lock_mode_toggle(lock_mode_active);
            });
        });
}

#[derive(Debug, Clone, Default)]
pub struct BottomBarControls {
    pub notifications_active: bool,
}

pub fn bottom_controls_bar(ctx: &Context, controls: &mut BottomBarControls) {
    egui::TopBottomPanel::bottom("bottom_panel")
        .show_separator_line(true)
        .frame(Frame::new().fill(ctx.style().visuals.panel_fill))
        .show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let btn = UnpaddedBottomBarButton::default()
                        .add_space(10.)
                        .add_icon(icons::Pulse)
                        .add_space(5.);
                    ui.add(btn);

                    let btn = UnpaddedBottomBarButton::default()
                        .padded()
                        .add_icon(icons::Documents)
                        .add_text("Help");
                    ui.add(btn);

                    let btn = UnpaddedBottomBarButton::default()
                        .padded()
                        .add_icon(icons::Layout)
                        .add_text("Layouts");
                    ui.add(btn);

                    let btn = UnpaddedBottomBarButton::default()
                        .add_icon(icons::Arrow::up())
                        .add_space(2.0)
                        .add_icon(icons::Arrow::down())
                        .add_space(4.0)
                        .add_text("0.5/s");
                    ui.add(btn);
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
                    let res = ui.add(btn);
                    if res.clicked() {
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
                        .add_icon(icons::Antenna)
                        .add_text("Sources");
                    ui.add(btn);

                    let btn = UnpaddedBottomBarButton::default()
                        .padded()
                        .add_icon(icons::Lightning)
                        .add_text("Quick Commands");
                    ui.add(btn);
                });
            });
        });
}

#[derive(Debug, Clone, Default)]
pub struct PanelsControls {
    pub left_panel_visible: bool,
    pub right_panel_visible: bool,
    pub bottom_panel_visible: bool,
}

pub fn main_view(
    ctx: &Context,
    panel_controls: &mut PanelsControls,
    add_contents_left: impl FnOnce(&mut egui::Ui),
    add_contents_right: impl FnOnce(&mut egui::Ui),
    add_contents_bottom: impl FnOnce(&mut egui::Ui),
    add_contents_main: impl FnOnce(&mut egui::Ui),
) {
    let frame = Frame::new().fill(ctx.style().visuals.panel_fill);
    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
        // Define collapse state based on visibility
        let mut collapsed_left = !panel_controls.left_panel_visible;
        let mut collapsed_right = !panel_controls.right_panel_visible;
        let mut collapsed_bottom = !panel_controls.bottom_panel_visible;

        // Outer frames are for the hierarchical ResizablePanel structure, just fill color
        let panel_outer_frame = Frame::new().fill(Color32::from_rgb(246, 246, 246));
        // Inner ones use margin to create spacing between panels and content
        let panel_inner_frame = Frame::new().inner_margin(10.);
        let main_inner_frame = Frame::new()
            .corner_radius(5.0)
            .fill(Color32::from_rgb(252, 252, 252))
            .stroke(Stroke::new(1., Color32::from_rgb(242, 242, 242)));

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

        // Update visibility state based on collapses
        panel_controls.left_panel_visible = !collapsed_left;
        panel_controls.right_panel_visible = !collapsed_right;
        panel_controls.bottom_panel_visible = !collapsed_bottom;
    });
}
