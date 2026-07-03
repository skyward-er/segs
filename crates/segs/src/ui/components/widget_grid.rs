use egui::{CornerRadius, Frame, Rect, Response, Sense, StrokeKind, Ui, UiBuilder, Vec2, pos2};
use segs_memory::MemoryExt;
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

        let app_style = ui.app_style();
        let corner_radius = CornerRadius::same(1);

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
            let drag_rect_id = widget.id.with("drag_rect");
            let floating: Option<Rect> = ui.mem().get_temp(drag_rect_id);

            // While a drag (move or resize) is in progress, render the widget at its floating
            // (unsnapped) rect
            let widget_rect = floating.unwrap_or_else(|| grid.to_screen_rect(widget.grect));

            // Create a child ui for the widget container
            ui.scope_builder(
                UiBuilder::new().id(widget.id.with("_container")).max_rect(widget_rect),
                |ui| {
                    // Show a solid background behind the widget
                    Frame::new()
                        .corner_radius(corner_radius)
                        .fill(app_style.main_panels_fill)
                        .show(ui, |ui| {
                            // Allocate the space for the widget in the grid
                            let res = ui.allocate_rect(widget_rect, Sense::drag());

                            // Drag ended (or the state was orphaned): snap the floating rect to the grid
                            if let Some(floating) = floating
                                && !res.dragged()
                            {
                                widget.grect = grid.to_grid_rect(floating);
                                ui.mem().remove_temp::<Rect>(drag_rect_id);
                            }

                            // Create a child ui for the widget content
                            ui.scope_builder(UiBuilder::new().id(widget.id).max_rect(widget_rect), |ui| {
                                // Disable the child if edit mode is active to prevent interactions
                                if edit_mode {
                                    ui.disable();
                                }

                                // Manually draw the widget background stroke
                                ui.painter().rect_stroke(
                                    res.rect,
                                    corner_radius,
                                    app_style.main_view_stroke,
                                    StrokeKind::Outside,
                                );

                                // Hide overflowing content
                                ui.set_clip_rect(widget_rect);
                                // Finally show the widget
                                widget.show(ui, data_store);
                            });

                            // Save the currently active widget for edit mode interactions.
                            // A dragged/drag-stopped widget takes priority over a merely hovered one.
                            if edit_mode
                                && (res.dragged() || res.drag_stopped() || (active_widget.is_none() && res.hovered()))
                            {
                                active_widget = Some((widget, res));
                            }
                        });
                },
            );
        }

        active_widget
    }
}
