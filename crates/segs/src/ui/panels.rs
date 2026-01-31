use egui::{Align, Context, Frame, Layout, Margin, Vec2};
use segs_assets::icons;
use segs_ui::{
    UiComponentExt,
    widgets::buttons::{BottomBarButton, UnpaddedBottomBarButton},
};

#[derive(Debug, Clone, Default)]
pub struct TopBarControls {
    pub lock_mode_active: bool,
    pub focus_mode_active: bool,
    pub left_panel_visible: bool,
    pub right_panel_visible: bool,
    pub bottom_panel_visible: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BottomBarControls {
    pub notifications_active: bool,
}

pub fn top_controls_bar(ctx: &Context, controls: &mut TopBarControls) {
    let TopBarControls {
        lock_mode_active,
        focus_mode_active,
        left_panel_visible,
        right_panel_visible,
        bottom_panel_visible,
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
