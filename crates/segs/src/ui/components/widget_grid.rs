use egui::{Frame, Response, Sense, Ui, UiBuilder, Vec2, pos2};
use segs_ui::style::CtxStyleExt;

use crate::{
    dataflow::DataStore,
    ui::{grid::Grid, widgets::WidgetData},
};

/// Draws the widgets on the grid.
///
/// When in edit mode, grid indicators are drawn and the widgets are rendered in a disabled style.
pub struct WidgetGrid<'a> {
    widgets: &'a mut [WidgetData],
    grid: &'a Grid,
    edit_mode: bool,
}

impl<'a> WidgetGrid<'a> {
    pub fn new(widgets: &'a mut [WidgetData], grid: &'a Grid) -> Self {
        Self {
            widgets,
            grid,
            edit_mode: false,
        }
    }

    /// Enable edit mode for the widgets, allowing them to be dragged and resized.
    pub fn edit_mode(mut self, mode: bool) -> Self {
        self.edit_mode = mode;
        self
    }

    /// Show the widgets in the grid.
    ///
    /// Returns the widget currently being hovered/dragged for edit mode interactions, if any.
    pub fn show(self, ui: &mut Ui, data_store: &mut DataStore) -> Option<(&'a mut WidgetData, Response)> {
        let Self {
            widgets,
            grid,
            edit_mode,
        } = self;

        let rect = grid.rect;

        if edit_mode {
            const DOT_RADIUS: f32 = 0.75;
            let painter = ui.painter();
            let color = ui.visuals().weak_text_color().gamma_multiply(0.75);

            let Vec2 {
                x: spacing_x,
                y: spacing_y,
            } = grid.cell_size;

            // Compute grid boundaries, adjust with spacing to avoid drawing dots on the very edge of the view
            let start_x = rect.min.x + spacing_x;
            let end_x = rect.max.x - spacing_x + 1.;
            let start_y = rect.min.y + spacing_y;
            let end_y = rect.max.y - spacing_y + 1.;
            // Draw the points
            let mut y = start_y;
            while y < end_y {
                let mut x = start_x;
                while x < end_x {
                    painter.circle_filled(pos2(x, y), DOT_RADIUS, color);
                    x += spacing_x;
                }
                y += spacing_y;
            }
        }

        let mut active_widget = None;

        for widget in widgets {
            let widget_rect = grid.to_screen_rect(widget.grect);

            // Create a child ui for the widget container
            ui.scope_builder(
                UiBuilder::new().id(widget.id.with("_container")).max_rect(widget_rect),
                |ui| {
                    // Show a solid background behind the widget
                    Frame::new().fill(ui.app_style().main_panels_fill).show(ui, |ui| {
                        // Allocate the space for the widget in the grid
                        let res = ui.allocate_rect(widget_rect, Sense::drag());

                        // Create a child ui for the widget content
                        ui.scope_builder(UiBuilder::new().id(widget.id).max_rect(widget_rect), |ui| {
                            // Disable the child if edit mode is active to prevent interactions
                            if edit_mode {
                                ui.disable();
                            }

                            // Hide overflowing content
                            ui.set_clip_rect(widget_rect);
                            // Finally show the widget
                            widget.show(ui, data_store);
                        });

                        // Save the currently active widget for edit mode interactions
                        if (res.hovered() || res.dragged()) && edit_mode && active_widget.is_none() {
                            active_widget = Some((widget, res));
                        }
                    });
                },
            );
        }

        active_widget
    }
}
