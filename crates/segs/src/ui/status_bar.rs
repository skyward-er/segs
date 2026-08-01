use std::sync::Arc;

use egui::{Align, CursorIcon, Frame, Layout, Panel, Ui, Vec2};

use segs_assets::icons::{self, Icon};
use segs_memory::MemoryExt;
use segs_ui::{
    style::CtxStyleExt,
    widgets::buttons::{StatusBarButton, UnpaddedStatusBarButton},
};

use crate::app::AppContext;
use crate::dataflow::transport::DataTransport::{Ethernet, Serial};
use crate::ui::modals::SourceModal;

/// Shows the status bar as a bottom panel of the application window, displaying information and controls relevant to
/// the current state of the application.
pub fn show(ui: &mut Ui, appctx: &mut AppContext) {
    let stroke = ui.app_style().main_view_stroke;
    let response = Panel::bottom("status_bar")
        .show_separator_line(false)
        .frame(Frame::new().fill(ui.style().visuals.panel_fill))
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6., 0.);
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| show_left_side(ui, appctx));
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| show_right_side(ui));
            });
        });

    let rect = response.response.rect;
    let y = rect.top() + stroke.width * 0.5;
    ui.painter().hline(rect.x_range(), y, stroke);
}

fn show_left_side(ui: &mut Ui, appctx: &mut AppContext) {
    let source_id = ui.id().with("status_bar_source");
    let mut source_selection: bool = ui.mem().get_temp_or_default(source_id);

    let adapter_status = appctx.data_adapter.as_ref().and_then(|a| Some(a.status()));

    let text = if adapter_status.is_some() {
        "Connected"
    } else {
        "Disconnected"
    };

    let icon: Arc<dyn Icon> = if let Some(s) = adapter_status {
        match s.transport {
            Ethernet { .. } => Arc::new(icons::Ethernet::default()),
            Serial { .. } => Arc::new(icons::Usb::default()),
        }
    } else {
        Arc::new(icons::PlugConnectedX::default())
    };

    let btn = UnpaddedStatusBarButton::default()
        .add_icon_dyn(icon)
        .add_text(text)
        .padded();
    let res = ui.add(btn).on_hover_cursor(CursorIcon::PointingHand);
    if res.clicked() {
        source_selection = !source_selection;
    }

    if source_selection {
        if SourceModal::new(appctx).show(ui).should_close() {
            source_selection = false;
        }
    }
    ui.mem().insert_temp(source_id, source_selection);
}

fn show_right_side(ui: &mut egui::Ui) {
    let notifications_id = ui.id().with("status_bar_notifications");
    let mut notifications_visible: bool = ui.mem().get_temp_or_default(notifications_id);

    let bell_icon = if notifications_visible {
        icons::Bell::solid()
    } else {
        icons::Bell::outline()
    };
    let btn = UnpaddedStatusBarButton::default().add_icon(bell_icon).add_space(4.);
    let res = ui.add(btn);
    if res.on_hover_cursor(CursorIcon::PointingHand).clicked() {
        notifications_visible = !notifications_visible;
    }
    ui.mem().insert_temp(notifications_id, notifications_visible);

    let btn = UnpaddedStatusBarButton::default()
        .padded()
        .add_icon(icons::Lightning)
        .add_text("Quick Commands");
    ui.add(btn);
}
