use std::time::{Duration, Instant};

use egui::{Align, CursorIcon, Frame, Layout, Panel, Response, Theme, Ui, Vec2};

use segs_assets::icons::{self, Icon};
use segs_memory::MemoryExt;
use segs_ui::{
    style::CtxStyleExt,
    widgets::buttons::{StatusBarButton, UnpaddedStatusBarButton},
};

use crate::app::AppContext;
use crate::dataflow::adapter::Status;
use crate::dataflow::transport::DataTransport::{Ethernet, Serial};
use crate::ui::modals::SourceModal;
use crate::ui::{command_panel, layout};

const RATE_WIDTH_REFERENCE: &str = "88.8 Hz";
const RX_ACTIVITY_PHASE: Duration = Duration::from_micros(62_500); // 8 Hz blink

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
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    show_left_side(ui, appctx);
                });
                ui.with_layout(Layout::right_to_left(Align::Min), show_right_side);
            });
        });

    let rect = response.response.rect;
    let y = rect.top() + stroke.width * 0.5;
    ui.painter().hline(rect.x_range(), y, stroke);
}

fn show_left_side(ui: &mut Ui, appctx: &mut AppContext) {
    let source_id = ui.id().with("status_bar_source");
    let mut source_selection: bool = ui.mem().get_temp_or_default(source_id);

    let adapter_status = appctx.data_adapter.as_ref().map(|adapter| adapter.status());
    let res = show_source_status(ui, adapter_status);
    if res.clicked() {
        source_selection = !source_selection;
    }

    if source_selection && SourceModal::new(appctx).show(ui).should_close() {
        source_selection = false;
    }
    ui.mem().insert_temp(source_id, source_selection);

    let icon = if command_panel::is_open(ui) {
        icons::Terminal2::solid()
    } else {
        icons::Terminal2::outline()
    };
    let button = UnpaddedStatusBarButton::default()
        .padded()
        .add_icon(icon)
        .add_text("Commands");
    if ui.add(button).on_hover_cursor(CursorIcon::PointingHand).clicked() {
        command_panel::toggle(ui);
    }

    let layout_name = appctx
        .layouts
        .active()
        .map_or("No layout selected", |layout| layout.name.as_str());
    let layout_button = UnpaddedStatusBarButton::default()
        .padded()
        .add_icon(icons::Layout::outline())
        .add_text(layout_name);
    let layout_response = ui
        .add(layout_button)
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text("Open Layout Manager");
    if layout_response.clicked() {
        layout::request_open_manager(ui, &appctx.layouts);
    }
    layout::show_open_manager_prompt(ui, &mut appctx.layouts, &layout_response);
}

fn show_source_status(ui: &mut Ui, status: Option<Status>) -> Response {
    let activity_id = ui.id().with("status_bar_rx_activity");
    let mut activity = ui.mem().get_temp_or_default::<RxActivity>(activity_id);

    let Some(status) = status else {
        ui.mem().insert_temp(activity_id, RxActivity::default());

        let button = UnpaddedStatusBarButton::default()
            .add_dot(ui.app_style().error_fill)
            .add_text_with_width_of("Offline", RATE_WIDTH_REFERENCE)
            .padded();

        return ui
            .add(button)
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_ui(|ui| {
                show_status_tooltip(ui, icons::PlugConnectedX, "Disconnected");
            });
    };

    let (illuminated, repaint_after) = activity.update(Instant::now(), status.rx.count, status.rx.last_time);
    if let Some(repaint_after) = repaint_after {
        ui.ctx().request_repaint_after(repaint_after);
    }
    ui.mem().insert_temp(activity_id, activity);

    let dot_color = if illuminated {
        ui.app_style().success_fill
    } else {
        ui.app_style().neutral_fill
    };
    let button = UnpaddedStatusBarButton::default()
        .add_dot(dot_color)
        .add_text_with_width_of(format!("{:.1} Hz", status.rx.rate), RATE_WIDTH_REFERENCE)
        .padded();
    let response = ui.add(button).on_hover_cursor(CursorIcon::PointingHand);

    match status.transport {
        Ethernet { .. } => response.on_hover_ui(|ui| {
            show_status_tooltip(ui, icons::Ethernet, "Connected");
        }),
        Serial { .. } => response.on_hover_ui(|ui| {
            show_status_tooltip(ui, icons::Usb, "Connected");
        }),
    }
}

fn show_status_tooltip(ui: &mut Ui, icon: impl Icon, text: &str) {
    ui.horizontal(|ui| {
        ui.add(
            icon.to_image()
                .fit_to_exact_size(Vec2::splat(15.))
                .tint(ui.visuals().text_color()),
        );
        ui.label(text);
    });
}

fn show_right_side(ui: &mut egui::Ui) {
    // Place the theme toggle first so it is rightmost in the right-to-left layout
    show_theme_toggle(ui);

    // Show notification controls
    let notifications_id = ui.id().with("status_bar_notifications");
    let mut notifications_visible: bool = ui.mem().get_temp_or_default(notifications_id);

    let bell_icon = if notifications_visible {
        icons::Bell::solid()
    } else {
        icons::Bell::outline()
    };
    let btn = UnpaddedStatusBarButton::default().add_icon(bell_icon);
    let res = ui.add(btn);
    if res.on_hover_cursor(CursorIcon::PointingHand).clicked() {
        notifications_visible = !notifications_visible;
    }
    ui.mem().insert_temp(notifications_id, notifications_visible);

    // Show quick command controls
    let btn = UnpaddedStatusBarButton::default()
        .padded()
        .add_icon(icons::Lightning)
        .add_text("Quick Commands");
    ui.add(btn);
}

/// Shows the theme toggle and switches between the light and dark themes when clicked.
fn show_theme_toggle(ui: &mut Ui) {
    let dark_mode = ui.visuals().dark_mode;

    // Show the icon for the theme that will be activated
    let clicked = if dark_mode {
        let button = UnpaddedStatusBarButton::default().add_icon(icons::Sun::outline());
        ui.add(button).on_hover_cursor(CursorIcon::PointingHand).clicked()
    } else {
        let button = UnpaddedStatusBarButton::default().add_icon(icons::Moon::outline());
        ui.add(button).on_hover_cursor(CursorIcon::PointingHand).clicked()
    };

    // Switch to the opposite theme
    if clicked {
        ui.ctx().set_theme(if dark_mode { Theme::Light } else { Theme::Dark });
        ui.ctx().request_discard("theme change");
    }
}

/// Tracks RX frame activity across UI updates and drives the indicator blink phases.
#[derive(Clone, Copy, Default)]
struct RxActivity {
    last_frame: Option<Instant>,
    phase: RxActivityPhase,
    pending: bool,
}

impl RxActivity {
    /// Records newly received frames and advances the activity indicator state.
    ///
    /// `count` distinguishes a real first frame from the initial timestamp, while
    /// `last_frame` identifies activity that has not been observed by the UI yet.
    ///
    /// Returns whether the indicator is currently illuminated and, while a blink
    /// is active, the delay before the UI should repaint for the next transition.
    fn update(&mut self, now: Instant, count: u32, last_frame: Instant) -> (bool, Option<Duration>) {
        if count > 0 && self.last_frame != Some(last_frame) {
            self.last_frame = Some(last_frame);
            self.pending = true;
        }

        loop {
            match self.phase {
                // Start a blink when activity is waiting to be displayed
                RxActivityPhase::Idle if self.pending => {
                    self.pending = false;
                    self.phase = RxActivityPhase::Illuminated(now + RX_ACTIVITY_PHASE);
                }
                // Stay idle until a new frame arrives
                RxActivityPhase::Idle => return (false, None),
                // Begin the dark half of the blink after illumination expires
                RxActivityPhase::Illuminated(until) if now >= until => {
                    self.phase = RxActivityPhase::Dark(until + RX_ACTIVITY_PHASE);
                }
                // Keep the indicator lit and repaint at the phase deadline
                RxActivityPhase::Illuminated(until) => {
                    return (true, Some(until.saturating_duration_since(now)));
                }
                // Finish the blink after the dark phase expires
                RxActivityPhase::Dark(until) if now >= until => {
                    self.phase = RxActivityPhase::Idle;
                }
                // Keep the indicator dark and repaint at the phase deadline
                RxActivityPhase::Dark(until) => {
                    return (false, Some(until.saturating_duration_since(now)));
                }
            }
        }
    }
}

/// Phases of the RX activity indicator's non-blocking blink state machine.
#[derive(Clone, Copy, Default)]
enum RxActivityPhase {
    /// The indicator is dark and no repaint is scheduled.
    #[default]
    Idle,
    /// The indicator remains lit until the stored deadline.
    Illuminated(Instant),
    /// The indicator remains dark until the stored deadline.
    Dark(Instant),
}
