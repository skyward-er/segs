//! Rendering of a P&ID, either fitted to the widget or through an editor view.

use std::f32::consts::PI;

use egui::{Align, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, Ui, Vec2, pos2};
use segs_ui::style::CtxStyleExt;

use super::{Element, IconKind, MotorValveKind, PidWidget, Symbol, ValveState, value_text};
use crate::{dataflow::store::DataStore, ui::components::auto_size};

/// Share of the widget left empty around the diagram.
const MARGIN_RATIO: f32 = 0.05;
/// Line width of the v1 artwork, in diagram units.
const STROKE_WIDTH: f32 = 0.2;
/// Line width of the bold strokes in the v1 artwork, in diagram units.
const THICK_STROKE_WIDTH: f32 = 0.4;
/// Segments used to approximate a half circle.
const ARC_SEGMENTS: usize = 16;

/// Paints the whole diagram, scaled and centered to fit the widget rect.
pub fn show(widget: &PidWidget, ui: &Ui, data_store: &DataStore) {
    let rect = ui.max_rect();
    let painter = ui.painter_at(rect);

    // Fit the bounding box of every element and bend point into the widget
    let Some(bounds) = diagram_bounds(widget) else {
        paint_fitted_text(
            ui,
            &painter,
            rect.shrink(rect.height() * 0.3),
            "Empty diagram",
            ui.visuals().weak_text_color(),
            false,
        );
        return;
    };
    let view = View::fit(bounds, rect.shrink(rect.size().min_elem() * MARGIN_RATIO));
    paint_diagram(widget, ui, &painter, &view, data_store);
}

/// Paints pipes and symbols through `view`, with live state taken from the data store.
pub(super) fn paint_diagram(widget: &PidWidget, ui: &Ui, painter: &Painter, view: &View, data_store: &DataStore) {
    let line_color = ui.visuals().text_color();

    // Draw pipes first so symbols paint over their ends
    let pipe_stroke = Stroke::new(view.stroke_width(STROKE_WIDTH), line_color);
    for connection in &widget.connections {
        let (Some(start), Some(end)) = (widget.anchor_pos(connection.start), widget.anchor_pos(connection.end)) else {
            continue; // Skip pipes whose element or anchor no longer exists
        };
        let points = std::iter::once(start)
            .chain(connection.waypoints.iter().copied())
            .chain(std::iter::once(end))
            .map(|point| view.to_screen(point))
            .collect();
        painter.add(Shape::line(points, pipe_stroke));
    }

    // Draw each symbol with live state taken from the data store
    for element in &widget.elements {
        let pen = Pen {
            painter,
            element,
            view,
            color: line_color,
        };
        match &element.symbol {
            Symbol::Icon(icon) => paint_icon(&pen, *icon),
            Symbol::MotorValve { kind, stream } => {
                let state = kind.state(*stream, data_store);
                paint_motor_valve(
                    &pen,
                    *kind,
                    state,
                    ui.app_style().success_fill,
                    ui.app_style().error_fill,
                );
            }
            Symbol::ValueDisplay { stream, unit, .. } => {
                let text = value_text(*stream, unit, data_store);
                paint_fitted_text(
                    ui,
                    painter,
                    view.rect_to_screen(element.bounds()),
                    &text,
                    line_color,
                    true,
                );
            }
            Symbol::Label { text, .. } => {
                paint_fitted_text(
                    ui,
                    painter,
                    view.rect_to_screen(element.bounds()),
                    text,
                    line_color,
                    false,
                );
            }
        }
    }
}

/// Mapping from diagram space to screen space with a uniform scale.
pub(super) struct View {
    /// Screen points per diagram unit.
    pub(super) scale: f32,
    /// Diagram point drawn at the center of `target`.
    pub(super) diagram_center: Pos2,
    /// Screen area the diagram is drawn into.
    pub(super) target: Rect,
}

impl View {
    /// Returns the largest uniform scale that keeps `bounds` inside `target`, centered.
    pub(super) fn fit(bounds: Rect, target: Rect) -> Self {
        let size = bounds.size().max(Vec2::splat(f32::EPSILON));
        Self {
            scale: (target.width() / size.x).min(target.height() / size.y),
            diagram_center: bounds.center(),
            target,
        }
    }

    pub(super) fn to_screen(&self, point: Pos2) -> Pos2 {
        self.target.center() + (point - self.diagram_center) * self.scale
    }

    /// Inverse of [`Self::to_screen`].
    pub(super) fn to_diagram(&self, point: Pos2) -> Pos2 {
        self.diagram_center + (point - self.target.center()) / self.scale
    }

    pub(super) fn rect_to_screen(&self, rect: Rect) -> Rect {
        Rect::from_min_max(self.to_screen(rect.min), self.to_screen(rect.max))
    }

    /// Converts a diagram line width to screen points, never thinner than one point.
    fn stroke_width(&self, width: f32) -> f32 {
        (width * self.scale).max(1.0)
    }
}

/// Draws one element's artwork, taking points in symbol space.
struct Pen<'a> {
    painter: &'a Painter,
    element: &'a Element,
    view: &'a View,
    /// Line color for the current theme.
    color: Color32,
}

impl Pen<'_> {
    fn to_screen(&self, local: Pos2) -> Pos2 {
        self.view.to_screen(self.element.to_diagram(local))
    }

    fn stroke(&self, width: f32) -> Stroke {
        Stroke::new(self.view.stroke_width(width), self.color)
    }

    /// Open polyline with the standard line width.
    fn line(&self, points: &[Pos2]) {
        self.polyline(points, STROKE_WIDTH);
    }

    /// Open polyline with the bold line width, used for letters and plates.
    fn thick_line(&self, points: &[Pos2]) {
        self.polyline(points, THICK_STROKE_WIDTH);
    }

    fn polyline(&self, points: &[Pos2], width: f32) {
        let points = points.iter().map(|&point| self.to_screen(point)).collect();
        self.painter.add(Shape::line(points, self.stroke(width)));
    }

    /// Closed outline, filled when `fill` is set. Filled shapes must be convex.
    fn shape(&self, points: &[Pos2], fill: Option<Color32>) {
        let points: Vec<_> = points.iter().map(|&point| self.to_screen(point)).collect();
        let stroke = self.stroke(STROKE_WIDTH);
        match fill {
            Some(fill) => self.painter.add(Shape::convex_polygon(points, fill, stroke)),
            None => self.painter.add(Shape::closed_line(points, stroke)),
        };
    }

    /// Circle outline, or a solid dot when `filled`.
    fn circle(&self, center: Pos2, radius: f32, filled: bool) {
        let center = self.to_screen(center);
        let radius = radius * self.view.scale;
        if filled {
            self.painter.circle_filled(center, radius, self.color);
        } else {
            self.painter.circle_stroke(center, radius, self.stroke(STROKE_WIDTH));
        }
    }
}

/// Paints a static icon, ported from the v1 SVG artwork.
fn paint_icon(pen: &Pen<'_>, icon: IconKind) {
    match icon {
        IconKind::Arrow => {
            pen.shape(&[pos2(0.7, 2.0), pos2(3.3, 0.5), pos2(3.3, 3.5)], Some(pen.color));
            pen.line(&[pos2(0.0, 2.0), pos2(4.0, 2.0)]);
        }
        IconKind::BurstDisk => {
            pen.line(&[pos2(0.5, 0.0), pos2(0.5, 6.0)]);
            let mut dome = vec![pos2(1.5, 0.0)];
            dome.extend(arc(pos2(1.5, 3.0), Vec2::splat(2.0), -90.0, 90.0));
            dome.push(pos2(1.5, 6.0));
            pen.line(&dome);
            pen.line(&[pos2(0.0, 3.0), pos2(0.5, 3.0)]);
            pen.line(&[pos2(3.5, 3.0), pos2(4.0, 3.0)]);
        }
        IconKind::CheckValve => {
            pen.circle(pos2(1.0, 4.5), 0.5, true);
            pen.line(&[pos2(0.0, 2.5), pos2(1.0, 2.5)]);
            pen.line(&[pos2(9.0, 2.5), pos2(10.0, 2.5)]);
            pen.line(&[pos2(1.0, 0.5), pos2(1.0, 4.5), pos2(9.0, 0.5), pos2(9.0, 4.5)]);
        }
        IconKind::FlexibleConnection => {
            pen.line(&[pos2(0.0, 3.0), pos2(1.0, 3.0)]);
            pen.line(&[pos2(9.0, 3.0), pos2(10.0, 3.0)]);
            for x in [1.0, 2.0, 8.0, 9.0] {
                pen.line(&[pos2(x, 0.0), pos2(x, 6.0)]);
            }
            let mut hose = vec![pos2(2.0, 3.0)];
            hose.extend(arc(pos2(5.0, 3.0), Vec2::splat(2.0), 180.0, 360.0));
            hose.push(pos2(8.0, 3.0));
            pen.line(&hose);
        }
        IconKind::ManualValve => {
            pen.line(&[pos2(0.0, 2.5), pos2(1.0, 2.5)]);
            pen.line(&[pos2(9.0, 2.5), pos2(10.0, 2.5)]);
            bowtie(pen, 0.5, 4.5, None, None);
        }
        IconKind::PressureGauge => {
            pen.circle(pos2(3.5, 3.5), 3.0, false);
            pen.line(&[pos2(3.5, 6.5), pos2(3.5, 7.0)]);
            let ticks = [
                [pos2(1.73, 5.43), pos2(2.43, 4.56)],
                [pos2(1.0, 3.5), pos2(2.0, 3.5)],
                [pos2(1.73, 1.73), pos2(2.43, 2.43)],
                [pos2(3.5, 1.0), pos2(3.5, 2.0)],
                [pos2(5.27, 1.73), pos2(4.56, 2.43)],
                [pos2(6.0, 3.5), pos2(5.0, 3.5)],
                [pos2(5.27, 5.27), pos2(4.56, 4.56)],
            ];
            for tick in ticks {
                pen.line(&tick);
            }
            pen.line(&[pos2(2.7, 6.3), pos2(3.5, 3.5), pos2(4.3, 6.3)]);
        }
        IconKind::PressureRegulator => {
            pen.line(&[pos2(0.0, 7.0), pos2(1.0, 7.0)]);
            pen.line(&[pos2(9.0, 7.0), pos2(10.0, 7.0)]);
            bowtie(pen, 5.0, 9.0, None, None);
            pen.line(&[pos2(5.0, 7.0), pos2(5.0, 4.0)]);
            pen.line(&[
                pos2(5.0, 7.0),
                pos2(8.0, 4.0),
                pos2(8.0, 1.0),
                pos2(5.0, 1.0),
                pos2(5.0, 2.0),
            ]);
            pen.shape(&arc(pos2(5.0, 4.0), Vec2::splat(2.0), 180.0, 360.0), None);
        }
        IconKind::PressureTransducer => {
            pen.circle(pos2(3.5, 3.5), 3.0, false);
            pen.line(&[pos2(3.5, 6.5), pos2(3.5, 7.0)]);
            pen.line(&[pos2(0.7, 4.5), pos2(6.3, 4.5)]);

            // Lettering above the divider, ported as drawn in v1
            let mut letter_p = vec![pos2(2.0, 4.2), pos2(2.0, 1.5), pos2(2.5, 1.5)];
            letter_p.extend(arc(pos2(2.5, 2.25), Vec2::splat(0.75), -90.0, 90.0));
            letter_p.push(pos2(2.0, 3.0));
            pen.thick_line(&letter_p);
            pen.thick_line(&[pos2(4.0, 4.0), pos2(5.0, 4.0)]);
            pen.thick_line(&[pos2(4.5, 4.0), pos2(4.5, 1.5)]);
            pen.thick_line(&[pos2(4.0, 1.5), pos2(5.0, 1.5)]);
        }
        IconKind::QuickConnector => {
            pen.shape(&[pos2(1.0, 0.5), pos2(1.0, 4.5), pos2(5.0, 4.5), pos2(5.0, 0.5)], None);
            pen.line(&[pos2(0.0, 2.5), pos2(1.0, 2.5)]);
            pen.line(&[pos2(5.0, 2.5), pos2(6.0, 2.5)]);
            pen.thick_line(&[pos2(3.0, 0.0), pos2(3.0, 5.0)]);
        }
        IconKind::ReliefValve => {
            pen.shape(&[pos2(3.0, 6.0), pos2(1.5, 8.6), pos2(4.5, 8.6)], None);
            pen.shape(&[pos2(3.0, 6.0), pos2(5.6, 7.5), pos2(5.6, 4.5)], None);
            pen.line(&[pos2(3.0, 8.6), pos2(3.0, 10.0)]);
            pen.thick_line(&[pos2(4.8, 4.4), pos2(4.8, 7.6)]);
            pen.line(&[
                pos2(3.0, 6.0),
                pos2(3.0, 5.0),
                pos2(2.0, 4.5),
                pos2(4.0, 3.5),
                pos2(2.0, 2.5),
                pos2(4.0, 1.5),
                pos2(3.0, 1.0),
                pos2(3.0, 0.0),
            ]);
            pen.circle(pos2(3.0, 6.0), 0.5, true);
        }
        IconKind::Vessel => {
            let mut outline = arc(pos2(4.1, 13.6), Vec2::new(4.0, 1.5), 180.0, 0.0);
            outline.extend(arc(pos2(4.1, 1.6), Vec2::new(4.0, 1.5), 0.0, -180.0));
            pen.shape(&outline, None);
        }
    }
}

/// Paints a motor valve, filling its body green or red according to `state`.
fn paint_motor_valve(pen: &Pen<'_>, kind: MotorValveKind, state: ValveState, open: Color32, closed: Color32) {
    match kind {
        MotorValveKind::TwoWay => {
            let fill = match state {
                ValveState::Open => Some(open),
                ValveState::Closed => Some(closed),
                _ => None,
            };
            pen.line(&[pos2(0.0, 5.0), pos2(1.0, 5.0)]);
            pen.line(&[pos2(9.0, 5.0), pos2(10.0, 5.0)]);
            bowtie(pen, 3.0, 7.0, fill, fill);

            // Actuator: stem and circled "M" above the body
            pen.line(&[pos2(5.0, 2.5), pos2(5.0, 5.0)]);
            pen.circle(pos2(5.0, 1.5), 1.0, false);
            pen.polyline(
                &[
                    pos2(4.6, 1.9),
                    pos2(4.6, 1.1),
                    pos2(5.0, 1.8),
                    pos2(5.4, 1.1),
                    pos2(5.4, 1.9),
                ],
                STROKE_WIDTH / 2.0,
            );
        }
        MotorValveKind::ThreeWay { .. } => {
            let (left, right, bottom) = match state {
                ValveState::ActiveLeft => (Some(open), Some(closed), Some(open)),
                ValveState::ActiveRight => (Some(closed), Some(open), Some(open)),
                _ => (None, None, None),
            };
            pen.line(&[pos2(0.0, 3.0), pos2(1.0, 3.0)]);
            pen.line(&[pos2(9.0, 3.0), pos2(10.0, 3.0)]);
            bowtie(pen, 1.0, 5.0, left, right);
            pen.shape(&[pos2(5.0, 3.0), pos2(7.0, 7.0), pos2(3.0, 7.0)], bottom);
            pen.line(&[pos2(5.0, 7.0), pos2(5.0, 8.0)]);
        }
    }
}

/// Paints the standard valve body spanning x 1–9: two triangles meeting at the center.
/// Drawn as separate triangles because the crossed outline is not convex and cannot be filled.
fn bowtie(pen: &Pen<'_>, top: f32, bottom: f32, left_fill: Option<Color32>, right_fill: Option<Color32>) {
    let center = pos2(5.0, (top + bottom) / 2.0);
    pen.shape(&[pos2(1.0, top), pos2(1.0, bottom), center], left_fill);
    pen.shape(&[pos2(9.0, top), pos2(9.0, bottom), center], right_fill);
}

/// Returns points along an elliptical arc from `start` to `end` degrees,
/// measured clockwise on screen from the positive x axis.
fn arc(center: Pos2, radius: Vec2, start: f32, end: f32) -> Vec<Pos2> {
    let segments = ((end - start).abs() / 180.0 * ARC_SEGMENTS as f32).ceil().max(1.0) as usize;
    (0..=segments)
        .map(|i| {
            let angle = (start + (end - start) * i as f32 / segments as f32) * PI / 180.0;
            center + Vec2::new(radius.x * angle.cos(), radius.y * angle.sin())
        })
        .collect()
}

/// Paints single-line text as large as fits inside `rect`, centered.
fn paint_fitted_text(ui: &Ui, painter: &Painter, rect: Rect, text: &str, color: Color32, monospace: bool) {
    let font = |size| {
        if monospace {
            FontId::monospace(size)
        } else {
            FontId::proportional(size)
        }
    };

    // Measure once at the reference size and scale to the limiting dimension
    let measured = auto_size::measure(ui, text, font(auto_size::REFERENCE_SIZE)).size();
    if measured.x <= 0.0 || measured.y <= 0.0 || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    let size = auto_size::REFERENCE_SIZE * (rect.width() / measured.x).min(rect.height() / measured.y);
    let galley = auto_size::layout(ui, text.to_owned(), font(size), Align::LEFT, true);
    painter.galley_with_override_text_color(rect.center() - galley.size() / 2.0, galley, color);
}

/// Returns the area covering every element and bend point, or `None` for an empty diagram.
pub(super) fn diagram_bounds(widget: &PidWidget) -> Option<Rect> {
    let elements = widget.elements.iter().map(Element::bounds);
    let waypoints = widget
        .connections
        .iter()
        .flat_map(|connection| &connection.waypoints)
        .map(|&point| Rect::from_min_max(point, point));
    elements.chain(waypoints).reduce(|a, b| a.union(b))
}
