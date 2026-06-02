use egui::{Frame, Sense, Ui, UiBuilder, pos2, vec2};
use segs_ui::style::CtxStyleExt;

use crate::{app::AppContext, ui::widgets::WidgetData};

pub const GRID_GRANULARITY: f32 = 50.0;

#[derive(Default)]
pub struct WidgetGrid<'a> {
    widgets: Option<&'a [WidgetData]>,
    show_snap_guide: bool,
}

impl<'a> WidgetGrid<'a> {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    /// Draw these widgets
    pub fn with_widgets(mut self, widgets: &'a [WidgetData]) -> Self {
        self.widgets = Some(widgets);
        self
    }

    /// Draw a dotted background to help visualize snap zones for widget placement
    pub fn show_snap_guide(mut self, snap_guide: bool) -> Self {
        self.show_snap_guide = snap_guide;
        self
    }

    pub fn show(self, ui: &mut Ui, appctx: &mut AppContext) {
        let Self {
            show_snap_guide,
            widgets,
        } = self;

        let rect = ui.available_rect_before_wrap();
        let origin = rect.min;

        // Determine how many slots fit
        let num_columns = (rect.width() / GRID_GRANULARITY).floor().max(1.);
        let num_rows = (rect.height() / GRID_GRANULARITY).floor().max(1.);

        // Calculate the spacing so dots stretch to perfectly fill the screen
        // This is the final span slot size, adjusted to the screen size
        let spacing_x = rect.width() / num_columns as f32;
        let spacing_y = rect.height() / num_rows as f32;

        if show_snap_guide {
            const DOT_RADIUS: f32 = 0.75;
            let painter = ui.painter();
            let color = ui.visuals().weak_text_color().gamma_multiply(0.75);

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

        let Some(widgets) = widgets else {
            return;
        };

        for widget in widgets {
            let widget_rect = widget.rect(origin, vec2(spacing_x, spacing_y));

            // Create a child ui for the widget container
            ui.scope_builder(
                UiBuilder::new().id(widget.id.with("_container")).max_rect(widget_rect),
                |ui| {
                    // Show a solid background behind the widget
                    Frame::new().fill(ui.app_style().main_panels_fill).show(ui, |ui| {
                        // Allocate the space for the widget in the grid
                        ui.allocate_rect(widget_rect, Sense::empty());
                        // Create a child ui for the widget content
                        ui.scope_builder(UiBuilder::new().id(widget.id).max_rect(widget_rect), |ui| {
                            // Hide overflowing content
                            ui.set_clip_rect(widget_rect);
                            // Finally show the widget
                            widget.show(ui, appctx);
                        });
                    });
                },
            );
        }
    }
}
