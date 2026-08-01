use bitflags::bitflags;
use egui::{Color32, CornerRadius, CursorIcon, Id, Pos2, Rect, Response, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use segs_assets::icons;
use segs_memory::MemoryExt;
use segs_ui::{style::CtxStyleExt, widgets::buttons::IconBtn};

use crate::ui::{grid::Grid, widgets::WidgetData};

/// How long the snap-preview rect takes to glide to a newly snapped grid cell, in seconds.
const SNAP_ANIMATION_TIME: f32 = 0.150;
const SELECTION_TINT_ALPHA: u8 = 40;
const HOVER_DARKEN_ALPHA: u8 = 40;
const REMOVE_BUTTON_SIZE: Vec2 = vec2(28., 28.);
/// `IconBtn`'s default padding, plus 1 point.
const REMOVE_BUTTON_PADDING: f32 = 4.;

/// `color` with its alpha channel replaced, for translucent editor overlays.
fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

/// Result of showing the editor for one frame.
pub struct WidgetEditorResponse {
    /// The id of the widget whose remove button was clicked, if any.
    pub remove_requested: Option<Id>,
}

pub struct WidgetEditor<'a> {
    grid: &'a Grid,
    widget: &'a mut WidgetData,
    response: Response,
}

impl<'a> WidgetEditor<'a> {
    /// Draws the selection tint independently of hover and drag interactions.
    pub fn show_selection(ui: &Ui, selected_rect: Option<Rect>) {
        if let Some(rect) = selected_rect {
            ui.painter().rect_filled(
                rect,
                CornerRadius::same(1),
                with_alpha(ui.app_style().accent_fill, SELECTION_TINT_ALPHA),
            );
        }
    }

    pub fn new(grid: &'a Grid, widget: &'a mut WidgetData, response: Response) -> Self {
        Self { grid, widget, response }
    }

    /// Shows edit controls for one frame.
    pub fn show(self, ui: &mut Ui) -> WidgetEditorResponse {
        let Self {
            grid,
            widget,
            response: res,
        } = self;

        let rect = res.rect;
        let is_hovered = res.dragged() || ui.rect_contains_pointer(rect);

        if is_hovered {
            ui.painter().rect_filled(
                rect,
                CornerRadius::same(1),
                with_alpha(Color32::BLACK, HOVER_DARKEN_ALPHA),
            );
        }

        // Keep the button registered while a click is in progress. The grid's pure drag response
        // becomes dragged on mouse-down, so hiding the button then would discard its click on release.
        let remove_requested = if is_hovered {
            let button_rect = Rect::from_center_size(rect.center(), REMOVE_BUTTON_SIZE);
            let button = IconBtn::new(icons::Trash).with_padding(REMOVE_BUTTON_PADDING);
            ui.place(button_rect, button).clicked().then_some(widget.id)
        } else {
            None
        };

        let Some(mut pointer_pos) = ui.input(|i| i.pointer.interact_pos()) else {
            return WidgetEditorResponse { remove_requested };
        };

        let hit_region_id = ui.id().with("_edit_hit_region");
        let mut last_hit_region = ui.mem().get_temp_or_insert(hit_region_id, None);

        let hit_region = get_hit_region(rect, pointer_pos);

        if res.drag_started() {
            last_hit_region = Some(hit_region);
        } else if !res.dragged() {
            last_hit_region = None;
        }
        ui.mem().insert_temp(hit_region_id, last_hit_region);

        let active_hit_region = if let Some(hr) = last_hit_region { hr } else { hit_region };

        let preview_stroke = Stroke {
            width: 2.,
            color: ui.app_style().accent_fill.gamma_multiply(0.75),
        };

        // While the widget is being moved or resized, only the snap-preview stroke is drawn (see
        // below): the indicator stroke on the widget itself would just duplicate/lag behind it.
        if !res.dragged() {
            // Use the widget's committed grid position rather than `rect`: on the frame a move
            // drag is released, `rect` still reflects the pre-snap floating rect for this frame,
            // while `widget.grect` has already been snapped by `WidgetGrid`'s commit.
            let indicator_rect = grid.to_screen_rect(widget.grect);
            ui.painter()
                .rect_stroke(indicator_rect, 1., preview_stroke, StrokeKind::Middle);
        }

        // Set cursor
        match active_hit_region {
            HitRegion::LEFT => ui.ctx().set_cursor_icon(CursorIcon::ResizeWest),
            HitRegion::RIGHT => ui.ctx().set_cursor_icon(CursorIcon::ResizeEast),
            HitRegion::TOP => ui.ctx().set_cursor_icon(CursorIcon::ResizeNorth),
            HitRegion::BOTTOM => ui.ctx().set_cursor_icon(CursorIcon::ResizeSouth),
            HitRegion::TOP_LEFT => ui.ctx().set_cursor_icon(CursorIcon::ResizeNorthWest),
            HitRegion::TOP_RIGHT => ui.ctx().set_cursor_icon(CursorIcon::ResizeNorthEast),
            HitRegion::BOTTOM_LEFT => ui.ctx().set_cursor_icon(CursorIcon::ResizeSouthWest),
            HitRegion::BOTTOM_RIGHT => ui.ctx().set_cursor_icon(CursorIcon::ResizeSouthEast),
            _ => {
                // The cursor is dragging the widget, regardless if the pointer is inside or outside
                ui.ctx().set_cursor_icon(CursorIcon::Grab);
            }
        }

        if !res.dragged() {
            // Widget is only being hovered, nothing more to do.
            return WidgetEditorResponse { remove_requested };
        }

        // Update the floating rect according to the drag interaction: translate while moving,
        // or adjust the relevant edge(s) while resizing. Either way the widget renders live at
        // this rect (see `WidgetGrid`) and only snaps to the grid once the drag ends.
        let drag_rect_id = widget.id.with("drag_rect");
        let floating = ui.mem().get_temp(drag_rect_id).unwrap_or(rect);

        let floating = match active_hit_region {
            HitRegion::INSIDE | HitRegion::OUTSIDE => {
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                clamp_rect_to(floating.translate(res.drag_delta()), grid.rect)
            }
            direction => {
                // Minimum rect the widget can be resized to, to avoid zero-sized widgets
                let min_rect = floating.shrink2(grid.cell_size);
                pointer_pos = grid.rect.clamp(pointer_pos);

                let mut widget_rect = floating;
                if direction.contains(HitRegion::LEFT) {
                    *widget_rect.left_mut() = pointer_pos.x.min(min_rect.right());
                }
                if direction.contains(HitRegion::RIGHT) {
                    *widget_rect.right_mut() = pointer_pos.x.max(min_rect.left());
                }
                if direction.contains(HitRegion::TOP) {
                    *widget_rect.top_mut() = pointer_pos.y.min(min_rect.bottom());
                }
                if direction.contains(HitRegion::BOTTOM) {
                    *widget_rect.bottom_mut() = pointer_pos.y.max(min_rect.top());
                }
                widget_rect
            }
        };
        ui.mem().insert_temp(drag_rect_id, floating);

        // Preview the grid-snapped target, animating toward it. The widget only actually snaps
        // there once the drag ends, see `WidgetGrid`.
        let target = grid.to_screen_rect(grid.to_grid_rect(floating));
        let anim_id = drag_session_id(ui, widget.id.with("_snap_preview_anim"), res.drag_started());
        let animated_target = animate_rect(ui.ctx(), anim_id, target, SNAP_ANIMATION_TIME);

        ui.painter()
            .rect_stroke(animated_target, 1., preview_stroke, StrokeKind::Middle);

        WidgetEditorResponse { remove_requested }
    }
}

bitflags! {
    /// The area of the widget the pointer is hovering on, for edit mode interactions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct HitRegion: u32 {
        const OUTSIDE = HitRegion::empty().bits();

        const LEFT = 0b0001;
        const RIGHT = 0b0010;
        const TOP = 0b0100;
        const BOTTOM = 0b1000;

        const TOP_LEFT = Self::TOP.bits() | Self::LEFT.bits();
        const TOP_RIGHT = Self::TOP.bits() | Self::RIGHT.bits();
        const BOTTOM_LEFT = Self::BOTTOM.bits() | Self::LEFT.bits();
        const BOTTOM_RIGHT = Self::BOTTOM.bits() | Self::RIGHT.bits();

        const INSIDE = Self::LEFT.bits() | Self::RIGHT.bits() | Self::TOP.bits() | Self::BOTTOM.bits();
    }
}

/// Returns an id whose generation bumps every time a new drag starts, so the first
/// `animate_value_with_time` call of a drag always starts from an un-animated, correct value.
///
/// `Context::animate_value_with_time` state is keyed globally and never cleared, so reusing a
/// fixed id across drag sessions lets a stale target from an unrelated previous drag leak in
/// e.g. resizing an edge shifts the widget's position without ever touching the *move* preview's
/// animation state, so the next move would otherwise animate in from that stale, pre-resize spot.
fn drag_session_id(ui: &Ui, base: Id, drag_started: bool) -> Id {
    let counter_id = base.with("_session");
    let mut generation: u64 = ui.mem().get_temp_or_insert(counter_id, 0);
    if drag_started {
        generation = generation.wrapping_add(1);
        ui.mem().insert_temp(counter_id, generation);
    }
    base.with(generation)
}

/// Animates a rect toward `target` by independently interpolating each edge.
///
/// egui has no built-in Rect-level animation, only `Context::animate_value_with_time` for a
/// single `f32`.
fn animate_rect(ctx: &egui::Context, id: Id, target: Rect, animation_time: f32) -> Rect {
    let left = ctx.animate_value_with_time(id.with("left"), target.min.x, animation_time);
    let top = ctx.animate_value_with_time(id.with("top"), target.min.y, animation_time);
    let right = ctx.animate_value_with_time(id.with("right"), target.max.x, animation_time);
    let bottom = ctx.animate_value_with_time(id.with("bottom"), target.max.y, animation_time);
    Rect::from_min_max(pos2(left, top), pos2(right, bottom))
}

/// Translates `rect` by the minimal amount needed to fit inside `bounds`.
fn clamp_rect_to(rect: Rect, bounds: Rect) -> Rect {
    let dx = (bounds.min.x - rect.min.x).max(0.) + (bounds.max.x - rect.max.x).min(0.);
    let dy = (bounds.min.y - rect.min.y).max(0.) + (bounds.max.y - rect.max.y).min(0.);
    rect.translate(vec2(dx, dy))
}

/// Compute which area of the widget the pointer is hovering on.
fn get_hit_region(rect: Rect, pointer_pos: Pos2) -> HitRegion {
    let inner_rect = rect.shrink(4.0);
    let outer_rect = rect.expand(4.0);

    // Check if the pointer is within the outer rect
    if !outer_rect.contains(pointer_pos) {
        return HitRegion::OUTSIDE;
    }

    // The pointer is at least within the outer rect here
    let Pos2 { x, y } = pointer_pos;
    let mut hit_region = HitRegion::empty();

    // Check which sides the pointer is *outside* of the inner rect (but inside the outer rect)
    // Horizontal sides
    if x < inner_rect.left() {
        hit_region |= HitRegion::LEFT;
    } else if x > inner_rect.right() {
        hit_region |= HitRegion::RIGHT;
    }
    // Vertical sides
    if y < inner_rect.top() {
        hit_region |= HitRegion::TOP;
    } else if y > inner_rect.bottom() {
        hit_region |= HitRegion::BOTTOM;
    }

    // If the pointer wasn't outside of any side, it's inside the inner rect
    if hit_region.is_empty() {
        HitRegion::INSIDE
    } else {
        hit_region
    }
}
