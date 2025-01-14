mod connections;
mod elements;
mod grid;
mod pos;
mod symbols;

use connections::Connection;
use core::f32;
use egui::{
    epaint::PathStroke, Color32, Context, CursorIcon, Painter, PointerButton, Pos2, Rounding,
    Sense, Stroke, Theme, Ui, Vec2,
};
use elements::Element;
use grid::{GridInfo, LINE_THICKNESS};
use pos::Pos;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use strum::IntoEnumIterator;
use symbols::Symbol;

use crate::ui::composable_view::PaneResponse;

use super::PaneBehavior;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
enum Action {
    Connect(usize),
    ContextMenu(Pos2),
    DragElement(usize),
    DragConnection(usize, usize),
    DragGrid,
}

/// Piping and instrumentation diagram
#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PidPane {
    elements: Vec<Element>,
    connections: Vec<Connection>,

    grid: GridInfo,

    #[serde(skip)]
    action: Option<Action>,

    #[serde(skip)]
    editable: bool,

    center_content: bool,
}

impl PaneBehavior for PidPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let theme = PidPane::find_theme(ui.ctx());

        if self.center_content && !self.editable {
            self.center(ui);
        }

        if self.editable {
            self.draw_grid(ui, theme);
        }

        self.draw_connections(ui, theme, self.editable);
        self.draw_elements(ui, theme);

        // Allocate the space to sense inputs
        let (_, response) = ui.allocate_at_least(ui.max_rect().size(), Sense::click_and_drag());
        let pointer_pos = response.hover_pos();

        if let Some(pointer_pos) = pointer_pos {
            if self.editable {
                self.handle_zoom(ui, theme, &pointer_pos);
            }
        }

        // Set grab icon when hovering an element
        if let Some(pointer_pos) = &pointer_pos {
            if self.editable
                && (self.is_hovering_element(pointer_pos)
                    || self.is_hovering_connection_point(pointer_pos))
            {
                ui.ctx()
                    .output_mut(|output| output.cursor_icon = CursorIcon::Grab);
            }
        }

        // Detect the action
        if let Some(pointer_pos) = &pointer_pos {
            if response.clicked_by(PointerButton::Secondary) {
                println!("Context menu opened");
                self.action = Some(Action::ContextMenu(*pointer_pos));
            } else if self.editable {
                if response.drag_started() {
                    if response.dragged_by(PointerButton::Middle) {
                        self.action = Some(Action::DragGrid);
                        println!("Grid drag started");
                    } else if let Some(drag_element_action) = self
                        .find_hovered_element_idx(pointer_pos)
                        .map(Action::DragElement)
                    {
                        self.action = Some(drag_element_action);
                        println!("Element drag started");
                    } else if let Some(drag_connection_point) = self
                        .find_hovered_connection_point(pointer_pos)
                        .map(|(idx1, idx2)| Action::DragConnection(idx1, idx2))
                    {
                        self.action = Some(drag_connection_point);
                        println!("Connection point drag started");
                    }
                } else if response.drag_stopped() {
                    self.action.take();
                    println!("Drag stopped");
                }
            }
        }

        // Context menu
        if let Some(Action::ContextMenu(pointer_pos)) = self.action.clone() {
            response.context_menu(|ui| self.draw_context_menu(ui, &pointer_pos));
        }

        // Connect action
        if let Some(pointer_pos) = pointer_pos {
            match self.action {
                Some(Action::Connect(start)) => {
                    if let Some(end) = self.find_hovered_element_idx(&pointer_pos) {
                        if response.clicked() {
                            if start != end {
                                self.connections.push(Connection::new(start, 0, end, 0));
                                println!("Added connection from {} to {}", start, end);
                            }
                            self.action.take();
                            println!("Connect action ended");
                        }
                    }
                }
                Some(Action::DragElement(idx)) => {
                    self.elements[idx].position = Pos::from_pos2(&self.grid, &pointer_pos)
                }
                Some(Action::DragConnection(conn_idx, midpoint_idx)) => {
                    self.connections[conn_idx].middle_points[midpoint_idx] =
                        Pos::from_pos2(&self.grid, &pointer_pos);
                }
                Some(Action::DragGrid) => {
                    self.grid.zero_pos += response.drag_delta();
                }
                _ => {}
            }
        }

        PaneResponse::default()
    }

    fn contains_pointer(&self) -> bool {
        false
    }
}

impl PidPane {
    fn is_hovering_element(&self, pointer_pos: &Pos2) -> bool {
        self.elements
            .iter()
            .any(|element| element.contains(&self.grid, pointer_pos))
    }

    fn is_hovering_connection_point(&self, pointer_pos: &Pos2) -> bool {
        self.connections.iter().any(|conn| {
            conn.middle_points
                .iter()
                .any(|p| p.distance(&self.grid, pointer_pos) < 10.0)
        })
    }

    /// Returns the currently used theme
    fn find_theme(ctx: &Context) -> Theme {
        // In Egui you can either decide a theme or use the system one.
        // If the system theme cannot be determined, a fallback theme can be set.
        ctx.options(|options| match options.theme_preference {
            egui::ThemePreference::Light => Theme::Light,
            egui::ThemePreference::Dark => Theme::Dark,
            egui::ThemePreference::System => match ctx.system_theme() {
                Some(Theme::Light) => Theme::Light,
                Some(Theme::Dark) => Theme::Dark,
                None => options.fallback_theme,
            },
        })
    }

    fn dots_color(theme: Theme) -> Color32 {
        match theme {
            Theme::Dark => Color32::DARK_GRAY,
            Theme::Light => Color32::BLACK,
        }
    }

    fn find_hovered_element_idx(&self, pos: &Pos2) -> Option<usize> {
        self.elements
            .iter()
            .position(|elem| elem.contains(&self.grid, pos))
    }

    fn find_hovered_element_mut(&mut self, pos: &Pos2) -> Option<&mut Element> {
        self.elements
            .iter_mut()
            .find(|element| element.contains(&self.grid, pos))
    }

    /// Return the connection and segment indexes where the position is on, if any
    fn find_hovered_connection_idx(&self, pos: &Pos2) -> Option<(usize, usize)> {
        self.connections
            .iter()
            .enumerate()
            .find_map(|(idx, conn)| Some(idx).zip(conn.contains(self, pos)))
    }

    fn find_hovered_connection_point(&self, pos: &Pos2) -> Option<(usize, usize)> {
        let mut midpoint_idx = Some(0);
        let connection_idx = self.connections.iter().position(|conn| {
            midpoint_idx = conn
                .middle_points
                .iter()
                .position(|p| p.distance(&self.grid, pos) < 12.0);
            midpoint_idx.is_some()
        });

        connection_idx.zip(midpoint_idx)
    }

    fn draw_grid(&self, ui: &Ui, theme: Theme) {
        let painter = ui.painter();
        let window_rect = ui.max_rect();
        let dot_color = PidPane::dots_color(theme);

        let start_x =
            (window_rect.min.x / self.grid.get_size()) as i32 * self.grid.get_size() as i32;
        let end_x =
            (window_rect.max.x / self.grid.get_size() + 1.0) as i32 * self.grid.get_size() as i32;
        let start_y =
            (window_rect.min.y / self.grid.get_size()) as i32 * self.grid.get_size() as i32;
        let end_y =
            (window_rect.max.y / self.grid.get_size() + 1.0) as i32 * self.grid.get_size() as i32;

        for x in (start_x..end_x).step_by(self.grid.get_size() as usize) {
            for y in (start_y..end_y).step_by(self.grid.get_size() as usize) {
                let rect = egui::Rect::from_min_size(
                    egui::Pos2::new(x as f32, y as f32),
                    egui::Vec2::new(1.0, 1.0),
                );
                painter.rect_filled(rect, 0.0, dot_color);
            }
        }
    }

    fn draw_connections(&self, ui: &Ui, theme: Theme, draw_handles: bool) {
        let painter = ui.painter();
        let color = match theme {
            Theme::Light => Color32::BLACK,
            Theme::Dark => Color32::WHITE,
        };

        // Each connection is composed from multiple lines
        for conn in &self.connections {
            let start = self.elements[conn.start].get_anchor(&self.grid, conn.start_anchor);
            let end = self.elements[conn.end].get_anchor(&self.grid, conn.end_anchor);

            let points: Vec<Pos2> = conn
                .middle_points
                .iter()
                .map(|p| p.to_pos2(&self.grid))
                .collect();

            // Draw line segments
            if points.is_empty() {
                self.draw_connection_segment(painter, color, start, end);
            } else {
                self.draw_connection_segment(painter, color, start, *points.first().unwrap());
                for i in 0..(points.len() - 1) {
                    self.draw_connection_segment(painter, color, points[i], points[i + 1]);
                }
                self.draw_connection_segment(painter, color, *points.last().unwrap(), end);
            }

            // Draw handles (dragging boxes)
            if draw_handles {
                for point in points {
                    painter.rect(
                        egui::Rect::from_center_size(
                            point,
                            Vec2::new(self.grid.get_size(), self.grid.get_size()),
                        ),
                        Rounding::ZERO,
                        Color32::DARK_GRAY,
                        Stroke::NONE,
                    );
                }
            }
        }
    }

    fn draw_connection_segment(&self, painter: &Painter, color: Color32, a: Pos2, b: Pos2) {
        painter.line_segment(
            [a, b],
            PathStroke::new(LINE_THICKNESS * self.grid.get_size(), color),
        );
    }

    fn draw_elements(&self, ui: &Ui, theme: Theme) {
        for element in &self.elements {
            let image_rect = egui::Rect::from_center_size(
                element.position.to_pos2(&self.grid),
                Vec2::splat(element.size as f32 * self.grid.get_size()),
            );

            egui::Image::new(element.symbol.get_image(theme))
                .rotate(element.rotation, Vec2::new(0.5, 0.5))
                .paint_at(ui, image_rect);
        }
    }

    fn draw_context_menu(&mut self, ui: &mut Ui, pointer_pos: &Pos2) {
        ui.set_max_width(120.0); // To make sure we wrap long text

        if !self.editable {
            if ui.button("Enable editing").clicked() {
                self.editable = true;
                ui.close_menu();
            }
            ui.checkbox(&mut self.center_content, "Center");
            return;
        }

        if self.is_hovering_element(pointer_pos) {
            let hovered_element = self.find_hovered_element_idx(pointer_pos);
            if ui.button("Connect").clicked() {
                if let Some(idx) = hovered_element {
                    println!("Connect action started");
                    self.action = Some(Action::Connect(idx));
                } else {
                    panic!("No element found where the \"Connect\" action was issued");
                }
                ui.close_menu();
            }
            if ui.button("Rotate 90° ⟲").clicked() {
                if let Some(elem) = self.find_hovered_element_mut(pointer_pos) {
                    elem.rotation += PI / 2.0;
                } else {
                    panic!("No element found where the \"Rotate 90° ⟲\" action was issued");
                }
                ui.close_menu();
            }
            if ui.button("Rotate 90° ⟳").clicked() {
                if let Some(elem) = self.find_hovered_element_mut(pointer_pos) {
                    elem.rotation -= PI / 2.0;
                } else {
                    panic!("No element found where the \"Rotate 90° ⟳\" action was issued");
                }
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                if let Some(idx) = self.find_hovered_element_idx(pointer_pos) {
                    self.delete_element(idx);
                } else {
                    panic!("No element found where the \"Delete\" action was issued");
                }
                ui.close_menu();
            }
        } else if let Some((conn_idx, segm_idx)) = self.find_hovered_connection_idx(pointer_pos) {
            if ui.button("Split").clicked() {
                println!("Splitting connection line");
                self.connections[conn_idx].split(segm_idx, Pos::from_pos2(&self.grid, pointer_pos));
                ui.close_menu();
            }
            if ui.button("Change start anchor").clicked() {
                let conn = &mut self.connections[conn_idx];
                conn.start_anchor = (conn.start_anchor + 1)
                    % self.elements[conn.start].symbol.get_anchor_points().len();
                ui.close_menu();
            }
            if ui.button("Change end anchor").clicked() {
                let conn = &mut self.connections[conn_idx];
                conn.end_anchor = (conn.end_anchor + 1)
                    % self.elements[conn.end].symbol.get_anchor_points().len();
                ui.close_menu();
            }
        } else {
            ui.menu_button("Symbols", |ui| {
                for symbol in Symbol::iter() {
                    if ui.button(symbol.to_string()).clicked() {
                        self.elements.push(Element::new(
                            Pos::from_pos2(&self.grid, pointer_pos),
                            symbol,
                        ));
                        ui.close_menu();
                    }
                }
            });
        }

        if ui.button("Disable editing").clicked() {
            self.editable = false;
            ui.close_menu();
        }
    }

    fn delete_element(&mut self, idx: usize) {
        // First delete connection referencing this element
        self.connections
            .retain(|elem| elem.start != idx && elem.end != idx);

        // Then the element
        self.elements.remove(idx);
    }

    fn center(&mut self, ui: &Ui) {
        let ui_center = ui.max_rect().center();

        let points: Vec<Pos> = self
            .elements
            .iter()
            .map(|e| e.position.clone())
            .chain(
                self.connections
                    .iter()
                    .flat_map(|conn| conn.middle_points.clone()),
            )
            .collect();

        let min_x = points.iter().map(|p| p.x).min().unwrap();
        let max_x = points.iter().map(|p| p.x).max().unwrap();
        let min_y = points.iter().map(|p| p.y).min().unwrap();
        let max_y = points.iter().map(|p| p.y).max().unwrap();

        let pid_center =
            Pos::new((max_x + min_x) / 2, (max_y + min_y) / 2).to_relative_pos2(&self.grid);

        self.grid.zero_pos = ui_center - pid_center.to_vec2();
    }

    fn handle_zoom(&mut self, ui: &Ui, theme: Theme, pointer_pos: &Pos2) {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta).y;
        if scroll_delta != 0.0 {
            self.grid.apply_scroll_delta(scroll_delta, pointer_pos);

            // Invalidate the cache to redraw the images
            for symbol in Symbol::iter() {
                let img: egui::ImageSource = symbol.get_image(theme);
                ui.ctx().forget_image(img.uri().unwrap());
            }
        }
    }
}
