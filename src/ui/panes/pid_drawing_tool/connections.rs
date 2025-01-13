use egui::Pos2;
use serde::{Deserialize, Serialize};

use super::{grid::LINE_DISTANCE_THRESHOLD, pos::Pos, PidPane};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Connection {
    /// Index of the start element
    pub start: usize,

    /// Index of the end element
    pub end: usize,

    /// Coordinates of middle points
    pub middle_points: Vec<Pos>,
}

impl Connection {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            middle_points: Vec::new(),
        }
    }

    /// Return the index of the segment the position is on, if any
    pub fn contains(&self, pid: &PidPane, pos: &Pos2) -> Option<usize> {
        let mut points = Vec::new();

        // Append start point
        let start = &pid.elements[self.start];
        points.push(start.position.into_pos2(&pid.grid));

        // Append all midpoints
        self.middle_points
            .iter()
            .map(|p| p.into_pos2(&pid.grid))
            .for_each(|p| points.push(p));

        // Append end point
        let end = &pid.elements[self.end];
        points.push(end.position.into_pos2(&pid.grid));

        // Check each segment
        for i in 0..(points.len() - 1) {
            let a = points[i];
            let b = points[i + 1];
            if is_hovering_segment(pos, &a, &b) {
                return Some(i);
            }
        }

        None
    }

    pub fn split(&mut self, idx: usize, pos: Pos) {
        self.middle_points.insert(idx, pos.clone());
    }
}

fn distance(a: &Pos2, b: &Pos2) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

/// Distance of a from the line defined by b and c
fn distance_from_line(p: &Pos2, m: f32, q: f32) -> f32 {
    (p.y - m * p.x - q).abs() / (1.0 + m * m).sqrt()
}

/// True if p hovers the segment defined by a and b
fn is_hovering_segment(p: &Pos2, a: &Pos2, b: &Pos2) -> bool {
    if a != b {
        let midpoint = Pos2 {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
        };
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

        d1 <= LINE_DISTANCE_THRESHOLD && d2 <= length
    } else {
        false
    }
}
