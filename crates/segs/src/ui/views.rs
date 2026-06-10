pub mod configuration;
pub mod operator;

use egui::{Align, CentralPanel, CornerRadius, Frame, Id, Layout, Panel, Ui};
use enum_dispatch::enum_dispatch;
use segs_memory::MemoryExt;
use segs_ui::{containers::ResizablePanel, style::CtxStyleExt};
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

pub const LEFT_PANEL_VISIBLE_ID: &str = "left_panel_visible";
pub const VIEW_MODE_ID: &str = "view_mode";

/// View represents what the user is currently looking at, imagine this as the
/// index of a document, but instead of pages, we index over possible layouts of
/// the UI. This is useful to keep track of which panels should be visible, and
/// which should not, as well as to keep track of the state of each view.
#[enum_dispatch(ViewTrait)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum View {
    Configuration(configuration::ConfigurationView),
    Operator(operator::OperatorView),
}

#[enum_dispatch]
trait ViewTrait {
    fn show_activities(&mut self, ui: &mut Ui, appctx: &mut AppContext);

    fn show_left_panel(&mut self, ui: &mut Ui, appctx: &mut AppContext);

    fn show_main_view(&mut self, ui: &mut Ui, appctx: &mut AppContext);
}

impl View {
    pub fn show(&mut self, ui: &mut Ui, appctx: &mut AppContext) {
        let app_style = ui.app_style();
        let style = ui.style().clone();
        let visuals = &style.visuals;
        let spacing = &style.spacing;

        let corner_radius = visuals.window_corner_radius;
        let main_view_corner_radius = {
            CornerRadius {
                nw: corner_radius.nw,
                ne: 0,
                sw: corner_radius.sw,
                se: 0,
            }
        };

        let left_panel_id = Id::new(LEFT_PANEL_VISIBLE_ID);

        Panel::left("activity_panel")
            .frame(Frame::new().fill(visuals.panel_fill))
            .resizable(false)
            .show_separator_line(false)
            .exact_size(34.)
            .show_inside(ui, |ui| self.show_activities(ui, appctx));

        // Read visibility flag after showing activities, since they might have modified it
        let mut left_panel_visible: bool = ui.mem().get_perm_or_default(left_panel_id);

        CentralPanel::default()
            .frame(Frame::new().fill(visuals.panel_fill))
            .show_inside(ui, |ui| {
                // Define collapse state based on visibility
                let mut collapsed_left = !left_panel_visible;

                let panel_outer_frame = Frame::new()
                    .corner_radius(corner_radius)
                    .fill(app_style.main_panels_fill);
                let panel_inner_frame = Frame::new().inner_margin(spacing.window_margin);

                let main_outer_frame = Frame::new()
                    .corner_radius(main_view_corner_radius)
                    .fill(app_style.main_panels_fill)
                    .stroke(app_style.main_view_stroke);
                let main_inner_frame = Frame::new()
                    .corner_radius(corner_radius)
                    .fill(app_style.main_panels_fill);

                let left_resizable_panel = ResizablePanel::horizontal_left()
                    .set_minimum_size(180.)
                    .inactive_separator_stroke(app_style.main_view_stroke)
                    .left_frame(panel_outer_frame)
                    .collapsed(&mut collapsed_left)
                    .animate(true);

                let layout = Layout::top_down(Align::Min);

                main_outer_frame.show(ui, |ui| {
                    left_resizable_panel
                        .show(ui, |panel| {
                            panel
                                .show_left(|ui| {
                                    // Show left panel content
                                    panel_inner_frame.show(ui, |ui| {
                                        ui.set_min_size(ui.available_size());
                                        ui.set_clip_rect(ui.max_rect());
                                        ui.with_layout(layout, |ui| self.show_left_panel(ui, appctx));
                                    });
                                })
                                .show_right(|ui| {
                                    // Show main view content
                                    main_inner_frame.show(ui, |ui| {
                                        ui.set_min_size(ui.available_size());
                                        ui.with_layout(layout, |ui| self.show_main_view(ui, appctx));
                                    });
                                });
                        })
                        .inner
                });

                // Update visibility state based on collapses
                left_panel_visible = !collapsed_left;
            });

        ui.mem().insert_perm(left_panel_id, left_panel_visible);
    }
}

impl Default for View {
    fn default() -> Self {
        Self::Configuration(configuration::ConfigurationView::default())
    }
}
