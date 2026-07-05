use egui::{Color32, CornerRadius, Frame, Id, Rect, Response, Sense, StrokeKind, Ui, UiBuilder, Vec2, pos2, vec2};
use segs_assets::icons;
use segs_memory::MemoryExt;
use segs_ui::{style::CtxStyleExt, widgets::buttons::IconBtn};

use crate::{
    dataflow::DataStore,
    ui::{grid::Grid, widgets::WidgetData},
};

/// Memory key for the id of the currently selected widget, if any.
const SELECTED_WIDGET_ID: &str = "selected_widget";

const SELECTION_TINT_ALPHA: u8 = 40;
const HOVER_DARKEN_ALPHA: u8 = 40;
const REMOVE_BUTTON_SIZE: Vec2 = vec2(28., 28.);
/// `IconBtn`'s default padding, plus 1 point.
const REMOVE_BUTTON_PADDING: f32 = 4.;

/// Id of the currently selected widget, if any. Lives in temp memory, not on [`WidgetData`].
pub fn selected_widget(ui: &Ui) -> Option<Id> {
    ui.mem().get_temp_or_default(Id::new(SELECTED_WIDGET_ID))
}

/// Selects `id`, replacing any previous selection.
pub fn set_selected_widget(ui: &Ui, id: Option<Id>) {
    ui.mem().insert_temp(Id::new(SELECTED_WIDGET_ID), id);
}

/// `color` with its alpha channel replaced, for translucent tints/overlays.
fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

/// Result of showing the widget grid for one frame.
pub struct WidgetGridResponse<'a> {
    /// The widget currently being hovered/dragged, for edit mode drag/resize interactions.
    pub active: Option<(&'a mut WidgetData, Response)>,
    /// The id of a widget whose remove button was clicked this frame, if any.
    pub remove_requested: Option<Id>,
}

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
    pub fn show(self, ui: &mut Ui, data_store: &mut DataStore) -> WidgetGridResponse<'a> {
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

        // Clears selection on empty-space clicks. Widgets are allocated after this and so are on
        // top, intercepting clicks on their own area first.
        if edit_mode && ui.allocate_rect(rect, Sense::click()).clicked() {
            set_selected_widget(ui, None);
        }

        let selected = edit_mode.then(|| selected_widget(ui)).flatten();

        let mut active_widget = None;
        let mut remove_requested = None;

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
                            // Plain `Sense::drag()`: with `click_and_drag` egui delays `dragged()`
                            // until it's sure this isn't a click, which lets the pointer drift off
                            // an edge before resize locks in. Selection uses a separate click-only
                            // interaction below instead.
                            let res = ui.allocate_rect(widget_rect, Sense::drag());

                            // Drag ended (or the state was orphaned): snap the floating rect to the grid
                            if let Some(floating) = floating
                                && !res.dragged()
                            {
                                widget.grect = grid.to_grid_rect(floating);
                                ui.mem().remove_temp::<Rect>(drag_rect_id);
                            }

                            // Must be registered after `res` so it wins hit-test priority for clicks.
                            let click_res = ui.interact(widget_rect, widget.id.with("select_click"), Sense::click());
                            if edit_mode && click_res.clicked() {
                                set_selected_widget(ui, Some(widget.id));
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

                            if edit_mode {
                                let is_selected = selected == Some(widget.id);
                                let is_hovered = res.dragged() || ui.rect_contains_pointer(widget_rect);

                                if is_selected {
                                    ui.painter().rect_filled(
                                        res.rect,
                                        corner_radius,
                                        with_alpha(app_style.accent_fill, SELECTION_TINT_ALPHA),
                                    );
                                }

                                if is_hovered {
                                    ui.painter().rect_filled(
                                        res.rect,
                                        corner_radius,
                                        with_alpha(Color32::BLACK, HOVER_DARKEN_ALPHA),
                                    );
                                }

                                // Hidden while dragging so it doesn't compete for clicks.
                                if is_hovered && !res.dragged() {
                                    let button_rect = Rect::from_center_size(res.rect.center(), REMOVE_BUTTON_SIZE);
                                    let button = IconBtn::new(icons::Trash).with_padding(REMOVE_BUTTON_PADDING);
                                    if ui.place(button_rect, button).clicked() {
                                        remove_requested = Some(widget.id);
                                    }
                                }
                            }

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

        WidgetGridResponse {
            active: active_widget,
            remove_requested,
        }
    }
}
