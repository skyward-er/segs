#![allow(dead_code, unused)]

mod dataflow_editor;
mod layout_composer;
mod level_editor;
mod online_resources;
mod pane_controls;

use egui::{
    Align, CentralPanel, Context, CornerRadius, CursorIcon, Frame, Id, Layout, Margin, Sense, Panel, Ui, Vec2, vec2,
};
use enum_dispatch::enum_dispatch;
use segs_assets::icons::{self, Icon};
use segs_memory::MemoryExt;
use segs_ui::{containers::ResizablePanel, style::CtxStyleExt};
use serde::{Deserialize, Serialize};

use crate::ui::components::{
    left_menu::LeftBarMenuButton,
    mode_toggle::{Mode, ModeToggle},
};

/// View subtype representing the different configuration views available when
/// the user is in the Configuration mode.
#[enum_dispatch(ConfigurationViewTrait)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigurationView {
    DataflowEditor(dataflow_editor::DataflowEditorView),
    LevelEditor(level_editor::LevelEditorView),
    LayoutComposer(layout_composer::LayoutComposerView),
    PaneControls(pane_controls::PaneControlsView),
    OnlineResources(online_resources::OnlineResourcesView),
}

#[enum_dispatch]
trait ConfigurationViewTrait {
    fn top_bar_left_fn(&mut self, _ui: &mut Ui) {
        // Default implementation does nothing
    }

    fn top_bar_right_fn(&mut self, _ui: &mut Ui) {
        // Default implementation does nothing
    }

    fn main_view_left_fn(&mut self, _ui: &mut Ui) {
        // Default implementation does nothing
    }

    fn main_view_right_fn(&mut self, _ui: &mut Ui) {
        // Default implementation does nothing
    }
}

impl ConfigurationView {
    fn left_bar_fn(&mut self, ui: &mut Ui) {
        let id = Id::new("left_bar_selector");
        let mut selector = ui.mem().get_perm_or_default(id);

        ui.spacing_mut().item_spacing = Vec2::ZERO;
        ui.add_space(5.);
        ui.add(LeftBarMenuButton::new(
            &mut selector,
            ConfigurationView::PaneControls(pane_controls::PaneControlsView),
            icons::RectangleVertical::outline(),
            icons::RectangleVertical::solid(),
        ));
        ui.add(LeftBarMenuButton::new(
            &mut selector,
            ConfigurationView::LayoutComposer(layout_composer::LayoutComposerView),
            icons::Layout::outline(),
            icons::Layout::solid(),
        ));
        ui.add(LeftBarMenuButton::new(
            &mut selector,
            ConfigurationView::LevelEditor(level_editor::LevelEditorView),
            icons::Stack::outline(),
            icons::Stack::solid(),
        ));
        ui.add(LeftBarMenuButton::new(
            &mut selector,
            ConfigurationView::DataflowEditor(dataflow_editor::DataflowEditorView),
            icons::Function::outline(),
            icons::Function::solid(),
        ));
        ui.add(LeftBarMenuButton::new(
            &mut selector,
            ConfigurationView::OnlineResources(online_resources::OnlineResourcesView),
            icons::Cloud::outline(),
            icons::Cloud::solid(),
        ));

        ui.mem().insert_perm(id, selector);
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            Self::PaneControls(_) => "Pane controls",
            Self::LayoutComposer(_) => "Layout composer",
            Self::LevelEditor(_) => "Level editor",
            Self::DataflowEditor(_) => "Dataflow editor",
            Self::OnlineResources(_) => "Online resources",
        }
    }
}

impl super::ViewTrait for ConfigurationView {
    fn top_bar_left_fn(&mut self, ui: &mut Ui) {
        <Self as ConfigurationViewTrait>::top_bar_left_fn(self, ui);
    }

    fn top_bar_right_fn(&mut self, ui: &mut Ui) {
        <Self as ConfigurationViewTrait>::top_bar_right_fn(self, ui);
    }

    fn top_bar_middle_fn(&mut self, ui: &mut Ui) {
        let style = ui.app_style();

        let mode_width = 41.;
        let pad_width = 5.;
        let selection_width = ui.available_width() - mode_width - pad_width;
        let height = 20.;

        // Frame::new()
        //     .fill(style.main_view_fill)
        //     .stroke(style.main_view_stroke)
        //     .inner_margin(Margin::symmetric(0, 0))
        //     .outer_margin(Margin::symmetric(0, 2))
        //     .corner_radius(5)
        //     .show(ui, |ui| {
        //         let (selection_rect, selection_response) =
        //             ui.allocate_exact_size(vec2(selection_width, height),
        // Sense::click());         // ui.set_min_size(selection_rect.size());
        //     });

        // ui.add_space(pad_width);

        let id = Id::new("current_mode");
        let mut mode: Mode = ui.mem().get_temp_or_default(id);

        ModeToggle::new(&mut mode).with_height(22.).with_width(300.).show(ui);

        ui.mem().insert_temp(id, mode);
    }

    fn main_view_fn(&mut self, ui: &mut Ui) {
        let frame = Frame::new().fill(ui.style().visuals.panel_fill);
        Panel::left("menu_panel")
            .frame(frame)
            .resizable(false)
            .show_separator_line(false)
            .exact_size(34.)
            .show_inside(ui, |ui| self.left_bar_fn(ui));

        let app_style = ui.app_style();
        let visuals = &ui.style().visuals;
        let id = Id::new("composition_left_panel_visible");
        let mut left_panel_visible: bool = ui.mem().get_perm_or_default(id);
        CentralPanel::default()
            .frame(Frame::new().fill(visuals.panel_fill))
            .show_inside(ui, |ui| {
                // Define collapse state based on visibility
                let mut collapsed_left = !left_panel_visible;

                let visuals = ui.app_style();
                let panel_outer_frame = Frame::new().corner_radius(5.).fill(visuals.main_panels_fill);
                let panel_inner_frame = Frame::NONE;
                let main_inner_frame = panel_inner_frame.corner_radius(5.).fill(visuals.main_panels_fill);

                let left = ResizablePanel::horizontal_left()
                    .set_minimum_size(180.)
                    .inactive_separator_stroke(visuals.main_view_stroke)
                    .left_frame(panel_outer_frame)
                    .collapsed(&mut collapsed_left);

                let layout = Layout::top_down(Align::Min);

                let cr = CornerRadius {
                    nw: 5,
                    ne: 0,
                    se: 0,
                    sw: 5,
                };
                Frame::new()
                    .corner_radius(cr)
                    .fill(app_style.main_panels_fill)
                    .stroke(app_style.main_view_stroke)
                    .show(ui, |ui| {
                        left.show(ui, |panel| {
                            panel
                                .show_left(|ui| {
                                    panel_inner_frame.show(ui, |ui| {
                                        ui.set_min_size(ui.available_size());
                                        ui.set_clip_rect(ui.max_rect());
                                        ui.with_layout(layout, |ui| self.main_view_left_fn(ui));
                                    });
                                })
                                .show_right(|ui| {
                                    main_inner_frame.show(ui, |ui| {
                                        ui.set_min_size(ui.available_size());
                                        ui.with_layout(layout, |ui| self.main_view_right_fn(ui));
                                    });
                                });
                        })
                        .inner
                    });

                // Update visibility state based on collapses
                left_panel_visible = !collapsed_left;
            });
        ui.mem().insert_perm(id, left_panel_visible);
    }
}

impl Default for ConfigurationView {
    fn default() -> Self {
        Self::DataflowEditor(dataflow_editor::DataflowEditorView)
    }
}
