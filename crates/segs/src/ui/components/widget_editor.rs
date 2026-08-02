use bitflags::bitflags;
use egui::{Color32, CornerRadius, CursorIcon, Id, Pos2, Rect, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use segs_assets::icons;
use segs_ui::{style::CtxStyleExt, widgets::buttons::IconBtn};

use crate::ui::grid::{GRect, Grid};

const SNAP_ANIMATION_TIME: f32 = 0.150;
const SELECTION_TINT_ALPHA: u8 = 40;
const SELECTION_OUTLINE_STRENGTH: f32 = 0.75;
const HOVER_DARKEN_ALPHA: u8 = 40;
const REMOVE_BUTTON_SIZE: Vec2 = vec2(28., 28.);
const REMOVE_BUTTON_PADDING: f32 = 4.;

/// Replaces a color's alpha value.
fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

/// Draws the selected-widget tint and subdued outline.
pub fn show_selection(ui: &Ui, rect: Rect) {
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(1),
        with_alpha(ui.app_style().accent_fill, SELECTION_TINT_ALPHA),
    );

    let mut stroke = outline_stroke(ui);
    stroke.color = stroke.color.gamma_multiply(SELECTION_OUTLINE_STRENGTH);
    paint_outline(ui, rect, stroke);
}

/// Draws the widget hover tint.
pub fn show_hover(ui: &Ui, rect: Rect) {
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(1),
        with_alpha(Color32::BLACK, HOVER_DARKEN_ALPHA),
    );
}

/// Draws the widget edit outline.
pub fn show_outline(ui: &Ui, rect: Rect) {
    paint_outline(ui, rect, outline_stroke(ui));
}

/// Paints an outline without inheriting nested UI clipping.
fn paint_outline(ui: &Ui, rect: Rect, stroke: Stroke) {
    ui.ctx()
        .layer_painter(ui.layer_id())
        .rect_stroke(rect, 1., stroke, StrokeKind::Outside);
}

/// Draws the remove button and reports clicks.
pub fn show_remove_button(ui: &mut Ui, rect: Rect) -> bool {
    let button_rect = Rect::from_center_size(rect.center(), REMOVE_BUTTON_SIZE);
    let button = IconBtn::new(icons::Trash).with_padding(REMOVE_BUTTON_PADDING);
    ui.place(button_rect, button).clicked()
}

/// Sets the cursor for a widget hit region.
pub fn set_cursor(ui: &Ui, region: HitRegion, dragging: bool) {
    let cursor = match region {
        HitRegion::LEFT => CursorIcon::ResizeWest,
        HitRegion::RIGHT => CursorIcon::ResizeEast,
        HitRegion::TOP => CursorIcon::ResizeNorth,
        HitRegion::BOTTOM => CursorIcon::ResizeSouth,
        HitRegion::TOP_LEFT => CursorIcon::ResizeNorthWest,
        HitRegion::TOP_RIGHT => CursorIcon::ResizeNorthEast,
        HitRegion::BOTTOM_LEFT => CursorIcon::ResizeSouthWest,
        HitRegion::BOTTOM_RIGHT => CursorIcon::ResizeSouthEast,
        _ if dragging => CursorIcon::Grabbing,
        _ => CursorIcon::Grab,
    };
    ui.ctx().set_cursor_icon(cursor);
}

/// Resizes a rectangle from the selected edges.
pub fn resize_rect(mut rect: Rect, pointer: Pos2, direction: HitRegion, min_size: Vec2) -> Rect {
    let min_rect = rect.shrink2(min_size);

    if direction.contains(HitRegion::LEFT) {
        *rect.left_mut() = pointer.x.min(min_rect.right());
    }
    if direction.contains(HitRegion::RIGHT) {
        *rect.right_mut() = pointer.x.max(min_rect.left());
    }
    if direction.contains(HitRegion::TOP) {
        *rect.top_mut() = pointer.y.min(min_rect.bottom());
    }
    if direction.contains(HitRegion::BOTTOM) {
        *rect.bottom_mut() = pointer.y.max(min_rect.top());
    }

    rect
}

/// Draws the snapped target and returns its grid rectangle.
pub fn show_snap_preview(ui: &Ui, grid: &Grid, floating: Rect, animation_id: Id) -> GRect {
    let snapped = grid.to_grid_rect(floating);
    let target = grid.to_screen_rect(snapped);
    let animated_target = animate_rect(ui.ctx(), animation_id, target, SNAP_ANIMATION_TIME);

    show_outline(ui, animated_target);

    snapped
}

/// Moves a rectangle inside the given bounds.
pub fn clamp_rect_to(rect: Rect, bounds: Rect) -> Rect {
    let dx = (bounds.min.x - rect.min.x).max(0.) + (bounds.max.x - rect.max.x).min(0.);
    let dy = (bounds.min.y - rect.min.y).max(0.) + (bounds.max.y - rect.max.y).min(0.);
    rect.translate(vec2(dx, dy))
}

/// Finds the widget region under the pointer.
pub fn hit_region(rect: Rect, pointer: Pos2) -> HitRegion {
    let inner_rect = rect.shrink(4.0);
    let outer_rect = rect.expand(4.0);

    if !outer_rect.contains(pointer) {
        return HitRegion::OUTSIDE;
    }

    let Pos2 { x, y } = pointer;
    let mut hit_region = HitRegion::empty();

    if x < inner_rect.left() {
        hit_region |= HitRegion::LEFT;
    } else if x > inner_rect.right() {
        hit_region |= HitRegion::RIGHT;
    }
    if y < inner_rect.top() {
        hit_region |= HitRegion::TOP;
    } else if y > inner_rect.bottom() {
        hit_region |= HitRegion::BOTTOM;
    }

    if hit_region.is_empty() {
        HitRegion::INSIDE
    } else {
        hit_region
    }
}

/// Builds the widget editor outline stroke.
fn outline_stroke(ui: &Ui) -> Stroke {
    Stroke {
        width: 2.,
        color: ui.app_style().accent_fill.gamma_multiply(0.75),
    }
}

/// Animates a rectangle toward its target.
fn animate_rect(ctx: &egui::Context, id: Id, target: Rect, animation_time: f32) -> Rect {
    let left = ctx.animate_value_with_time(id.with("left"), target.min.x, animation_time);
    let top = ctx.animate_value_with_time(id.with("top"), target.min.y, animation_time);
    let right = ctx.animate_value_with_time(id.with("right"), target.max.x, animation_time);
    let bottom = ctx.animate_value_with_time(id.with("bottom"), target.max.y, animation_time);
    Rect::from_min_max(pos2(left, top), pos2(right, bottom))
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct HitRegion: u32 {
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
