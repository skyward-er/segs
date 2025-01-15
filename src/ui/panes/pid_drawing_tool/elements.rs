use crate::ui::utils::glam_to_egui;

use super::grid::GridInfo;
use super::symbols::Symbol;
use egui::{Rect, Theme, Ui};
use glam::{Mat2, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Element {
    /// Anchor postion in grid coordinates, top-left corner
    position: glam::Vec2,

    /// Symbol to be displayed
    symbol: Symbol,

    /// Rotation in radiants
    rotation: f32,

    /// Anchor point in grid coordinates relative to the element's center
    ///
    /// These vectors include the current rotation of the element.
    /// They are cached to avoid recomputing the rotation.
    anchor_points: Vec<Vec2>,
}

impl Element {
    pub fn new(position: Vec2, symbol: Symbol) -> Self {
        Self {
            position,
            rotation: 0.0,
            anchor_points: symbol.anchor_points(),
            symbol,
        }
    }

    /// Check if the given position is inside the element
    pub fn contains(&self, p_g: Vec2) -> bool {
        // First we need to do a rotostranslation from the grid's frame to the element's frame
        let rotm = Mat2::from_angle(-self.rotation);
        let p_e = rotm * (p_g - self.position);

        // The bounding box is just the size
        let min_e = Vec2::ZERO;
        let max_e = self.symbol.size();

        // Check if the point is in the bounding box
        min_e.x <= p_e.x && p_e.x <= max_e.x && min_e.y <= p_e.y && p_e.y <= max_e.y
    }

    /// Moves the element such that its center is at the given position
    pub fn set_center_at(&mut self, p_g: Vec2) {
        // Rotation matrix from element's frame to grid's frame
        let rotm_e_to_g = Mat2::from_angle(self.rotation);

        // Center in grid's frame
        let center_g = rotm_e_to_g * self.symbol.size() / 2.0;

        self.position = p_g - center_g;
    }

    pub fn change_symbol(&mut self, symbol: Symbol) {
        self.symbol = symbol;

        // Anchor points can be different between symbols, realod the cache
        self.reload_anchor_points();
    }

    /// Rotate the element by its center
    pub fn rotate(&mut self, rotation: f32) {
        // Current center position relative to the top-left point in the grid reference frame
        let center_g = Mat2::from_angle(self.rotation) * self.symbol.size() / 2.0;

        // Rotate the position by the element's center
        self.position += (Mat2::IDENTITY - Mat2::from_angle(rotation)) * center_g;

        // Update absolute rotation
        self.rotation += rotation;

        // Recompute anchor points cache
        self.reload_anchor_points();
    }

    fn reload_anchor_points(&mut self) {
        // Rotation matrix from element's frame to grid's frame
        let rotm_e_to_g = Mat2::from_angle(self.rotation);

        // Then rotate the anchor points
        self.anchor_points = self
            .symbol
            .anchor_points()
            .iter()
            .map(|&p| rotm_e_to_g * p)
            .collect();
    }

    /// Returns the position of one anchor point in grid coordinates
    pub fn anchor_point(&self, idx: usize) -> Vec2 {
        self.anchor_points[idx] + self.position
    }

    pub fn anchor_points_len(&self) -> usize {
        self.anchor_points.len()
    }

    /// Size in grid units
    pub fn size(&self) -> Vec2 {
        self.symbol.size()
    }

    /// Position of the element's top-left corner
    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn draw(&self, grid: &GridInfo, ui: &Ui, theme: Theme) {
        let center = glam_to_egui(grid.grid_to_screen(self.position)).to_pos2();
        let image_rect = Rect::from_min_size(center, glam_to_egui(self.size() * grid.size()));

        egui::Image::new(self.symbol.get_image(theme))
            .rotate(self.rotation, egui::Vec2::splat(0.0))
            .paint_at(ui, image_rect);
    }
}
