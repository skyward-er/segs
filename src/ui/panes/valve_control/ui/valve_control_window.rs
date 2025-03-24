use egui::{
    Color32, DragValue, Frame, Key, Label, Modal, Modifiers, Response, RichText, Sense, Stroke, Ui,
    UiBuilder, Vec2, Widget,
};
use tracing::info;

use crate::ui::shortcuts::{ShortcutHandler, ShortcutMode};

use super::{
    commands::Command, icons::Icon, map_key_to_shortcut, shortcut_widget::ShortcutCard,
    valves::Valve,
};

const WIGGLE_KEY: Key = Key::Minus;
const TIMING_KEY: Key = Key::Slash;
const APERTURE_KEY: Key = Key::Period;

#[derive(Debug, Clone, PartialEq)]
pub struct ValveControlWindow {
    valve: Valve,
    state: ValveWindowState,
    timing_ms: u32,
    aperture_perc: f32,
}

impl ValveControlWindow {
    pub fn new(valve: Valve) -> ValveControlWindow {
        ValveControlWindow {
            valve,
            state: ValveWindowState::Open,
            timing_ms: 0,
            aperture_perc: 0.0,
        }
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.state, ValveWindowState::Closed)
    }

    #[profiling::function]
    pub fn ui(&mut self, ui: &mut Ui, shortcut_handler: &mut ShortcutHandler) -> Option<Command> {
        // Show only if the window is open
        if self.is_closed() {
            return None;
        }

        // Capture the keyboard shortcuts
        let mut action = self.keyboard_actions(shortcut_handler);

        // Draw the window UI
        Modal::new(ui.auto_id_with("valve_control"))
            .show(ui.ctx(), self.draw_window_ui(&mut action));

        // Handle the actions
        self.handle_actions(action)
    }

    fn draw_window_ui(&mut self, action: &mut Option<WindowAction>) -> impl FnOnce(&mut Ui) {
        |ui: &mut Ui| {
            let icon_size = Vec2::splat(25.);
            let text_size = 16.;

            fn btn_ui<R>(
                window_state: &ValveWindowState,
                key: Key,
                add_contents: impl FnOnce(&mut Ui) -> R,
            ) -> impl FnOnce(&mut Ui) -> Response {
                move |ui| {
                    let wiggle_btn = Frame::canvas(ui.style())
                        .inner_margin(ui.spacing().menu_margin)
                        .corner_radius(ui.visuals().noninteractive().corner_radius);

                    ui.scope_builder(UiBuilder::new().id_salt(key).sense(Sense::click()), |ui| {
                        let response = ui.response();

                        let clicked = response.clicked();
                        let shortcut_down = ui.ctx().input(|input| input.key_down(key));

                        let visuals = ui.style().interact(&response);
                        let (fill_color, stroke) =
                            if clicked || shortcut_down && window_state.is_open() {
                                let visuals = ui.visuals().widgets.active;
                                (visuals.bg_fill, visuals.bg_stroke)
                            } else if response.hovered() {
                                (visuals.bg_fill, visuals.bg_stroke)
                            } else {
                                let stroke = Stroke::new(1., Color32::TRANSPARENT);
                                (visuals.bg_fill.gamma_multiply(0.3), stroke)
                            };

                        wiggle_btn
                            .fill(fill_color)
                            .stroke(stroke)
                            .stroke(stroke)
                            .show(ui, |ui| {
                                ui.set_width(200.);
                                ui.horizontal(|ui| add_contents(ui))
                            });

                        if response.clicked() {
                            info!("Clicked!");
                        }
                    })
                    .response
                }
            }

            let wiggle_btn_response = btn_ui(&self.state, WIGGLE_KEY, |ui| {
                ShortcutCard::new(map_key_to_shortcut(WIGGLE_KEY))
                    .text_color(ui.visuals().text_color())
                    .fill_color(ui.visuals().widgets.inactive.bg_fill)
                    .text_size(20.)
                    .ui(ui);
                ui.add(
                    Icon::Wiggle
                        .as_image(ui.ctx().theme())
                        .fit_to_exact_size(icon_size),
                );
                ui.add(Label::new(RichText::new("Wiggle").size(text_size)).selectable(false));
            })(ui);

            let aperture_btn_response = btn_ui(&self.state, APERTURE_KEY, |ui| {
                ShortcutCard::new(map_key_to_shortcut(APERTURE_KEY))
                    .text_color(ui.visuals().text_color())
                    .fill_color(ui.visuals().widgets.inactive.bg_fill)
                    .text_size(20.)
                    .ui(ui);
                ui.add(
                    Icon::Aperture
                        .as_image(ui.ctx().theme())
                        .fit_to_exact_size(icon_size),
                );
                ui.add(Label::new(RichText::new("Aperture: ").size(text_size)).selectable(false));
                let drag_value_id = ui.next_auto_id();
                ui.add(
                    DragValue::new(&mut self.aperture_perc)
                        .speed(0.5)
                        .range(0.0..=100.0)
                        .fixed_decimals(0)
                        .update_while_editing(false)
                        .suffix("%"),
                );
                if matches!(&self.state, ValveWindowState::ApertureFocused) {
                    ui.ctx().memory_mut(|m| {
                        m.request_focus(drag_value_id);
                    });
                }
            })(ui);

            let timing_btn_response = btn_ui(&self.state, TIMING_KEY, |ui| {
                ShortcutCard::new(map_key_to_shortcut(TIMING_KEY))
                    .text_color(ui.visuals().text_color())
                    .fill_color(ui.visuals().widgets.inactive.bg_fill)
                    .text_size(20.)
                    .ui(ui);
                ui.add(
                    Icon::Timing
                        .as_image(ui.ctx().theme())
                        .fit_to_exact_size(icon_size),
                );
                ui.add(Label::new(RichText::new("Timing: ").size(text_size)).selectable(false));
                let drag_value_id = ui.next_auto_id();
                ui.add(
                    DragValue::new(&mut self.timing_ms)
                        .speed(1)
                        .range(1..=10000)
                        .fixed_decimals(0)
                        .update_while_editing(false)
                        .suffix(" [ms]"),
                );
                if matches!(&self.state, ValveWindowState::TimingFocused) {
                    ui.ctx().memory_mut(|m| {
                        m.request_focus(drag_value_id);
                    });
                }
            })(ui);

            // consider that action may be different that null if a keyboard shortcut was captured
            if wiggle_btn_response.clicked() {
                action.replace(WindowAction::Wiggle);
            } else if aperture_btn_response.clicked() {
                action.replace(WindowAction::SetAperture);
            } else if timing_btn_response.clicked() {
                action.replace(WindowAction::SetTiming);
            }
        }
    }

    fn handle_actions(&mut self, action: Option<WindowAction>) -> Option<Command> {
        match action {
            // If the action close is called, close the window
            Some(WindowAction::CloseWindow) => {
                self.state = ValveWindowState::Closed;
                None
            }
            Some(WindowAction::LooseFocus) => {
                self.state = ValveWindowState::Open;
                None
            }
            Some(WindowAction::Wiggle) => {
                info!("Issued command to Wiggle valve: {:?}", self.valve);
                Some(Command::wiggle(self.valve))
            }
            Some(WindowAction::SetTiming) => {
                info!(
                    "Issued command to set timing for valve {:?} to {} ms",
                    self.valve, self.timing_ms
                );
                self.state = ValveWindowState::Open;
                Some(Command::set_atomic_valve_timing(self.valve, self.timing_ms))
            }
            Some(WindowAction::SetAperture) => {
                info!(
                    "Issued command to set aperture for valve {:?} to {}%",
                    self.valve, self.aperture_perc
                );
                self.state = ValveWindowState::Open;
                Some(Command::set_valve_maximum_aperture(
                    self.valve,
                    self.aperture_perc / 100.,
                ))
            }
            Some(WindowAction::FocusOnTiming) => {
                self.state = ValveWindowState::TimingFocused;
                None
            }
            Some(WindowAction::FocusOnAperture) => {
                self.state = ValveWindowState::ApertureFocused;
                None
            }
            _ => None,
        }
    }
}

impl ValveControlWindow {
    #[profiling::function]
    fn keyboard_actions(&self, shortcut_handler: &mut ShortcutHandler) -> Option<WindowAction> {
        let mut key_action_pairs = Vec::new();

        shortcut_handler.activate_mode(ShortcutMode::valve_control());
        match self.state {
            ValveWindowState::Open => {
                // A window is open, so we can map the keys to control the valve
                key_action_pairs.push((Modifiers::NONE, WIGGLE_KEY, WindowAction::Wiggle));
                key_action_pairs.push((Modifiers::NONE, TIMING_KEY, WindowAction::FocusOnTiming));
                key_action_pairs.push((
                    Modifiers::NONE,
                    APERTURE_KEY,
                    WindowAction::FocusOnAperture,
                ));
                key_action_pairs.push((Modifiers::NONE, Key::Escape, WindowAction::CloseWindow));
            }
            ValveWindowState::TimingFocused => {
                // The timing field is focused, so we can map the keys to control the timing
                key_action_pairs.push((Modifiers::NONE, Key::Enter, WindowAction::SetTiming));
                key_action_pairs.push((Modifiers::NONE, Key::Escape, WindowAction::LooseFocus));
            }
            ValveWindowState::ApertureFocused => {
                // The aperture field is focused, so we can map the keys to control the aperture
                key_action_pairs.push((Modifiers::NONE, Key::Enter, WindowAction::SetAperture));
                key_action_pairs.push((Modifiers::NONE, Key::Escape, WindowAction::LooseFocus));
            }
            ValveWindowState::Closed => {}
        }
        shortcut_handler.consume_if_mode_is(ShortcutMode::valve_control(), &key_action_pairs[..])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValveWindowState {
    Closed,
    Open,
    TimingFocused,
    ApertureFocused,
}

impl ValveWindowState {
    #[inline]
    fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }
}

#[derive(Debug, Clone, Copy)]
enum WindowAction {
    // window actions
    CloseWindow,
    LooseFocus,
    // commands
    Wiggle,
    SetTiming,
    SetAperture,
    // UI focus
    FocusOnTiming,
    FocusOnAperture,
}
