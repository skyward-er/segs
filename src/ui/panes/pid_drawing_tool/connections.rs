use egui::{epaint::PathStroke, Color32, Painter, Rect, Rounding, Stroke, Theme};
use glam::{Mat2, Vec2};
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
        self.points_g.iter().for_each(|&p| points.push(p));

        // Append end point
        points.push(pid.elements[self.end].anchor_point(self.end_anchor));

        // Check each segment
        for i in 0..(points.len() - 1) {
            let a = points[i];
            let b = points[i + 1];
            if hovers_segment(&pid.grid, p_g, a, b) {
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

/// True if p hovers the segment defined by a and b
fn hovers_segment(grid: &GridInfo, p_g: Vec2, a_g: Vec2, b_g: Vec2) -> bool {
    if a_g != b_g {
        let segment_g = b_g - a_g;
        let rotm = Mat2::from_angle(-segment_g.to_angle());

        // Rototranslate the point in the segment frame with a as origin
        let p_s = rotm * (p_g - a_g);

        let y_threshold = CONNECTION_LINE_THRESHOLD / grid.size();
        0.0 <= p_s.x && p_s.x <= segment_g.length() && p_s.y.abs() <= y_threshold
    } else {
        // If a and b are the same point, prevent adding another
        false
    }
}
