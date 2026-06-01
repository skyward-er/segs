pub mod configuration;
pub mod operator;

use egui::{CentralPanel, CornerRadius, Frame, Id, Panel, Ui, UiBuilder};
use enum_dispatch::enum_dispatch;
use segs_memory::MemoryExt;
use segs_ui::style::CtxStyleExt;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

const LEFT_PANEL_VISIBLE_ID: &str = "left_panel_visible";

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

        let left_panel_id = Id::new(LEFT_PANEL_VISIBLE_ID);

        Panel::left("activity_panel")
            .frame(Frame::new().fill(visuals.panel_fill))
            .resizable(false)
            .show_separator_line(false)
            .exact_size(34.)
            .show_inside(ui, |ui| self.show_activities(ui, appctx));

        // Read visibility flag after showing activities, since they might have modified it
        let left_panel_visible = ui.mem().get_perm_or_default(left_panel_id);
        Panel::left("left_panel")
            .frame(
                Frame::new()
                    .fill(visuals.panel_fill)
                    .inner_margin(spacing.window_margin),
            )
            .resizable(false)
            .exact_size(180.)
            .show_separator_line(false)
            .show_animated_inside(ui, left_panel_visible, |ui| self.show_left_panel(ui, appctx));

        CentralPanel::default()
            .frame(Frame::new().fill(visuals.panel_fill))
            .show_inside(ui, |ui| {
                let corner_radius = {
                    let cr = visuals.window_corner_radius;
                    CornerRadius {
                        nw: cr.nw,
                        ne: 0,
                        sw: cr.sw,
                        se: 0,
                    }
                };

                Frame::new()
                    .corner_radius(corner_radius)
                    .fill(app_style.main_panels_fill)
                    .stroke(app_style.main_view_stroke)
                    .show(ui, |ui| {
                        ui.scope_builder(UiBuilder::new().id_salt("_contents"), |ui| {
                            ui.expand_to_include_rect(ui.max_rect());
                            self.show_main_view(ui, appctx)
                        })
                    });
            });

        ui.mem().insert_perm(left_panel_id, left_panel_visible);
    }
}

impl Default for View {
    fn default() -> Self {
        Self::Configuration(configuration::ConfigurationView::default())
    }
}
