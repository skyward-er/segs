mod gallery;
mod settings;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use egui::{
    CentralPanel, Color32, Frame, Id, Panel, Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2,
};
use segs_assets::icons;
use segs_memory::MemoryExt;
use segs_ui::{components::panel_header::PanelHeader, style::CtxStyleExt, widgets::buttons::IconBtn};

use crate::{
    app::AppContext,
    ui::{
        components::{
            widget_editor::{
                HitRegion, clamp_rect_to, hit_region, resize_rect, set_cursor, show_hover, show_outline,
                show_remove_button, show_selection, show_snap_preview,
            },
            widget_renderer::{show_snapping_guide, show_widget, show_widgets},
        },
        grid::Grid,
        layout,
        popups::GridSettingsPopup,
        views::ViewTrait,
        widgets::{WidgetTrait, WidgetVariant},
    },
};

const SELECTED_WIDGET_ID: &str = "selected_widget";
const GRID_SETTINGS_VISIBLE_ID: &str = "configuration_grid_settings_visible";
const GRID_SETTINGS_BUTTON_ID: &str = "configuration_grid_settings_button";
const GRID_SETTINGS_BUTTON_SIZE: Vec2 = vec2(24., 24.);
const GRID_SETTINGS_BUTTON_MARGIN: f32 = 4.;
static NEXT_DRAG_SESSION: AtomicU64 = AtomicU64::new(1);

/// View subtype representing the different configuration views available when
/// the user is in the Configuration mode.
#[derive(Default)]
pub struct ConfigurationView {}

enum WidgetDragSource {
    Layout(Id),
    Gallery(WidgetVariant),
}

struct WidgetDragPayload {
    source: WidgetDragSource,
    session: u64,
    snap_generation: AtomicU64,
    snap_visible: AtomicBool,
    interaction: HitRegion,
    initial_rect: Option<Rect>,
    pointer_offset: Vec2,
}

impl ViewTrait for ConfigurationView {
    fn show_main_view(&mut self, ui: &mut Ui, appctx: &mut AppContext) {
        if appctx.layouts.active().is_none() {
            return;
        }
        let app_style = ui.app_style();
        let panel_frame = Frame::new().fill(app_style.main_panels_fill);

        Panel::left("configuration_widget_gallery")
            .default_size(200.)
            .min_size(180.)
            .max_size(300.)
            .frame(panel_frame)
            .show_inside(ui, |ui| {
                show_panel(ui, "WIDGET GALLERY", "Drag to add to the layout", |ui| {
                    gallery::show(ui, &mut appctx.data_store);
                });
            });

        Panel::right("configuration_widget_settings")
            .default_size(200.)
            .min_size(180.)
            .max_size(300.)
            .frame(panel_frame)
            .show_inside(ui, |ui| {
                show_panel(ui, "WIDGET SETTINGS", "Edit the selected widget", |ui| {
                    let selected = selected_widget(ui);
                    let protocol = appctx.data_adapter.as_ref().map(|adapter| adapter.describe_protocol());
                    let widget = selected.and_then(|id| {
                        appctx
                            .layouts
                            .active_mut()
                            .and_then(|layout| layout.widgets.iter_mut().find(|widget| widget.id == id))
                    });
                    settings::show(ui, widget, protocol);
                });
            });

        let grid_settings = appctx.layouts.active().expect("active layout checked").grid_settings;
        let grid = Grid::new(ui.available_rect_before_wrap(), grid_settings);

        CentralPanel::default()
            .frame(Frame::new().fill(app_style.main_panels_fill))
            .show_inside(ui, |ui| show_layout_editor(ui, appctx, &grid));

        show_widget_drag(ui, appctx, &grid);
    }
}

/// Draws the layout and handles editor interactions.
fn show_layout_editor(ui: &mut Ui, appctx: &mut AppContext, grid: &Grid) {
    show_snapping_guide(ui, grid);

    let grid_response = ui.allocate_rect(grid.rect, Sense::click());

    let drag_in_progress = egui::DragAndDrop::has_payload_of_type::<WidgetDragPayload>(ui.ctx());
    let pointer = ui.ctx().pointer_interact_pos();
    let mut hovered_widget = None;
    let mut widget_clicked = false;

    if !drag_in_progress {
        for widget in &appctx
            .layouts
            .active()
            .expect("configuration requires a layout")
            .widgets
        {
            let rect = grid.to_screen_rect(widget.grect);
            let response = ui.interact(rect, widget.id.with("edit_interaction"), Sense::click_and_drag());

            if response.clicked() {
                set_selected_widget(ui, Some(widget.id));
                widget_clicked = true;
            }

            // Keep the pressed edge active when the pointer leaves the widget
            let pointer_down = response.is_pointer_button_down_on();
            let region_pointer = if pointer_down {
                ui.input(|input| input.pointer.press_origin()).or(pointer)
            } else {
                pointer
            };
            let region = region_pointer.map_or(HitRegion::OUTSIDE, |pointer| hit_region(rect, pointer));
            if response.drag_started()
                && let Some(pointer) = response.interact_pointer_pos()
            {
                // Preserve the resize handle and offset from the initial press position
                let origin = ui.input(|input| input.pointer.press_origin()).unwrap_or(pointer);
                egui::DragAndDrop::set_payload(
                    ui.ctx(),
                    WidgetDragPayload {
                        source: WidgetDragSource::Layout(widget.id),
                        session: next_drag_session(),
                        snap_generation: AtomicU64::new(0),
                        snap_visible: AtomicBool::new(false),
                        interaction: hit_region(rect, origin),
                        initial_rect: Some(rect),
                        pointer_offset: origin - rect.min,
                    },
                );
                hovered_widget = None;
                break;
            }

            // Keep edit controls visible while the widget owns the pointer
            if ui.rect_contains_pointer(rect) || pointer_down {
                hovered_widget = Some((widget.id, rect, region));
            }
        }
    }

    // Hide a layout widget as soon as its floating drag starts
    let dragged_layout_widget = egui::DragAndDrop::payload::<WidgetDragPayload>(ui.ctx()).and_then(|payload| {
        if let WidgetDragSource::Layout(id) = payload.source {
            Some(id)
        } else {
            None
        }
    });

    ui.add_enabled_ui(false, |ui| {
        show_widgets(
            ui,
            appctx
                .layouts
                .active()
                .expect("configuration requires a layout")
                .widgets
                .iter()
                .filter(|widget| Some(widget.id) != dragged_layout_widget),
            grid,
            &mut appctx.data_store,
        );
    });

    let selected = selected_widget(ui);
    if let Some(widget) = appctx
        .layouts
        .active()
        .expect("configuration requires a layout")
        .widgets
        .iter()
        .find(|widget| Some(widget.id) == selected && Some(widget.id) != dragged_layout_widget)
    {
        show_selection(ui, grid.to_screen_rect(widget.grect));
    }

    let mut remove_requested = None;
    if let Some((id, rect, region)) = hovered_widget {
        show_hover(ui, rect);
        show_outline(ui, rect);
        set_cursor(ui, region, false);
        if show_remove_button(ui, rect) {
            remove_requested = Some(id);
        }
    }

    if let Some(id) = remove_requested {
        appctx
            .layouts
            .active_mut()
            .expect("configuration requires a layout")
            .remove_widget(id);
        if selected_widget(ui) == Some(id) {
            set_selected_widget(ui, None);
        }
    }

    let layout_control_clicked = show_layout_controls(ui, appctx, grid);
    if grid_response.clicked() && !widget_clicked && remove_requested.is_none() && !layout_control_clicked {
        set_selected_widget(ui, None);
    }
}

/// Draws the grid settings button and its anchored popup.
fn show_layout_controls(ui: &mut Ui, appctx: &mut AppContext, grid: &Grid) -> bool {
    let done_rect = Rect::from_min_size(
        pos2(
            grid.rect.right() - GRID_SETTINGS_BUTTON_SIZE.x - GRID_SETTINGS_BUTTON_MARGIN,
            grid.rect.top() + GRID_SETTINGS_BUTTON_MARGIN,
        ),
        GRID_SETTINGS_BUTTON_SIZE,
    );
    let done_response = IconBtn::new(icons::Check)
        .show_at(ui, done_rect, Id::new("configuration_done_editing_button"))
        .on_hover_text("Done Editing");
    if done_response.clicked() {
        layout::request_done_editing(ui, &appctx.layouts);
    }
    layout::show_done_editing_prompt(ui, &mut appctx.layouts, &done_response);

    let grid_rect = done_rect.translate(vec2(-GRID_SETTINGS_BUTTON_SIZE.x - GRID_SETTINGS_BUTTON_MARGIN, 0.));
    let response = IconBtn::new(icons::GridSettings)
        .show_at(ui, grid_rect, Id::new(GRID_SETTINGS_BUTTON_ID))
        .on_hover_text("Grid Settings");

    let mut any_clicked = done_response.clicked() || response.clicked();
    if appctx.layouts.is_dirty() {
        let save_rect = grid_rect.translate(vec2(-GRID_SETTINGS_BUTTON_SIZE.x - GRID_SETTINGS_BUTTON_MARGIN, 0.));
        let save_response = IconBtn::new(icons::Save)
            .show_at(ui, save_rect, Id::new("configuration_save_layout_button"))
            .on_hover_text("Save Layout");
        if save_response.clicked() {
            layout::save_active(ui, &mut appctx.layouts, save_response.id);
        }
        layout::show_control_error(ui, &save_response);
        any_clicked |= save_response.clicked();
    }

    let mut visible: bool = ui.mem().get_temp_or_default(Id::new(GRID_SETTINGS_VISIBLE_ID));
    if response.clicked() {
        visible = !visible;
        ui.ctx().request_repaint();
    }

    // Defer a newly opened popup so the toggle click is not treated as an outside click
    if visible && !response.clicked() {
        GridSettingsPopup::new(
            &mut visible,
            &mut appctx
                .layouts
                .active_mut()
                .expect("configuration requires a layout")
                .grid_settings,
            response.rect.right_bottom(),
        )
        .show(ui);
    }

    ui.mem().insert_temp(Id::new(GRID_SETTINGS_VISIBLE_ID), visible);
    any_clicked
}

/// Draws and commits the active widget drag.
fn show_widget_drag(ui: &mut Ui, appctx: &mut AppContext, grid: &Grid) {
    let Some(payload) = egui::DragAndDrop::payload::<WidgetDragPayload>(ui.ctx()) else {
        return;
    };
    let Some(pointer) = ui.ctx().pointer_interact_pos() else {
        return;
    };

    set_cursor(ui, payload.interaction, true);

    let (preview_id, variant, show_floating_selection) = match &payload.source {
        WidgetDragSource::Layout(id) => {
            let Some(widget) = appctx
                .layouts
                .active()
                .expect("configuration requires a layout")
                .widgets
                .iter()
                .find(|widget| widget.id == *id)
            else {
                egui::DragAndDrop::clear_payload(ui.ctx());
                return;
            };
            (*id, widget.variant.clone(), selected_widget(ui) == Some(*id))
        }
        WidgetDragSource::Gallery(variant) => (
            Id::new("gallery_widget_drag_preview").with(payload.session),
            variant.clone(),
            false,
        ),
    };

    let is_resize = payload.initial_rect.is_some() && payload.interaction != HitRegion::INSIDE;
    let raw_rect = match payload.initial_rect {
        Some(initial_rect) if is_resize => {
            // Stop resized edges at the grid boundary
            let pointer = grid.rect.clamp(pointer);
            resize_rect(
                initial_rect,
                pointer,
                payload.interaction,
                validated_size(variant.min_size(), Vec2::ONE) * grid.cell_size,
            )
        }
        Some(initial_rect) => Rect::from_min_size(pointer - payload.pointer_offset, initial_rect.size()),
        None => Rect::from_center_size(
            pointer,
            capped_grid_size(grid, variant.default_size(), variant.min_size()) * grid.cell_size,
        ),
    };

    let over_grid = is_resize || grid.rect.contains(pointer);
    let drop_candidate = if over_grid {
        // Snap only the drop target while keeping the floating widget under the pointer
        // Paint the target first so the floating widget passes over its outline
        let placement_rect = clamp_rect_to(raw_rect, grid.rect);
        // Use a fresh animation after each grid re-entry
        if !payload.snap_visible.swap(true, Ordering::Relaxed) {
            payload.snap_generation.fetch_add(1, Ordering::Relaxed);
        }
        let snap_generation = payload.snap_generation.load(Ordering::Relaxed);
        let drop_candidate = show_snap_preview(
            ui,
            grid,
            placement_rect,
            preview_id.with(("snap", payload.session, snap_generation)),
        );
        show_disabled_widget(ui, preview_id, raw_rect, &variant, &mut appctx.data_store);
        if show_floating_selection {
            show_selection(ui, raw_rect);
        }
        Some(drop_candidate)
    } else {
        payload.snap_visible.store(false, Ordering::Relaxed);
        show_disabled_widget(ui, preview_id, raw_rect, &variant, &mut appctx.data_store);
        if show_floating_selection {
            show_selection(ui, raw_rect);
        }
        show_rejected_tint(ui, raw_rect);
        None
    };

    if ui.input(|input| input.pointer.any_released()) {
        egui::DragAndDrop::clear_payload(ui.ctx());

        if let Some(grect) = drop_candidate {
            match &payload.source {
                WidgetDragSource::Layout(id) => {
                    if let Some(widget) = appctx
                        .layouts
                        .active_mut()
                        .expect("configuration requires a layout")
                        .widgets
                        .iter_mut()
                        .find(|widget| widget.id == *id)
                    {
                        widget.grect = grect;
                    }
                }
                WidgetDragSource::Gallery(variant) => {
                    appctx
                        .layouts
                        .active_mut()
                        .expect("configuration requires a layout")
                        .add_widget(variant.clone(), grect);
                }
            }
        } else if let WidgetDragSource::Layout(id) = payload.source {
            // Rejected layout drops delete the committed widget
            appctx
                .layouts
                .active_mut()
                .expect("configuration requires a layout")
                .remove_widget(id);
            if selected_widget(ui) == Some(id) {
                set_selected_widget(ui, None);
            }
        }
    }
}

/// Draws a widget without content interactions.
fn show_disabled_widget(
    ui: &mut Ui,
    id: Id,
    rect: Rect,
    variant: &WidgetVariant,
    data_store: &mut crate::dataflow::DataStore,
) {
    ui.scope(|ui| {
        ui.disable();
        show_widget(ui, id, rect, variant, data_store);
    });
}

/// Draws the invalid-placement overlay.
fn show_rejected_tint(ui: &Ui, rect: Rect) {
    let error = ui.visuals().error_fg_color;
    let tint = Color32::from_rgba_unmultiplied(error.r(), error.g(), error.b(), 48);
    ui.painter().rect_filled(rect, 1., tint);
    ui.painter().rect_stroke(
        rect,
        1.,
        Stroke::new(1.5, error.gamma_multiply(0.8)),
        StrokeKind::Middle,
    );
}

/// Returns the selected widget id.
fn selected_widget(ui: &Ui) -> Option<Id> {
    ui.mem().get_temp_or_default(Id::new(SELECTED_WIDGET_ID))
}

/// Updates the selected widget id.
fn set_selected_widget(ui: &Ui, id: Option<Id>) {
    ui.mem().insert_temp(Id::new(SELECTED_WIDGET_ID), id);
}

/// Returns a unique drag session number.
fn next_drag_session() -> u64 {
    NEXT_DRAG_SESSION.fetch_add(1, Ordering::Relaxed)
}

/// Checks whether a size is finite and positive.
fn valid_size(size: Vec2) -> bool {
    size.is_finite() && size.x > 0. && size.y > 0.
}

/// Returns a valid size or its fallback.
fn validated_size(size: Vec2, fallback: Vec2) -> Vec2 {
    if valid_size(size) { size } else { fallback }
}

/// Chooses a widget size that fits the grid.
fn capped_grid_size(grid: &Grid, default_size: Vec2, min_size: Vec2) -> Vec2 {
    let requested = if valid_size(default_size) {
        default_size
    } else if valid_size(min_size) {
        min_size
    } else {
        Vec2::ONE
    };
    let grid_extent = grid.rect.size() / grid.cell_size;
    Vec2::new(requested.x.min(grid_extent.x), requested.y.min(grid_extent.y))
}

fn show_panel(ui: &mut Ui, title: &str, subtitle: &str, content: impl FnOnce(&mut Ui)) {
    ui.add(PanelHeader::new(title).subtitle(subtitle));

    ScrollArea::vertical()
        .auto_shrink(false)
        .content_margin(ui.spacing().window_margin)
        .show(ui, |ui| {
            ui.vertical(content);
        });
}
