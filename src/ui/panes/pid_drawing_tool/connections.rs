use egui::{epaint::PathStroke, Color32, Painter, Rect, Rounding, Stroke, Theme};
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::ui::utils::glam_to_egui;

use super::{
    grid::{GridInfo, CONNECTION_LINE_THICKNESS, CONNECTION_LINE_THRESHOLD, CONNECTION_POINT_SIZE},
    PidPane,
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Connection {
    /// Index of the start element
    pub start: usize,
    pub start_anchor: usize,

    /// Index of the end element
    pub end: usize,
    pub end_anchor: usize,

    /// Mid points in grid coordinates
    points_g: Vec<Vec2>,
}

impl Connection {
    pub fn new(start: usize, start_anchor: usize, end: usize, end_anchor: usize) -> Self {
        Self {
            start,
            start_anchor,
            end,
            end_anchor,
            points_g: Vec::new(),
        }
    }

    /// Mid points in grid coordinates
    pub fn points(&self) -> Vec<Vec2> {
        self.points_g.clone()
    }

    /// Return the index of the segment the point is on, if any
    pub fn contains(&self, pid: &PidPane, p_s: Vec2) -> Option<usize> {
        let p_g = pid.grid.screen_to_grid(p_s);
        let mut points = Vec::new();

        // Append start point
        points.push(pid.elements[self.start].anchor_point(self.start_anchor));

        // Append all midpoints
        self.points_g
            .iter()
            .map(|p| pid.grid.grid_to_screen(*p))
            .for_each(|p| points.push(p));

        // Append end point
        points.push(pid.elements[self.end].anchor_point(self.end_anchor));

        // Check each segment
        for i in 0..(points.len() - 1) {
            let a = points[i];
            let b = points[i + 1];
            if is_hovering_segment(p_g, a, b) {
                return Some(i);
            }
        }

        None
    }

    /// Checks if the connection references the given element index
    pub fn connected(&self, elem_idx: usize) -> bool {
        self.start == elem_idx || self.end == elem_idx
    }

    /// Returns the index of the point the point is on, if any
    pub fn hovers_point(&self, p_g: Vec2) -> Option<usize> {
        self.points_g
            .iter()
            .position(|p| p.distance(p_g) < CONNECTION_POINT_SIZE)
    }

    /// Splits a segment of the connection with a new point
    pub fn split(&mut self, idx: usize, p_g: Vec2) {
        self.points_g.insert(idx, p_g);
    }

    /// Sets the poisition of one of the path points
    pub fn set_point(&mut self, idx: usize, p_g: Vec2) {
        self.points_g[idx] = p_g;
    }

    fn line_color(theme: Theme) -> Color32 {
        match theme {
            Theme::Light => Color32::BLACK,
            Theme::Dark => Color32::WHITE,
        }
    }

    pub fn draw(&self, pid: &PidPane, painter: &Painter, theme: Theme) {
        let color = Connection::line_color(theme);

        let start = pid.elements[self.start].anchor_point(self.start_anchor);
        let start = pid.grid.grid_to_screen(start);
        let end = pid.elements[self.end].anchor_point(self.end_anchor);
        let end = pid.grid.grid_to_screen(end);

        // Draw line segments
        if self.points_g.is_empty() {
            Connection::draw_segment(&pid.grid, painter, color, start, end);
        } else {
            let points: Vec<Vec2> = self
                .points_g
                .iter()
                .map(|p| pid.grid.grid_to_screen(*p))
                .collect();
            Connection::draw_segment(&pid.grid, painter, color, start, *points.first().unwrap());
            for i in 0..(points.len() - 1) {
                Connection::draw_segment(&pid.grid, painter, color, points[i], points[i + 1]);
            }
            Connection::draw_segment(&pid.grid, painter, color, *points.last().unwrap(), end);

            if pid.editable {
                for point in points {
                    painter.rect(
                        Rect::from_center_size(
                            glam_to_egui(point).to_pos2(),
                            egui::Vec2::splat(CONNECTION_POINT_SIZE * pid.grid.size()),
                        ),
                        Rounding::ZERO,
                        Color32::DARK_GRAY,
                        Stroke::NONE,
                    );
                }
            }
        }
    }

    fn draw_segment(grid: &GridInfo, painter: &Painter, color: Color32, a: Vec2, b: Vec2) {
        painter.line_segment(
            [glam_to_egui(a).to_pos2(), glam_to_egui(b).to_pos2()],
            PathStroke::new(CONNECTION_LINE_THICKNESS * grid.size(), color),
        );
    }
}

fn distance(a: Vec2, b: Vec2) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

/// Distance of a from the line defined by b and c
fn distance_from_line(p: Vec2, m: f32, q: f32) -> f32 {
    (p.y - m * p.x - q).abs() / (1.0 + m * m).sqrt()
}

/// True if p hovers the segment defined by a and b
fn is_hovering_segment(p: Vec2, a: Vec2, b: Vec2) -> bool {
    if a != b {
        let midpoint = (a + b) / 2.0;
        let m = (a.y - b.y) / (a.x - b.x);

        let (d1, d2) = if m == 0.0 {
            ((p.y - midpoint.y).abs(), (p.x - midpoint.x).abs())
        } else if m == f32::INFINITY {
            ((p.x - midpoint.x).abs(), (p.y - midpoint.y).abs())
        } else {
            let q = (a.x * b.y - b.x * a.y) / (a.x - b.x);

            let m_inv = -1.0 / m;
            let q_inv = midpoint.y - m_inv * midpoint.x;

            (
                distance_from_line(p, m, q),
                distance_from_line(p, m_inv, q_inv),
            )
        };

        let length = distance(a, b);

        d1 <= CONNECTION_LINE_THRESHOLD && d2 <= length
    } else {
        false
    }
}
