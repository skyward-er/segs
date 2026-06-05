use std::cell;

use bitflags::bitflags;
use egui::{CursorIcon, Pos2, Rect, Response, Stroke, StrokeKind, Ui};
use segs_memory::MemoryExt;
use segs_ui::style::CtxStyleExt;

use crate::ui::{grid::Grid, widgets::WidgetData};

pub struct WidgetEditor<'a> {
    grid: &'a Grid,
    widget: &'a mut WidgetData,
    response: Response,
}

impl<'a> WidgetEditor<'a> {
    pub fn new(grid: &'a Grid, widget: &'a mut WidgetData, response: Response) -> Self {
        Self { grid, widget, response }
    }

    pub fn show(self, ui: &mut Ui) {
        let Self {
            grid,
            widget,
            response: res,
        } = self;

        let rect = res.rect;
        let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) else {
            return;
        };

        // Draw the indicator stroke
        let stroke = Stroke {
            width: 2.,
            color: ui.app_style().accent_fill,
        };
        let painter = ui.painter();
        painter.rect_stroke(rect, 1., stroke, StrokeKind::Middle);

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
            // Widget is only being hovered, nothing more to do
            return;
        }

        // Handle drag interaction
        match active_hit_region {
            HitRegion::INSIDE | HitRegion::OUTSIDE => {
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);

                // Compute the new position
                let new_rect = Rect::from_center_size(pointer_pos, rect.size());
                // Transform and apply the new position
                widget.grect = grid.to_grid_rect(new_rect);
            }
            _ => {
                let direction = active_hit_region;
                let mut widget_rect = rect;
                // Minimum rect the widget can be resized to, to avoid zero-sized widgets
                let min_rect = rect.shrink2(grid.cell_size);

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

                // Transform and apply the new size
                widget.grect = grid.to_grid_rect(widget_rect);
                println!("rect: {widget_rect:?}, grect: {:?}", widget.grect);
            }
        }
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
