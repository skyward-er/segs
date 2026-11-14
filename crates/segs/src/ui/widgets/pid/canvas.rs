//! Interactive diagram area of the P&ID editor: view navigation, selection, moving,
//! rotating, deleting and connecting symbols, and bending pipes.

use egui::{
    Align2, Color32, CursorIcon, Event, FontId, Key, Modifiers, MouseWheelUnit, Painter, Pos2, Rect, Response, Sense,
    Shape, Stroke, StrokeKind, Ui, Vec2, pos2, vec2,
};
use segs_ui::style::CtxStyleExt;

use super::{
    AnchorRef, Connection, ElementId, PidWidget, Symbol,
    paint::{self, View},
};
use crate::dataflow::store::DataStore;

/// Step element centers and bend points snap to, in diagram units.
const GRID_STEP: f32 = 1.0;
/// Grid dots closer than this on screen are thinned out, in points.
const MIN_GRID_SPACING: f32 = 16.0;
const GRID_DOT_RADIUS: f32 = 1.0;
/// Zoom limits, in screen points per diagram unit.
const MIN_SCALE: f32 = 0.2;
const MAX_SCALE: f32 = 2000.0;
/// Zoom factor for one mouse wheel notch.
const ZOOM_PER_LINE: f32 = 1.15;
/// Zoom change per scrolled point, for trackpads and pixel-precise wheels.
const ZOOM_PER_POINT: f32 = 0.003;
/// Zoom factor for a page scroll.
const ZOOM_PER_PAGE: f32 = 2.0;
/// Margin around the diagram when fitting, as a share of the canvas.
const FIT_MARGIN_RATIO: f32 = 0.1;
/// Zoom used for an empty diagram, in screen points per diagram unit.
const EMPTY_SCALE: f32 = 10.0;
/// Gap between a selected element and its outline, in points.
const SELECTION_PADDING: f32 = 4.0;
/// Radius of the anchor dots, in points.
const ANCHOR_RADIUS: f32 = 4.0;
/// How close the pointer must be to an anchor or bend point to grab it, in points.
const HANDLE_HIT_RADIUS: f32 = 8.0;
/// Half the side of a bend point handle, in points.
const BEND_HANDLE_HALF_SIZE: f32 = 4.0;
/// How close the pointer must be to a pipe to select it, in points.
const PIPE_HIT_DISTANCE: f32 = 6.0;
/// Offset of a duplicated element from the original, in diagram units.
const DUPLICATE_OFFSET: f32 = 2.0;
const HINT: &str = "Drag a symbol to move it · drag from a dot to connect · drag a pipe to bend it · drop a pipe on a pipe to branch it · drag a selected pipe's end to move it · \
                    R / Shift+R rotate · Ctrl+D duplicate · Del delete · Ctrl+Z undo · right-click for more · scroll to zoom";

/// Editor-only canvas state, never saved with the widget.
#[derive(Clone, Default)]
pub(super) struct Canvas {
    /// Current view, or `None` to fit the diagram on the next frame.
    camera: Option<Camera>,
    selection: Option<Selection>,
    drag: Option<Drag>,
}

/// Free view over the diagram.
#[derive(Clone, Copy)]
struct Camera {
    /// Diagram point shown at the canvas center.
    center: Pos2,
    /// Screen points per diagram unit.
    scale: f32,
}

/// What the editing commands act on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Selection {
    Element(ElementId),
    /// Index into the diagram's connections.
    Connection(usize),
}

/// Pointer drag in progress.
#[derive(Clone, Copy)]
enum Drag {
    /// Moving an element, holding it at `grab` from its center in diagram units.
    Element { id: ElementId, grab: Vec2 },
    /// Moving bend point `index` of pipe `connection`.
    BendPoint { connection: usize, index: usize },
    /// Moving one end of pipe `connection` to another anchor, applied where the drag is released.
    PipeEnd { connection: usize, end: PipeEnd },
    /// Drawing a pipe out of `from`, to be connected where the drag is released.
    Connect { from: AnchorRef },
    /// Moving the view.
    Pan,
}

/// Which end of a pipe.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PipeEnd {
    Start,
    End,
}

/// Edit applied to the selection.
#[derive(Clone, Copy)]
enum Command {
    RotateClockwise,
    RotateCounterClockwise,
    Duplicate,
    RemoveBendPoints,
    Delete,
}

impl Canvas {
    /// Fits the whole diagram into the canvas on the next frame.
    pub(super) fn fit(&mut self) {
        self.camera = None;
    }

    /// Returns the diagram point at the canvas center, snapped to the grid, or `None`
    /// before the canvas has been shown once.
    pub(super) fn center(&self) -> Option<Pos2> {
        self.camera.map(|camera| snap(camera.center))
    }

    /// Returns the selected element, if the selection is an element.
    pub(super) fn selected_element(&self) -> Option<ElementId> {
        match self.selection {
            Some(Selection::Element(id)) => Some(id),
            _ => None,
        }
    }

    pub(super) fn select_element(&mut self, id: ElementId) {
        self.selection = Some(Selection::Element(id));
    }

    /// Clears the selection, for when the editor deletes the selected element itself.
    pub(super) fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Handles navigation and editing input inside `rect`, then paints the grid, the diagram
    /// and the editing overlays.
    pub(super) fn show(&mut self, ui: &mut Ui, rect: Rect, diagram: &mut PidWidget, data_store: &DataStore) {
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        if self.camera.is_none() {
            self.camera = Some(fit_camera(diagram, rect));
        }

        // Apply this frame's input before painting so the picture never lags a frame
        self.handle_zoom(ui, &response, rect);
        self.handle_pointer(ui, &response, rect, diagram);
        let command = self.keyboard_command(ui).or_else(|| self.context_menu(&response));
        if let Some(command) = command {
            self.apply(command, diagram);
        }

        // Paint the grid under the diagram, then the editing overlays on top
        let view = self.view(rect);
        let painter = ui.painter_at(rect);
        let accent = ui.app_style().accent_fill;
        paint_grid(&painter, &view, ui.visuals().weak_text_color().gamma_multiply(0.5));
        paint::paint_diagram(diagram, ui, &painter, &view, data_store);
        self.paint_selection(ui, &painter, &view, diagram, accent);
        self.paint_anchors(ui, &response, &painter, &view, diagram, accent);
        painter.text(
            rect.left_bottom() + vec2(8.0, -8.0),
            Align2::LEFT_BOTTOM,
            HINT,
            FontId::proportional(12.0),
            ui.visuals().weak_text_color(),
        );
    }

    /// Zooms around the pointer with the mouse wheel or a pinch gesture.
    fn handle_zoom(&mut self, ui: &Ui, response: &Response, rect: Rect) {
        let Some(pointer) = pointer_pos(ui, response) else {
            return;
        };
        let factor = ui.input(|input| zoom_factor(&input.events));
        if factor == 1.0 {
            return;
        }

        // Keep the diagram point under the pointer fixed while the scale changes
        let anchor = self.view(rect).to_diagram(pointer);
        let camera = self.camera.as_mut().expect("camera is set before input is handled");
        camera.scale = (camera.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        camera.center = anchor - (pointer - rect.center()) / camera.scale;
    }

    /// Selects on click, and drags whatever is under the press: an anchor, a bend point,
    /// an element, a pipe or, failing all of those, the view.
    fn handle_pointer(&mut self, ui: &Ui, response: &Response, rect: Rect, diagram: &mut PidWidget) {
        let view = self.view(rect);

        // Decide what a new drag does from where the button was pressed
        if response.drag_started()
            && let Some(origin) = ui.input(|input| input.pointer.press_origin())
        {
            self.drag = Some(self.start_drag(diagram, &view, origin));
        }

        // Move the dragged item along the grid, or pan the view
        if response.dragged() {
            let pointer = response.interact_pointer_pos().map(|p| view.to_diagram(p));
            match (self.drag, pointer) {
                (Some(Drag::Element { id, grab }), Some(pointer)) => {
                    if let Some(element) = diagram.elements.iter_mut().find(|element| element.id == id) {
                        element.pos = snap(pointer + grab);
                    }
                }
                (Some(Drag::BendPoint { connection, index }), Some(pointer)) => {
                    if let Some(point) = diagram
                        .connections
                        .get_mut(connection)
                        .and_then(|connection| connection.waypoints.get_mut(index))
                    {
                        *point = snap(pointer);
                    }
                }
                (Some(Drag::Pan), _) => {
                    let camera = self.camera.as_mut().expect("camera is set before input is handled");
                    camera.center -= response.drag_delta() / camera.scale;
                }
                _ => {}
            }
        }

        // Reattach a dragged pipe end to the anchor it is dropped on
        if response.drag_stopped()
            && let Some(Drag::PipeEnd { connection, end }) = self.drag
            && let Some(target) = response
                .interact_pointer_pos()
                .and_then(|p| hit_anchor(diagram, &view, p))
        {
            reattach(diagram, connection, end, target);
        }

        // Finish a pipe on another element's anchor, or branch an existing pipe where it is
        // dropped, ignoring duplicates
        if response.drag_stopped() {
            if let Some(Drag::Connect { from }) = self.drag
                && let Some(pointer) = response.interact_pointer_pos()
                && let Some(to) = hit_anchor(diagram, &view, pointer).or_else(|| {
                    let (connection, segment) = hit_connection(diagram, &view, pointer)?;
                    self.selection = None;
                    Some(split_with_junction(
                        diagram,
                        connection,
                        segment,
                        snap(view.to_diagram(pointer)),
                    ))
                })
                && to.element != from.element
                && !diagram.connections.iter().any(|connection| {
                    (connection.start, connection.end) == (from, to) || (connection.start, connection.end) == (to, from)
                })
            {
                diagram.connect(from, to);
            }
            self.drag = None;
        }

        // A click without a drag selects what is under the pointer, or clears the selection
        if (response.clicked() || response.secondary_clicked())
            && let Some(pointer) = response.interact_pointer_pos()
        {
            // Junctions are tiny and sit on pipes, so they are picked by their dot before pipes
            self.selection = hit(diagram, view.to_diagram(pointer))
                .map(|(id, _)| id)
                .or_else(|| hit_junction(diagram, &view, pointer))
                .map(Selection::Element)
                .or_else(|| hit_connection(diagram, &view, pointer).map(|(index, _)| Selection::Connection(index)));
        }

        // A double click on a bend point removes it
        if response.double_clicked()
            && let Some(pointer) = response.interact_pointer_pos()
            && let Some((connection, index)) = self.hit_bend_point(diagram, &view, pointer)
        {
            diagram.connections[connection].waypoints.remove(index);
        }

        // Show what a press would grab
        let pointer = pointer_pos(ui, response);
        let over_handle = pointer.is_some_and(|p| {
            hit_anchor(diagram, &view, p).is_some() || self.hit_bend_point(diagram, &view, p).is_some()
        });
        let over_element = pointer.is_some_and(|p| hit(diagram, view.to_diagram(p)).is_some());
        let over_pipe = pointer.is_some_and(|p| hit_connection(diagram, &view, p).is_some());
        let cursor = match self.drag {
            Some(Drag::Element { .. } | Drag::BendPoint { .. }) => Some(CursorIcon::Grabbing),
            Some(Drag::Connect { .. } | Drag::PipeEnd { .. }) => Some(CursorIcon::Crosshair),
            Some(Drag::Pan) => Some(CursorIcon::AllScroll),
            None if over_handle => Some(CursorIcon::Crosshair),
            None if over_element || over_pipe => Some(CursorIcon::Grab),
            None => None,
        };
        if let Some(cursor) = cursor {
            ui.ctx().set_cursor_icon(cursor);
        }
    }

    /// Returns the drag starting at `origin`, updating the selection to what it grabs.
    fn start_drag(&mut self, diagram: &mut PidWidget, view: &View, origin: Pos2) -> Drag {
        // The ends of the selected pipe sit on anchors, so they are checked before anchors
        if let Some((connection, end)) = self.hit_pipe_end(diagram, view, origin) {
            return Drag::PipeEnd { connection, end };
        }

        // Anchors win over everything so pipes can start at any symbol edge, except junctions,
        // which are moved by dragging and receive pipes dropped onto them instead
        let in_diagram = view.to_diagram(origin);
        if let Some(anchor) = hit_anchor(diagram, view, origin) {
            match diagram.element(anchor.element) {
                Some(junction) if junction.symbol == Symbol::Junction => {
                    self.selection = Some(Selection::Element(anchor.element));
                    return Drag::Element {
                        id: anchor.element,
                        grab: junction.pos - in_diagram,
                    };
                }
                _ => return Drag::Connect { from: anchor },
            }
        }
        if let Some((connection, index)) = self.hit_bend_point(diagram, view, origin) {
            return Drag::BendPoint { connection, index };
        }
        if let Some((id, center)) = hit(diagram, in_diagram) {
            self.selection = Some(Selection::Element(id));
            return Drag::Element {
                id,
                grab: center - in_diagram,
            };
        }

        // Grabbing a pipe segment adds a bend point there and drags it
        if let Some((connection, segment)) = hit_connection(diagram, view, origin) {
            self.selection = Some(Selection::Connection(connection));
            diagram.connections[connection]
                .waypoints
                .insert(segment, snap(in_diagram));
            return Drag::BendPoint {
                connection,
                index: segment,
            };
        }
        Drag::Pan
    }

    /// Returns the end of the selected pipe within grabbing distance of `screen_point`.
    fn hit_pipe_end(&self, diagram: &PidWidget, view: &View, screen_point: Pos2) -> Option<(usize, PipeEnd)> {
        let Some(Selection::Connection(index)) = self.selection else {
            return None;
        };
        let connection = diagram.connections.get(index)?;
        [(connection.start, PipeEnd::Start), (connection.end, PipeEnd::End)]
            .into_iter()
            .find(|(anchor, _)| {
                diagram
                    .anchor_pos(*anchor)
                    .is_some_and(|pos| view.to_screen(pos).distance(screen_point) <= HANDLE_HIT_RADIUS)
            })
            .map(|(_, end)| (index, end))
    }

    /// Returns the bend point of the selected pipe within grabbing distance of `screen_point`,
    /// as (connection index, waypoint index).
    fn hit_bend_point(&self, diagram: &PidWidget, view: &View, screen_point: Pos2) -> Option<(usize, usize)> {
        let Some(Selection::Connection(connection)) = self.selection else {
            return None;
        };
        diagram
            .connections
            .get(connection)?
            .waypoints
            .iter()
            .position(|&point| view.to_screen(point).distance(screen_point) <= HANDLE_HIT_RADIUS)
            .map(|index| (connection, index))
    }

    /// Reads rotate and delete shortcuts for the selection, unless a text field has focus.
    fn keyboard_command(&self, ui: &Ui) -> Option<Command> {
        if self.selection.is_none() || ui.ctx().text_edit_focused() {
            return None;
        }
        ui.input_mut(|input| {
            if input.consume_key(Modifiers::COMMAND, Key::D) {
                Some(Command::Duplicate)
            } else if input.consume_key(Modifiers::SHIFT, Key::R) {
                Some(Command::RotateCounterClockwise)
            } else if input.consume_key(Modifiers::NONE, Key::R) {
                Some(Command::RotateClockwise)
            } else if input.consume_key(Modifiers::NONE, Key::Delete)
                || input.consume_key(Modifiers::NONE, Key::Backspace)
            {
                Some(Command::Delete)
            } else {
                None
            }
        })
    }

    /// Shows the right-click menu for the selection, returning the command picked from it.
    fn context_menu(&mut self, response: &Response) -> Option<Command> {
        let mut command = None;
        response.context_menu(|ui| {
            let items: &[(&str, Command)] = match self.selection {
                Some(Selection::Element(_)) => &[
                    ("Rotate clockwise\tR", Command::RotateClockwise),
                    ("Rotate counter-clockwise\tShift+R", Command::RotateCounterClockwise),
                    ("Duplicate\tCtrl+D", Command::Duplicate),
                    ("Delete\tDel", Command::Delete),
                ],
                Some(Selection::Connection(_)) => &[
                    ("Remove bend points", Command::RemoveBendPoints),
                    ("Delete pipe\tDel", Command::Delete),
                ],
                None => {
                    if ui.button("Fit view").clicked() {
                        self.fit();
                    }
                    return;
                }
            };
            for &(label, item) in items {
                if ui.button(label).clicked() {
                    command = Some(item);
                }
            }
        });
        command
    }

    /// Applies `command` to the selection, ignoring commands that do not fit it.
    fn apply(&mut self, command: Command, diagram: &mut PidWidget) {
        match (self.selection, command) {
            (Some(Selection::Element(id)), Command::Delete) => {
                diagram.remove_element(id);
                self.selection = None;
            }
            (Some(Selection::Element(id)), Command::RotateClockwise | Command::RotateCounterClockwise) => {
                if let Some(element) = diagram.elements.iter_mut().find(|element| element.id == id) {
                    element.rotation = match command {
                        Command::RotateClockwise => element.rotation.clockwise(),
                        _ => element.rotation.counter_clockwise(),
                    };
                }
            }
            (Some(Selection::Element(id)), Command::Duplicate) => {
                // Copy the symbol and orientation next to the original, and select the copy
                let Some(original) = diagram.element(id).cloned() else {
                    return;
                };
                let offset = Vec2::splat(DUPLICATE_OFFSET);
                let copy = diagram.add_element(snap(original.pos + offset), original.symbol);
                if let Some(element) = diagram.elements.iter_mut().find(|element| element.id == copy) {
                    element.rotation = original.rotation;
                }
                self.selection = Some(Selection::Element(copy));
            }
            (Some(Selection::Connection(index)), Command::Delete) => {
                diagram.remove_connection(index);
                self.selection = None;
            }
            (Some(Selection::Connection(index)), Command::RemoveBendPoints) => {
                if let Some(connection) = diagram.connections.get_mut(index) {
                    connection.waypoints.clear();
                }
            }
            _ => {}
        }
    }

    /// Outlines the selected element, or highlights the selected pipe with its bend points.
    fn paint_selection(&self, ui: &Ui, painter: &Painter, view: &View, diagram: &PidWidget, accent: Color32) {
        match self.selection {
            Some(Selection::Element(id)) => {
                if let Some(element) = diagram.element(id) {
                    let outline = view.rect_to_screen(element.bounds()).expand(SELECTION_PADDING);
                    painter.rect_stroke(outline, 2, Stroke::new(1.5, accent), StrokeKind::Outside);
                }
            }
            Some(Selection::Connection(index)) => {
                let Some(points) = diagram
                    .connections
                    .get(index)
                    .and_then(|connection| connection_points(diagram, view, connection))
                else {
                    return;
                };

                // Highlight the pipe under its end and bend point handles
                painter.add(Shape::line(
                    points.clone(),
                    Stroke::new(3.0, accent.gamma_multiply(0.6)),
                ));
                for end in [points[0], points[points.len() - 1]] {
                    painter.circle_filled(end, ANCHOR_RADIUS + 1.0, accent);
                }
                for point in &points[1..points.len() - 1] {
                    let handle = Rect::from_center_size(*point, Vec2::splat(BEND_HANDLE_HALF_SIZE * 2.0));
                    painter.rect(
                        handle,
                        1,
                        ui.visuals().extreme_bg_color,
                        Stroke::new(1.5, accent),
                        StrokeKind::Middle,
                    );
                }
            }
            None => {}
        }
    }

    /// Paints anchor dots on the hovered and selected elements, or on every element while a
    /// pipe is being drawn, plus the pipe preview.
    fn paint_anchors(
        &self,
        ui: &Ui,
        response: &Response,
        painter: &Painter,
        view: &View,
        diagram: &PidWidget,
        accent: Color32,
    ) {
        let pointer = pointer_pos(ui, response);
        let target = pointer.and_then(|p| hit_anchor(diagram, view, p));
        let hovered = pointer
            .and_then(|p| hit(diagram, view.to_diagram(p)))
            .map(|(id, _)| id)
            .or(target.map(|anchor| anchor.element));
        let connecting = matches!(self.drag, Some(Drag::Connect { .. } | Drag::PipeEnd { .. }));
        let selected = self.selected_element();

        // Dot every anchor that can be used right now, highlighting the one under the pointer
        for anchor in all_anchors(diagram) {
            let visible = connecting || Some(anchor.element) == hovered || Some(anchor.element) == selected;
            let Some(pos) = diagram.anchor_pos(anchor).filter(|_| visible) else {
                continue;
            };
            let fill = if Some(anchor) == target {
                accent
            } else {
                ui.visuals().extreme_bg_color
            };
            painter.circle(view.to_screen(pos), ANCHOR_RADIUS, fill, Stroke::new(1.5, accent));
        }

        // Preview the pipe from its fixed side to the pointer
        let fixed = match self.drag {
            Some(Drag::Connect { from }) => diagram.anchor_pos(from),
            Some(Drag::PipeEnd { connection, end }) => diagram.connections.get(connection).and_then(|connection| {
                // The moving end connects to its nearest bend point, or to the other end
                match end {
                    PipeEnd::Start => connection
                        .waypoints
                        .first()
                        .copied()
                        .or_else(|| diagram.anchor_pos(connection.end)),
                    PipeEnd::End => connection
                        .waypoints
                        .last()
                        .copied()
                        .or_else(|| diagram.anchor_pos(connection.start)),
                }
            }),
            _ => None,
        };
        if let (Some(fixed), Some(pointer)) = (fixed, pointer) {
            painter.line_segment([view.to_screen(fixed), pointer], Stroke::new(2.0, accent));
        }
    }

    fn view(&self, rect: Rect) -> View {
        let camera = self.camera.expect("camera is set before the canvas is used");
        View {
            scale: camera.scale,
            diagram_center: camera.center,
            target: rect,
        }
    }
}

/// Combines this frame's wheel and pinch events into one zoom factor.
///
/// Raw events are read instead of the smoothed scroll delta, which scroll areas
/// elsewhere in the window may consume before the canvas sees it.
fn zoom_factor(events: &[Event]) -> f32 {
    events
        .iter()
        .map(|event| match event {
            Event::MouseWheel { unit, delta, .. } => match unit {
                MouseWheelUnit::Line => ZOOM_PER_LINE.powf(delta.y),
                MouseWheelUnit::Point => (delta.y * ZOOM_PER_POINT).exp(),
                MouseWheelUnit::Page => ZOOM_PER_PAGE.powf(delta.y),
            },
            Event::Zoom(factor) => *factor,
            _ => 1.0,
        })
        .product()
}

/// Returns the pointer position while it is inside the canvas.
///
/// Uses the raw pointer position, since hover can be claimed by the modal around the canvas.
fn pointer_pos(ui: &Ui, response: &Response) -> Option<Pos2> {
    ui.input(|input| input.pointer.latest_pos())
        .filter(|_| response.contains_pointer())
}

/// Returns every anchor of every element in the diagram.
fn all_anchors(diagram: &PidWidget) -> impl Iterator<Item = AnchorRef> + '_ {
    diagram.elements.iter().flat_map(|element| {
        (0..element.symbol.anchors().len()).map(|anchor| AnchorRef {
            element: element.id,
            anchor,
        })
    })
}

/// Returns the anchor nearest to `screen_point` within grabbing distance.
fn hit_anchor(diagram: &PidWidget, view: &View, screen_point: Pos2) -> Option<AnchorRef> {
    all_anchors(diagram)
        .filter_map(|anchor| {
            let distance = view.to_screen(diagram.anchor_pos(anchor)?).distance(screen_point);
            (distance <= HANDLE_HIT_RADIUS).then_some((anchor, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(anchor, _)| anchor)
}

/// Returns the junction whose dot is within grabbing distance of `screen_point`.
fn hit_junction(diagram: &PidWidget, view: &View, screen_point: Pos2) -> Option<ElementId> {
    hit_anchor(diagram, view, screen_point)
        .map(|anchor| anchor.element)
        .filter(|&id| {
            diagram
                .element(id)
                .is_some_and(|element| element.symbol == Symbol::Junction)
        })
}

/// Returns the topmost element containing `point`, with its center.
fn hit(diagram: &PidWidget, point: Pos2) -> Option<(ElementId, Pos2)> {
    diagram
        .elements
        .iter()
        .rev()
        .find(|element| element.bounds().contains(point))
        .map(|element| (element.id, element.pos))
}

/// Returns the pipe nearest to `screen_point` within selecting distance, as
/// (connection index, segment index). Segment `i` ends where waypoint `i` would be inserted.
fn hit_connection(diagram: &PidWidget, view: &View, screen_point: Pos2) -> Option<(usize, usize)> {
    diagram
        .connections
        .iter()
        .enumerate()
        .filter_map(|(index, connection)| {
            let points = connection_points(diagram, view, connection)?;
            points
                .windows(2)
                .enumerate()
                .map(|(segment, ends)| (segment, segment_distance(screen_point, ends[0], ends[1])))
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(segment, distance)| (index, segment, distance))
        })
        .filter(|&(_, _, distance)| distance <= PIPE_HIT_DISTANCE)
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(index, segment, _)| (index, segment))
}

/// Moves `end` of pipe `connection` onto `target`, unless that would join an element to itself
/// or duplicate another pipe.
fn reattach(diagram: &mut PidWidget, connection: usize, end: PipeEnd, target: AnchorRef) {
    let Some(pipe) = diagram.connections.get(connection) else {
        return;
    };
    let (start, finish) = match end {
        PipeEnd::Start => (target, pipe.end),
        PipeEnd::End => (pipe.start, target),
    };
    let duplicate = diagram.connections.iter().enumerate().any(|(index, other)| {
        index != connection
            && ((other.start, other.end) == (start, finish) || (other.start, other.end) == (finish, start))
    });
    if start.element == finish.element || duplicate {
        return;
    }
    let pipe = &mut diagram.connections[connection];
    let previous = match end {
        PipeEnd::Start => pipe.start.element,
        PipeEnd::End => pipe.end.element,
    };
    pipe.start = start;
    pipe.end = finish;

    // A junction the pipe left may no longer be needed
    diagram.dissolve_junction(previous);
}

/// Splits pipe `connection` at `point` on its segment `segment` with a new junction, keeping
/// each half's bend points.
///
/// Returns the junction's anchor, ready for another pipe to connect to.
fn split_with_junction(diagram: &mut PidWidget, connection: usize, segment: usize, point: Pos2) -> AnchorRef {
    let junction = AnchorRef {
        element: diagram.add_element(point, Symbol::Junction),
        anchor: 0,
    };
    let original = diagram.connections.remove(connection);
    let (before, after) = original.waypoints.split_at(segment);
    diagram.connections.push(Connection {
        start: original.start,
        end: junction,
        waypoints: before.to_vec(),
    });
    diagram.connections.push(Connection {
        start: junction,
        end: original.end,
        waypoints: after.to_vec(),
    });
    junction
}

/// Returns a pipe's screen points from its start anchor through its bend points to its end
/// anchor, or `None` if either end no longer exists.
fn connection_points(diagram: &PidWidget, view: &View, connection: &Connection) -> Option<Vec<Pos2>> {
    let start = diagram.anchor_pos(connection.start)?;
    let end = diagram.anchor_pos(connection.end)?;
    Some(
        std::iter::once(start)
            .chain(connection.waypoints.iter().copied())
            .chain(std::iter::once(end))
            .map(|point| view.to_screen(point))
            .collect(),
    )
}

/// Returns the distance from `point` to the segment between `a` and `b`.
fn segment_distance(point: Pos2, a: Pos2, b: Pos2) -> f32 {
    let along = b - a;
    let t = ((point - a).dot(along) / along.length_sq().max(f32::EPSILON)).clamp(0.0, 1.0);
    point.distance(a + along * t)
}

/// Rounds a diagram point to the nearest grid step.
fn snap(point: Pos2) -> Pos2 {
    pos2(
        (point.x / GRID_STEP).round() * GRID_STEP,
        (point.y / GRID_STEP).round() * GRID_STEP,
    )
}

/// Returns a view showing the whole diagram, or a default zoom around the origin when empty.
fn fit_camera(diagram: &PidWidget, rect: Rect) -> Camera {
    match paint::diagram_bounds(diagram) {
        Some(bounds) => {
            let view = View::fit(bounds, rect.shrink(rect.size().min_elem() * FIT_MARGIN_RATIO));
            Camera {
                center: view.diagram_center,
                scale: view.scale.clamp(MIN_SCALE, MAX_SCALE),
            }
        }
        None => Camera {
            center: Pos2::ZERO,
            scale: EMPTY_SCALE,
        },
    }
}

/// Paints a dot at every visible grid point, thinned out when zoomed far away.
fn paint_grid(painter: &Painter, view: &View, color: Color32) {
    let mut step = GRID_STEP;
    while step * view.scale < MIN_GRID_SPACING {
        step *= 5.0;
    }

    // Walk the grid points between the visible corners
    let min = view.to_diagram(view.target.min);
    let max = view.to_diagram(view.target.max);
    let (first_x, last_x) = ((min.x / step).ceil() as i32, (max.x / step).floor() as i32);
    let (first_y, last_y) = ((min.y / step).ceil() as i32, (max.y / step).floor() as i32);
    for x in first_x..=last_x {
        for y in first_y..=last_y {
            let point = pos2(x as f32 * step, y as f32 * step);
            painter.circle_filled(view.to_screen(point), GRID_DOT_RADIUS, color);
        }
    }
}
