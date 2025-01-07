mod pid_elements;

use egui::{epaint::PathStroke, Color32, PointerButton, Pos2, Sense, Theme, Vec2};
use pid_elements::{PidElement, PidSymbol};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use strum::IntoEnumIterator;

use crate::ui::composable_view::PaneResponse;

use super::PaneBehavior;

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PidPane {
    elements: Vec<PidElement>,
    dragged: Option<usize>,
    context_menu_pos: (i32, i32),
    connect_element: Option<usize>,
    connections: Vec<(usize, usize)>,
}

impl PaneBehavior for PidPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let step_size: i32 = 10;
        let window_rect = ui.max_rect();
        let painter = ui.painter();

        let theme = ui.ctx().options(|options| match options.theme_preference {
            egui::ThemePreference::Light => Theme::Light,
            egui::ThemePreference::Dark => Theme::Dark,
            egui::ThemePreference::System => match ui.ctx().system_theme() {
                Some(Theme::Light) => Theme::Light,
                Some(Theme::Dark) => Theme::Dark,
                None => options.fallback_theme,
            },
        });

        // Draw the dot grid
        let dot_color = match theme {
            Theme::Dark => egui::Color32::DARK_GRAY,
            Theme::Light => egui::Color32::BLACK,
        };
        for x in
            (window_rect.min.x as i32..window_rect.max.x.round() as i32).step_by(step_size as usize)
        {
            for y in (window_rect.min.y as i32..window_rect.max.y.round() as i32)
                .step_by(step_size as usize)
            {
                let rect = egui::Rect::from_min_size(
                    egui::Pos2::new(x as f32, y as f32),
                    egui::Vec2::new(1.0, 1.0),
                );
                painter.rect_filled(rect, 0.0, dot_color);
            }
        }

        for connection in &self.connections {
            let elem1 = &self.elements[connection.0];
            let elem2 = &self.elements[connection.1];

            let x1 = (elem1.pos.0 + elem1.size / 2) * step_size;
            let y1 = (elem1.pos.1 + elem1.size / 2) * step_size;
            let x2 = (elem2.pos.0 + elem2.size / 2) * step_size;
            let y2 = (elem2.pos.1 + elem2.size / 2) * step_size;

            painter.line_segment(
                [
                    Pos2::new(x1 as f32, y1 as f32),
                    Pos2::new(x2 as f32, y2 as f32),
                ],
                PathStroke::new(1.0, Color32::GREEN),
            );
        }

        // Draw elements
        for element in &self.elements {
            let image_rect = egui::Rect::from_min_size(
                egui::Pos2::new(
                    (element.pos.0 * step_size) as f32,
                    (element.pos.1 * step_size) as f32,
                ),
                egui::Vec2::new(
                    (element.size * step_size) as f32,
                    (element.size * step_size) as f32,
                ),
            );

            egui::Image::new(element.get_image(theme))
                .rotate(element.rotation, Vec2::new(0.5, 0.5))
                .paint_at(ui, image_rect);
        }

        let (_, response) = ui.allocate_at_least(window_rect.size(), Sense::click_and_drag());

        let pointer_pos = response
            .hover_pos()
            .map(|pos| (pos.x as i32 / step_size, pos.y as i32 / step_size))
            .unwrap_or((0, 0));

        if response.clicked_by(PointerButton::Secondary) {
            self.context_menu_pos = pointer_pos;
        }
        response.context_menu(|ui| {
            ui.set_max_width(200.0); // To make sure we wrap long text

            if self.is_hovering_element(self.context_menu_pos) {
                if ui.button("Connect").clicked() {
                    self.connect_element = self
                        .elements
                        .iter()
                        .position(|element| element.contains(self.context_menu_pos));
                    ui.close_menu();
                }

                if ui.button("Rotate 90° ⟲").clicked() {
                    if let Some(elem) = self
                        .elements
                        .iter_mut()
                        .find(|element| element.contains(self.context_menu_pos))
                    {
                        elem.rotation += PI / 2.0;
                    }
                    ui.close_menu();
                }
                if ui.button("Rotate 90° ⟳").clicked() {
                    if let Some(elem) = self
                        .elements
                        .iter_mut()
                        .find(|element| element.contains(self.context_menu_pos))
                    {
                        elem.rotation -= PI / 2.0;
                    }
                    ui.close_menu();
                }
            }

            ui.menu_button("Symbols", |ui| {
                for symbol in PidSymbol::iter() {
                    if ui.button(symbol.to_string()).clicked() {
                        self.elements.push(PidElement {
                            pos: self.context_menu_pos,
                            size: 10,
                            symbol,
                            rotation: 0.0,
                        });
                        ui.close_menu();
                    }
                }
            });
        });

        if response.clicked() && self.connect_element.is_some() {
            let second_connect_element = self
                .elements
                .iter()
                .position(|element| element.contains(pointer_pos));

            if let (Some(elem1), Some(elem2)) = (self.connect_element, second_connect_element) {
                if elem1 != elem2 {
                    self.connections.push((elem1, elem2));
                }
                self.connect_element.take();
            }
        }

        if response.drag_started() {
            // Find which element the drag started on
            self.dragged = self
                .elements
                .iter()
                .position(|element| element.contains(pointer_pos));
        }
        if response.dragged() {
            if let Some(dragged) = self.dragged {
                let element = &mut self.elements[dragged];

                element.pos.0 = pointer_pos.0 - element.size / 2;
                element.pos.1 = pointer_pos.1 - element.size / 2;
            }
        }
        if response.drag_stopped() {
            self.dragged.take();
        }

        PaneResponse::default()
    }

    fn contains_pointer(&self) -> bool {
        false
    }
}

impl PidPane {
    fn is_hovering_element(&self, pointer_pos: (i32, i32)) -> bool {
        self.elements
            .iter()
            .find(|element| element.contains(pointer_pos))
            .is_some()
    }
}
