use std::marker::PhantomData;

use egui::{
    Align, Color32, CursorIcon, Frame, Id, Layout, Pos2, Rect, Response, Sense, Stroke, Ui, UiBuilder, Vec2,
    emath::GuiRounding, vec2,
};
use segs_memory::MemoryExt;
use serde::{Deserialize, Serialize};

pub struct ResizablePanel<'a, D: DirectionTrait> {
    /// The direction in which the panel can be resized. This determines how the
    /// sides are arranged and how the resizing behavior works. For example:
    /// - Horizontal: The panel is split into left and right sides, and the
    ///   separator allows resizing horizontally.
    /// - Vertical: The panel is split into top and bottom sides, and the
    ///   separator allows resizing vertically.
    direction: D,

    /// The side of the panel that can be contracted and resized. Depending on
    /// the direction:
    /// - Horizontal: Left or Right
    /// - Vertical: Top or Bottom
    panel_side: Alignment,

    /// Whether the panel is currently collapsed. If `true`, the panel is
    /// hidden. This is used to allow the user to collapse the panel by
    /// dragging the separator all the way to the edge if the `collapsible`
    /// option is enabled.
    collapsed: Option<&'a mut bool>,

    /// The unique identifier for the panel, used for storing state in the UI
    /// memory.
    id: Option<Id>,

    /// The minimum size of the panel in pixels. This is used to prevent the
    /// panel from being resized too small, which could make it unusable or
    /// cause layout issues.
    minimum_size: f32,

    /// The maximum size of the panel in pixels. This is used to prevent the
    /// panel from being resized too large, which could cause layout issues or
    /// make other panels unusable.
    maximum_size: f32,

    /// Depending on the direction:
    /// - Horizontal: [left panel, right panel]
    /// - Vertical: [top panel, bottom panel]
    sides: [Panel; 2],

    /// The style configuration for the container.
    style: ContainerStyle,
}

impl ResizablePanel<'_, HorizontalDirection> {
    pub fn horizontal_left() -> Self {
        Self {
            direction: HorizontalDirection,
            panel_side: Alignment::LeftTop,
            collapsed: None,
            id: None,
            minimum_size: 100.,
            maximum_size: 300.,
            sides: Default::default(),
            style: Default::default(),
        }
    }

    pub fn horizontal_right() -> Self {
        Self {
            panel_side: Alignment::RightBottom,
            ..Self::horizontal_left()
        }
    }

    pub fn left_frame(mut self, frame: Frame) -> Self {
        match self.panel_side {
            Alignment::LeftTop => self.sides[0].frame = frame,
            Alignment::RightBottom => self.sides[1].frame = frame,
        }
        self
    }

    pub fn right_frame(mut self, frame: Frame) -> Self {
        match self.panel_side {
            Alignment::LeftTop => self.sides[1].frame = frame,
            Alignment::RightBottom => self.sides[0].frame = frame,
        }
        self
    }
}

impl ResizablePanel<'_, VerticalDirection> {
    pub fn vertical_top() -> Self {
        Self {
            direction: VerticalDirection,
            panel_side: Alignment::LeftTop,
            collapsed: None,
            id: None,
            minimum_size: 100.,
            maximum_size: 300.,
            sides: Default::default(),
            style: Default::default(),
        }
    }

    pub fn vertical_bottom() -> Self {
        Self {
            panel_side: Alignment::RightBottom,
            ..Self::vertical_top()
        }
    }

    pub fn top_frame(mut self, frame: Frame) -> Self {
        match self.panel_side {
            Alignment::LeftTop => self.sides[0].frame = frame,
            Alignment::RightBottom => self.sides[1].frame = frame,
        }
        self
    }

    pub fn bottom_frame(mut self, frame: Frame) -> Self {
        match self.panel_side {
            Alignment::LeftTop => self.sides[1].frame = frame,
            Alignment::RightBottom => self.sides[0].frame = frame,
        }
        self
    }
}

impl<'a, D: DirectionTrait> ResizablePanel<'a, D> {
    pub fn collapsed(mut self, collapsed: &'a mut bool) -> Self {
        self.collapsed = Some(collapsed);
        self
    }

    pub fn set_minimum_size(mut self, minimum_size: f32) -> Self {
        self.minimum_size = minimum_size;
        self
    }

    pub fn set_maximum_size(mut self, maximum_size: f32) -> Self {
        self.maximum_size = maximum_size;
        self
    }

    pub fn inactive_separator_stroke(mut self, stroke: Stroke) -> Self {
        self.style.separator_inactive.color = stroke.color;
        self.style.separator_inactive.width = stroke.width;
        self
    }

    pub fn inactive_separator_width(mut self, width: f32) -> Self {
        self.style.separator_inactive.width = width;
        self
    }

    pub fn animate(mut self, animate: bool) -> Self {
        self.style.animate = animate;
        self
    }

    pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut PanelUI<D>)) {
        let Self {
            direction,
            panel_side: align,
            collapsed,
            id,
            minimum_size,
            maximum_size,
            sides,
            style,
        } = self;
        let dir = direction.to_direction();
        let side = dir.side(align);

        // We use the maximum available size to fit the available space to the panel.
        let max_size = ui.max_rect().size();
        let layout = side.get_layout();
        ui.allocate_ui_with_layout(max_size, layout, |ui| {
            // We set the item spacing to zero to ensure that there is no gap between the
            // panels and the separator.
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let id = id.unwrap_or_else(|| ui.id().with("resizable_panel"));

            let default_pers = PersistentPanelState::new(minimum_size);
            let pstate = ui.ctx().mem().get_perm_or_insert(id, default_pers.clone());
            let default_temp = TemporaryPanelState::new(pstate.separator_pos);
            let tstate = ui.ctx().mem().get_temp_or_insert(id, default_temp.clone());

            // Determine the current separator interpolating for animation if enabled.
            let separator_pos = if style.animate && tstate.separator_pos != 0.0 {
                let id = id.with("separator_animation");
                ui.ctx().animate_value_with_time(id, tstate.separator_pos, 0.05)
            } else {
                tstate.separator_pos
            };

            if collapsed.as_ref().is_some_and(|v| **v) {
                let (rect_second, _) = ui.allocate_exact_size(max_size, Sense::empty());

                // Show children UIs.
                let mut panel_ui = PanelUI::new(align, sides, ui, None, rect_second);
                add_contents(&mut panel_ui);
            } else {
                let (max_main, max_cross) = (dir.vec_main(max_size), dir.vec_cross(max_size));
                let side_first_size = dir.side_vec2(separator_pos, max_cross);
                let side_second_size = dir.side_vec2(max_main - separator_pos, max_cross);
                let separator_size = dir.side_vec2(style.separator_inactive.width, max_cross);

                let (rect_first, _) = ui.allocate_exact_size(side_first_size, Sense::empty());
                let (rect_sep, response_sep) = ui.allocate_exact_size(separator_size, Sense::drag());
                let (rect_second, _) = ui.allocate_exact_size(side_second_size, Sense::empty());

                // Show children UIs.
                let mut panel_ui = PanelUI::new(align, sides, ui, Some(rect_first), rect_second);
                add_contents(&mut panel_ui);

                // Handle the separator UI to paint it on top.
                Separator {
                    side,
                    collapsed,
                    persistent_state: pstate.clone(),
                    temporary_state: tstate.clone(),
                    id,
                    rect: rect_sep,
                    response: response_sep,
                    minimum_size,
                    maximum_size,
                    style,
                }
                .show(ui);
            }
        });
    }
}

struct Separator<'a> {
    side: Side,
    collapsed: Option<&'a mut bool>,

    // ~ States ~
    persistent_state: PersistentPanelState,
    temporary_state: TemporaryPanelState,

    // ~ Egui ~
    id: Id,
    rect: Rect,
    response: Response,

    // ~ Bounds ~
    minimum_size: f32,
    maximum_size: f32,

    // ~ Style ~
    style: ContainerStyle,
}

impl Separator<'_> {
    fn show(self, ui: &mut Ui) {
        let Separator {
            side,
            mut collapsed,
            persistent_state: pstate,
            temporary_state: tstate,
            id,
            rect,
            mut response,
            minimum_size,
            maximum_size,
            style,
        } = self;

        let max_rect = ui.max_rect();
        let dir = side.direction;

        let painter = ui.painter();
        // Show the right cursor icon based on the interaction state of the separator.
        response = response.on_hover_cursor(dir.resize_cursor_icon());

        // Handle the dragging of the separator to resize the panels
        if response.dragged() {
            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let pointer_dist = side.dist_from_panel_edge(pointer_pos, max_rect);
                let mem = ui.ctx().mem();
                if pointer_dist < minimum_size {
                    // If collapsible, allow the panel to be fully hidden by dragging the separator
                    // all the way to the edge. Otherwise, clamp it to the minimum size.
                    if let Some(collapsed) = collapsed.as_mut()
                        && pointer_dist < minimum_size * 0.4
                    {
                        **collapsed = true;
                        response = response.on_hover_cursor(CursorIcon::Grabbing);
                    } else {
                        response = response.on_hover_cursor(side.minimum_resized_cursor_icon());
                        // Clamp the current position to the minimum
                        mem.insert_temp(id, TemporaryPanelState::new(minimum_size));
                    }
                } else if pointer_dist > maximum_size {
                    response = response.on_hover_cursor(side.maximum_resized_cursor_icon());
                    // Clamp the current position to the maximum
                    mem.insert_temp(id, TemporaryPanelState::new(maximum_size));
                } else {
                    let drag = dir.vec_main(response.total_drag_delta().unwrap_or_default());
                    // Replace the temporary state with the new separator position, clamping it to
                    // the minimum and maximum sizes.
                    let new_main =
                        (pstate.separator_pos + side.align.correct_drag(drag)).clamp(minimum_size, maximum_size);
                    mem.insert_temp(id, TemporaryPanelState::new(new_main));
                }
            }
        } else {
            ui.ctx()
                .mem()
                .insert_perm(id, PersistentPanelState::new(tstate.separator_pos));
        }

        // Paint the separator and animate it
        let is_active = response.dragged() || response.hovered();
        let active_t = if style.animate {
            let id = id.with("_active_animation");
            ui.ctx().animate_bool_with_time(id, is_active, 0.1)
        } else if is_active {
            1.
        } else {
            0.
        };
        let color = style
            .separator_inactive
            .color
            .lerp_to_gamma(style.separator_active.color, active_t);
        let sep_size = dir.side_vec2(style.separator_active.width, dir.vec_cross(max_rect.size()));
        let active_rect = Rect::from_center_size(rect.center(), sep_size).round_to_pixels(ui.pixels_per_point());
        let rect = rect.lerp_towards(&active_rect, active_t);
        painter.rect_filled(rect, 0., color);
    }
}

pub struct PanelUI<'a, D: DirectionTrait> {
    align: Alignment,
    sides: [Panel; 2],

    // ~ Egui ~
    ui: &'a mut Ui,
    panel_rect: Option<Rect>,
    main_rect: Rect,

    _marker: PhantomData<D>,
}

impl<'a, D: DirectionTrait> PanelUI<'a, D> {
    fn new(align: Alignment, sides: [Panel; 2], ui: &'a mut Ui, panel_rect: Option<Rect>, main_rect: Rect) -> Self {
        Self {
            align,
            sides,
            ui,
            panel_rect,
            main_rect,
            _marker: PhantomData,
        }
    }

    fn show_first(&mut self, add_contents: impl FnOnce(&mut Ui)) -> &mut Self {
        if self.align.is_inverse() {
            // Left is panel, right is main
            self.show_pane(1, Some(self.main_rect), add_contents)
        } else {
            // Left is main, right is panel
            self.show_pane(0, self.panel_rect, add_contents)
        }
    }

    fn show_second(&mut self, add_contents: impl FnOnce(&mut Ui)) -> &mut Self {
        if self.align.is_inverse() {
            self.show_pane(0, self.panel_rect, add_contents)
        } else {
            self.show_pane(1, Some(self.main_rect), add_contents)
        }
    }

    fn show_pane(&mut self, pane_index: usize, rect: Option<Rect>, add_contents: impl FnOnce(&mut Ui)) -> &mut Self {
        // Early exit if this ui should not be painted
        let Some(rect) = rect else {
            return self;
        };

        let Panel { frame } = self.sides[pane_index];
        self.ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            frame.show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                add_contents(ui);
            })
        });
        self
    }
}

impl<'a> PanelUI<'a, HorizontalDirection> {
    pub fn show_left(&mut self, add_contents: impl FnOnce(&mut Ui)) -> &mut Self {
        self.show_first(add_contents)
    }

    pub fn show_right(&mut self, add_contents: impl FnOnce(&mut Ui)) -> &mut Self {
        self.show_second(add_contents)
    }
}

impl<'a> PanelUI<'a, VerticalDirection> {
    pub fn show_top(&mut self, add_contents: impl FnOnce(&mut Ui)) -> &mut Self {
        self.show_first(add_contents)
    }

    pub fn show_bottom(&mut self, add_contents: impl FnOnce(&mut Ui)) -> &mut Self {
        self.show_second(add_contents)
    }
}

#[derive(Debug, Clone)]
struct TemporaryPanelState {
    separator_pos: f32,
}

impl TemporaryPanelState {
    fn new(separator_pos: f32) -> Self {
        Self { separator_pos }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PersistentPanelState {
    separator_pos: f32,
}

impl PersistentPanelState {
    fn new(separator_pos: f32) -> Self {
        Self { separator_pos }
    }
}

struct Panel {
    frame: Frame,
}

impl Default for Panel {
    fn default() -> Self {
        Self { frame: Frame::NONE }
    }
}

#[derive(Debug, Clone, Copy)]
enum Alignment {
    LeftTop,
    RightBottom,
}

impl Alignment {
    /// Corrects the drag delta based on the panel side. For example, if the
    /// panel is on the right or bottom side, the drag should be inverted to
    /// move in the correct direction.
    fn correct_drag(&self, drag: f32) -> f32 {
        if self.is_inverse() { -drag } else { drag }
    }

    fn is_inverse(&self) -> bool {
        matches!(self, Alignment::RightBottom)
    }
}

struct ContainerStyle {
    animate: bool,

    // - Separator styles -
    separator_inactive: Stroke,
    separator_active: Stroke,
}

impl Default for ContainerStyle {
    fn default() -> Self {
        Self {
            animate: true,
            separator_inactive: Stroke {
                width: 1.0,
                color: Color32::from_rgb(242, 242, 242),
            },
            separator_active: Stroke {
                width: 2.0,
                color: Color32::from_rgb(152, 152, 153),
            },
        }
    }
}

pub struct HorizontalDirection;
pub struct VerticalDirection;

pub trait DirectionTrait {
    fn to_direction(&self) -> Direction;
}

impl DirectionTrait for HorizontalDirection {
    fn to_direction(&self) -> Direction {
        Direction::Horizontal
    }
}

impl DirectionTrait for VerticalDirection {
    fn to_direction(&self) -> Direction {
        Direction::Vertical
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    fn side(self, align: Alignment) -> Side {
        Side { direction: self, align }
    }

    /// Gets the appropriate cursor icon for resizing based on the current
    /// direction.
    fn resize_cursor_icon(&self) -> CursorIcon {
        match self {
            Direction::Horizontal => CursorIcon::ResizeHorizontal,
            Direction::Vertical => CursorIcon::ResizeVertical,
        }
    }

    /// Gets the main axis position from a 2D position based on the current
    /// direction. For example, if the direction is horizontal, it returns the x
    /// coordinate, and if it's vertical, it returns the y coordinate.
    fn pos_main(&self, pos: Pos2) -> f32 {
        match self {
            Direction::Horizontal => pos.x,
            Direction::Vertical => pos.y,
        }
    }

    /// Gets the cross axis position from a 2D position based on the current
    /// direction. For example, if the direction is horizontal, it returns the y
    /// coordinate, and if it's vertical, it returns the x coordinate.
    fn vec_main(&self, vec: Vec2) -> f32 {
        match self {
            Direction::Horizontal => vec.x,
            Direction::Vertical => vec.y,
        }
    }

    /// Gets the cross axis position from a 2D position based on the current
    /// direction. For example, if the direction is horizontal, it returns the y
    /// coordinate, and if it's vertical, it returns the x coordinate.
    fn vec_cross(&self, vec: Vec2) -> f32 {
        match self {
            Direction::Horizontal => vec.y,
            Direction::Vertical => vec.x,
        }
    }

    /// Creates a 2D vector from the main and cross axis values based on the
    /// current direction. For example, if the direction is horizontal, it
    /// returns Vec2(main_axis, cross_axis), and if it's vertical, it returns
    /// Vec2(cross_axis, main_axis).
    fn side_vec2(&self, main_axis: f32, cross_axis: f32) -> Vec2 {
        match self {
            Direction::Horizontal => vec2(main_axis, cross_axis),
            Direction::Vertical => vec2(cross_axis, main_axis),
        }
    }
}

struct Side {
    direction: Direction,
    align: Alignment,
}

impl Side {
    /// Calculates the distance from the pointer position to the panel edge
    /// based on the current direction and panel side. This is used to determine
    /// how close the pointer is to the edge for resizing purposes.
    fn dist_from_panel_edge(&self, pointer_pos: Pos2, panel_rect: Rect) -> f32 {
        let panel_edge_pos = self.panel_edge_pos(panel_rect);
        (self.direction.pos_main(pointer_pos) - panel_edge_pos).abs()
    }

    /// Gets the position of the panel edge based on the current direction and
    /// panel side. This is used to determine where the separator is located for
    /// resizing purposes.
    fn panel_edge_pos(&self, panel_rect: Rect) -> f32 {
        match (self.direction, self.align) {
            (Direction::Horizontal, Alignment::LeftTop) => panel_rect.left(),
            (Direction::Horizontal, Alignment::RightBottom) => panel_rect.right(),
            (Direction::Vertical, Alignment::LeftTop) => panel_rect.top(),
            (Direction::Vertical, Alignment::RightBottom) => panel_rect.bottom(),
        }
    }

    /// Gets the layout for the panels based on the current direction and panel
    /// side. This determines how the panels are arranged and aligned within the
    /// container.
    fn get_layout(&self) -> Layout {
        match (self.direction, self.align) {
            (Direction::Horizontal, Alignment::LeftTop) => Layout::left_to_right(Align::Min),
            (Direction::Horizontal, Alignment::RightBottom) => Layout::right_to_left(Align::Min),
            (Direction::Vertical, Alignment::LeftTop) => Layout::top_down(Align::Min),
            (Direction::Vertical, Alignment::RightBottom) => Layout::bottom_up(Align::Min),
        }
    }

    /// Gets the appropriate cursor icon for resizing to the minimum size based
    /// on the current direction and panel side. This is used to indicate to the
    /// user that they are trying to resize beyond the minimum limit.
    fn minimum_resized_cursor_icon(&self) -> CursorIcon {
        match (self.direction, self.align) {
            (Direction::Horizontal, Alignment::LeftTop) => CursorIcon::ResizeEast,
            (Direction::Horizontal, Alignment::RightBottom) => CursorIcon::ResizeWest,
            (Direction::Vertical, Alignment::LeftTop) => CursorIcon::ResizeSouth,
            (Direction::Vertical, Alignment::RightBottom) => CursorIcon::ResizeNorth,
        }
    }

    /// Gets the appropriate cursor icon for resizing to the maximum size based
    /// on the current direction and panel side. This is used to indicate to the
    /// user that they are trying to resize beyond the maximum limit.
    fn maximum_resized_cursor_icon(&self) -> CursorIcon {
        match (self.direction, self.align) {
            (Direction::Horizontal, Alignment::LeftTop) => CursorIcon::ResizeWest,
            (Direction::Horizontal, Alignment::RightBottom) => CursorIcon::ResizeEast,
            (Direction::Vertical, Alignment::LeftTop) => CursorIcon::ResizeNorth,
            (Direction::Vertical, Alignment::RightBottom) => CursorIcon::ResizeSouth,
        }
    }
}
