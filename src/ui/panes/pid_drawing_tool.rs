mod connections;
mod elements;
mod grid;
mod symbols;

use connections::Connection;
use core::f32;
use egui::{
    Button, Color32, Context, CursorIcon, PointerButton, Response, Sense, Theme, Ui, Widget,
};
use egui_tiles::TileId;
use elements::Element;
use glam::Vec2;
use grid::GridInfo;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use symbols::{Symbol, icons::Icon};

use crate::{
    MAVLINK_PROFILE,
    error::ErrInstrument,
    mavlink::{GSE_TM_DATA, MessageData, TimedMessage, reflection::MessageLike},
    ui::{app::PaneResponse, cache::ChangeTracker, utils::egui_to_glam},
};

use super::PaneBehavior;

#[derive(Clone, Debug)]
enum Action {
    Connect(usize),
    ContextMenu(Vec2),
    DragElement(usize),
    DragConnection(usize, usize),
    DragGrid,
}

/// Piping and instrumentation diagram
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PidPane {
    // Persistent internal state
    elements: Vec<Element>,
    connections: Vec<Connection>,
    grid: GridInfo,
    message_subscription_id: u32,

    // UI settings
    center_content: bool,

    // Temporary internal state
    #[serde(skip)]
    action: Option<Action>,
    #[serde(skip)]
    editable: bool,
    #[serde(skip)]
    is_subs_window_visible: bool,
    #[serde(skip)]
    contains_pointer: bool,
}

impl Default for PidPane {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            connections: Vec::new(),
            grid: GridInfo::default(),
            message_subscription_id: GSE_TM_DATA::ID,
            center_content: false,
            action: None,
            editable: false,
            is_subs_window_visible: false,
            contains_pointer: false,
        }
    }
}

impl PartialEq for PidPane {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
            && self.connections == other.connections
            && self.grid == other.grid
            && self.center_content == other.center_content
    }
}

impl PaneBehavior for PidPane {
    fn ui(&mut self, ui: &mut egui::Ui, _: TileId) -> PaneResponse {
        let mut pane_response = PaneResponse::default();

        let theme = PidPane::find_theme(ui.ctx());

        if self.center_content && !self.editable {
            self.center(ui);
        }

        if self.editable {
            self.draw_grid(ui, theme);
        }
        self.draw_connections(ui, theme);
        self.elements_ui(ui, theme);

        // Handle things that require knowing the position of the pointer
        let (_, response) = ui.allocate_at_least(ui.max_rect().size(), Sense::click_and_drag());
        if let Some(pointer_pos) = response.hover_pos().map(|p| egui_to_glam(p.to_vec2())) {
            if self.editable {
                self.handle_zoom(ui, theme, pointer_pos);
            }

            // Set grab icon when hovering something
            let hovers_element = self.hovers_element(pointer_pos).is_some();
            let hovers_connection_point = self.hovers_connection_point(pointer_pos).is_some();
            if self.editable && (hovers_element || hovers_connection_point) {
                ui.ctx()
                    .output_mut(|output| output.cursor_icon = CursorIcon::Grab);
            }

            self.detect_action(&response, pointer_pos);
            self.handle_actions(&response, pointer_pos);
        }

        // The context menu does not need the pointer's position.
        // If active it has to be shown even if the pointer goes off screen.
        if let Some(Action::ContextMenu(pointer_pos)) = self.action.clone() {
            response.context_menu(|ui| self.draw_context_menu(ui, pointer_pos));
        }

        let change_tracker = ChangeTracker::record_initial_state(self.message_subscription_id);
        egui::Window::new("Subscription")
            .id(ui.auto_id_with("sub_settings"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut self.is_subs_window_visible)
            .show(ui.ctx(), |ui| {
                subscription_window(ui, &mut self.message_subscription_id)
            });
        if change_tracker.has_changed(self.message_subscription_id) {
            self.reset_subscriptions();
        }

        // Check if the user is draqging the pane
        self.contains_pointer = response.contains_pointer();
        let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);
        if response.dragged() && (ctrl_pressed || !self.editable) {
            pane_response.set_drag_started();
        }

        pane_response
    }

    fn contains_pointer(&self) -> bool {
        self.contains_pointer
    }

    fn update(&mut self, messages: &[&TimedMessage]) {
        if let Some(msg) = messages.last() {
            for element in &mut self.elements {
                element.update(&msg.message, self.message_subscription_id);
            }
        }
    }

    fn get_message_subscriptions(&self) -> Box<dyn Iterator<Item = u32>> {
        Box::new(Some(self.message_subscription_id).into_iter())
    }
}

impl PidPane {
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
            Theme::Light => Color32::BLACK.gamma_multiply(0.2),
        }
    }

    /// Returns the index of the element the point is on, if any
    fn hovers_element(&self, p_s: Vec2) -> Option<usize> {
        self.elements
            .iter()
            .position(|elem| elem.contains(self.grid.screen_to_grid(p_s)))
    }

    /// Return the connection and segment indexes where the position is on, if any
    fn hovers_connection(&self, p_s: Vec2) -> Option<(usize, usize)> {
        self.connections
            .iter()
            .enumerate()
            .find_map(|(conn_idx, conn)| {
                let segm_idx = conn.contains(self, p_s);
                Some(conn_idx).zip(segm_idx)
            })
    }

    fn hovers_connection_point(&self, p_s: Vec2) -> Option<(usize, usize)> {
        self.connections
            .iter()
            .enumerate()
            .find_map(|(conn_idx, conn)| {
                let p_idx = conn.hovers_point(self.grid.screen_to_grid(p_s));
                Some(conn_idx).zip(p_idx)
            })
    }

    fn draw_grid(&self, ui: &Ui, theme: Theme) {
        let painter = ui.painter();
        let window_rect = ui.max_rect();
        let dot_color = PidPane::dots_color(theme);

        let offset_x = (self.grid.zero_pos.x % self.grid.size()) as i32;
        let offset_y = (self.grid.zero_pos.y % self.grid.size()) as i32;

        let start_x =
            (window_rect.min.x / self.grid.size()) as i32 * self.grid.size() as i32 + offset_x;
        let end_x = (window_rect.max.x / self.grid.size() + 2.0) as i32 * self.grid.size() as i32
            + offset_x;
        let start_y =
            (window_rect.min.y / self.grid.size()) as i32 * self.grid.size() as i32 + offset_y;
        let end_y = (window_rect.max.y / self.grid.size() + 2.0) as i32 * self.grid.size() as i32
            + offset_y;

        for x in (start_x..end_x).step_by(self.grid.size() as usize) {
            for y in (start_y..end_y).step_by(self.grid.size() as usize) {
                let rect = egui::Rect::from_min_size(
                    egui::Pos2::new(x as f32, y as f32),
                    egui::Vec2::new(2.0, 2.0),
                );
                painter.rect_filled(rect, 0.0, dot_color);
            }
        }
    }

    fn draw_connections(&self, ui: &Ui, theme: Theme) {
        let painter = ui.painter();

        for conn in &self.connections {
            conn.draw(self, painter, theme);
        }
    }

    fn elements_ui(&mut self, ui: &mut Ui, theme: Theme) {
        for element in &mut self.elements {
            ui.scope(|ui| {
                element.ui(ui, &self.grid, theme, self.message_subscription_id);
            });
        }
    }

    fn draw_context_menu(&mut self, ui: &mut Ui, pointer_pos: Vec2) {
        ui.set_max_width(180.0); // To make sure we wrap long text

        if !self.editable {
            if ui.button("Enable editing").clicked() {
                self.editable = true;
                self.action.take();
                ui.close_menu();
            }
            ui.checkbox(&mut self.center_content, "Center");
            return;
        }

        if let Some(elem_idx) = self.hovers_element(pointer_pos) {
            if ui.button("Connect").clicked() {
                self.action = Some(Action::Connect(elem_idx));
                ui.close_menu();
            }
            let btn_response = Button::new("Delete").ui(ui);
            self.elements[elem_idx].context_menu(ui);
            // Handle the delete button
            if btn_response.clicked() {
                self.delete_element(elem_idx);
                self.action.take();
                ui.close_menu();
            }
        } else if let Some((conn_idx, segm_idx)) = self.hovers_connection(pointer_pos) {
            if ui.button("Split").clicked() {
                self.connections[conn_idx].split(segm_idx, self.grid.screen_to_grid(pointer_pos));
                self.action.take();
                ui.close_menu();
            }
            if ui.button("Change start anchor").clicked() {
                let conn = &mut self.connections[conn_idx];
                conn.start_anchor =
                    (conn.start_anchor + 1) % self.elements[conn.start].anchor_points_len();
                self.action.take();
                ui.close_menu();
            }
            if ui.button("Change end anchor").clicked() {
                let conn = &mut self.connections[conn_idx];
                conn.end_anchor =
                    (conn.end_anchor + 1) % self.elements[conn.end].anchor_points_len();
                self.action.take();
                ui.close_menu();
            }
        } else {
            ui.menu_button("Symbols", |ui| {
                for symbol in Symbol::iter() {
                    if let Symbol::Icon(_) = symbol {
                        ui.menu_button("Icons", |ui| {
                            for icon in Icon::iter() {
                                if ui.button(icon.to_string()).clicked() {
                                    self.elements.push(Element::new(
                                        self.grid.screen_to_grid(pointer_pos).round(),
                                        Symbol::Icon(icon),
                                    ));
                                    self.action.take();
                                    ui.close_menu();
                                }
                            }
                        });
                    } else if ui.button(symbol.to_string()).clicked() {
                        self.elements.push(Element::new(
                            self.grid.screen_to_grid(pointer_pos).round(),
                            symbol,
                        ));
                        self.action.take();
                        ui.close_menu();
                    }
                }
            });
        }

        if ui.button("Pane subscription settings…").clicked() {
            self.is_subs_window_visible = true;
            ui.close_menu();
        }

        if ui.button("Disable editing").clicked() {
            self.editable = false;
            ui.close_menu();
        }
    }

    /// Removes an element from the diagram
    fn delete_element(&mut self, elem_idx: usize) {
        // First delete connection referencing this element
        self.connections.retain(|elem| !elem.connected(elem_idx));

        // Then the element
        self.elements.remove(elem_idx);
    }

    fn center(&mut self, ui: &Ui) {
        let ui_center = egui_to_glam(ui.max_rect().center().to_vec2());

        // Chain elements positions and connection mid points
        let points: Vec<Vec2> = self
            .elements
            .iter()
            .map(|e| e.center())
            .chain(self.connections.iter().flat_map(|conn| conn.points()))
            .collect();

        if !points.is_empty() {
            let min_x = points
                .iter()
                .map(|p| p.x)
                .min_by(|a, b| a.total_cmp(b))
                .log_unwrap();
            let min_y = points
                .iter()
                .map(|p| p.y)
                .min_by(|a, b| a.total_cmp(b))
                .log_unwrap();
            let min = Vec2::new(min_x, min_y);

            let max_x = points
                .iter()
                .map(|p| p.x)
                .max_by(|a, b| a.total_cmp(b))
                .log_unwrap();
            let max_y = points
                .iter()
                .map(|p| p.y)
                .max_by(|a, b| a.total_cmp(b))
                .log_unwrap();
            let max = Vec2::new(max_x, max_y);

            self.grid.zero_pos = ui_center - min.midpoint(max) * self.grid.size();
        }
    }

    fn handle_zoom(&mut self, ui: &Ui, theme: Theme, pointer_pos: Vec2) {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta).y;
        if scroll_delta != 0.0 {
            self.grid.apply_scroll_delta(scroll_delta, pointer_pos);

            // Invalidate the cache to redraw the images
            for icon in Icon::iter() {
                let img: egui::ImageSource = icon.get_image(theme);
                ui.ctx().forget_image(img.uri().log_unwrap());
            }
        }
    }

    fn detect_action(&mut self, response: &Response, pointer_pos: Vec2) {
        if response.clicked_by(PointerButton::Secondary) {
            self.action = Some(Action::ContextMenu(pointer_pos));
        } else if self.editable {
            if response.drag_started() {
                if response.dragged_by(PointerButton::Middle) {
                    self.action = Some(Action::DragGrid);
                } else if let Some(drag_element_action) =
                    self.hovers_element(pointer_pos).map(Action::DragElement)
                {
                    self.action = Some(drag_element_action);
                } else if let Some(drag_connection_point) = self
                    .hovers_connection_point(pointer_pos)
                    .map(|(idx1, idx2)| Action::DragConnection(idx1, idx2))
                {
                    self.action = Some(drag_connection_point);
                }
            } else if response.drag_stopped() {
                self.action.take();
            }
        }
    }

    fn handle_actions(&mut self, response: &Response, pointer_pos: Vec2) {
        match self.action {
            Some(Action::Connect(start)) => {
                if response.clicked() {
                    if let Some(end) = self.hovers_element(pointer_pos) {
                        if start != end {
                            self.connections.push(Connection::new(start, 0, end, 0));
                        }
                        self.action.take();
                    }
                }
            }
            Some(Action::DragElement(idx)) => {
                let pointer_pos_g = self.grid.screen_to_grid(pointer_pos).round();
                self.elements[idx].set_center_at(pointer_pos_g);
            }
            Some(Action::DragConnection(conn_idx, point_idx)) => {
                let pointer_pos_g = self.grid.screen_to_grid(pointer_pos).round();
                self.connections[conn_idx].set_point(point_idx, pointer_pos_g);
            }
            Some(Action::DragGrid) => {
                self.grid.zero_pos += egui_to_glam(response.drag_delta());
            }
            // Context menu has to be handled outside since it does not reuquire the pointer's position
            Some(Action::ContextMenu(_)) => {}
            None => {}
        }
    }

    fn reset_subscriptions(&mut self) {
        for element in &mut self.elements {
            element.reset_subscriptions();
        }
    }
}

fn subscription_window(ui: &mut Ui, msg_id: &mut u32) {
    let current_msg = msg_id.to_mav_message(&MAVLINK_PROFILE).log_unwrap();
    egui::ComboBox::from_label("Message subscription")
        .selected_text(current_msg.name.as_str())
        .show_ui(ui, |ui| {
            for msg in MAVLINK_PROFILE.get_sorted_msgs() {
                ui.selectable_value(msg_id, msg.id, &msg.name);
            }
        });
}
