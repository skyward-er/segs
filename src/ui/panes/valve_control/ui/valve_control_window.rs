use std::ops::DerefMut;

use egui::{
    Align, Color32, Context, Direction, DragValue, Frame, Id, Key, Label, Layout, Margin,
    Modifiers, Response, RichText, Sense, Stroke, Ui, UiBuilder, Vec2, Widget,
};
use egui_extras::{Size, StripBuilder};
use tracing::info;

use crate::ui::{
    shortcuts::{ShortcutHandler, ShortcutHandlerExt},
    widgets::ShortcutCard,
};

use super::{
    commands::Command,
    icons::Icon,
    map_key_to_shortcut,
    valves::{ParameterValue, Valve, ValveStateManager},
};

const WIGGLE_KEY: Key = Key::Minus;
/// Key used to focus on the aperture field
const FOCUS_APERTURE_KEY: Key = Key::Num1;
/// Key used to focus on the timing field
const FOCUS_TIMING_KEY: Key = Key::Num2;
/// Key used to set the parameter and loose focus on the field
const SET_PAR_KEY: Key = Key::Plus;

#[derive(Debug, Clone, PartialEq)]
pub struct ValveControlView {
    valve: Valve,
    state: ValveViewState,
    timing_ms: u32,
    aperture_perc: f32,
    id: Id,
}

impl ValveControlView {
    pub fn new(valve: Valve, valve_state: &ValveStateManager, id: Id) -> ValveControlView {
        let timing_ms = valve_state.get_timing_for(valve).valid_or(100);
        let aperture_perc = valve_state.get_aperture_for(valve).valid_or(50.0);
        ValveControlView {
            valve,
            state: ValveViewState::Open,
            timing_ms,
            aperture_perc,
            id,
        }
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.state, ValveViewState::Closed)
    }

    #[profiling::function]
    pub fn ui(&mut self, ui: &mut Ui, valve_state: &ValveStateManager) -> Option<Command> {
        // Show only if the window is open
        if self.is_closed() {
            return None;
        }

        // Capture the keyboard shortcuts
        let mut action = self.keyboard_actions(
            ui.id().with("shortcut_lease"),
            ui.ctx().shortcuts().lock().deref_mut(),
        );

        // Draw the view inside the pane
        ui.scope(self.draw_view_ui(&mut action, valve_state));

        // Handle the actions
        self.handle_actions(action, ui.ctx())
    }

    // DISCLAIMER: the code for the UI is really ugly, still learning how to use
    // egui and in a hurry due to deadlines. If you know how to do it better
    // feel free to help us
    fn draw_view_ui(
        &mut self,
        action: &mut Option<WindowAction>,
        valve_state: &ValveStateManager,
    ) -> impl FnOnce(&mut Ui) {
        |ui: &mut Ui| {
            let aperture_field_focus = self.id.with("aperture_field_focus");
            let timing_field_focus = self.id.with("timing_field_focus");

            let valid_fill = ui
                .visuals()
                .widgets
                .inactive
                .bg_fill
                .lerp_to_gamma(Color32::GREEN, 0.3);
            let missing_fill = ui
                .visuals()
                .widgets
                .inactive
                .bg_fill
                .lerp_to_gamma(Color32::YELLOW, 0.3);
            let invalid_fill = ui
                .visuals()
                .widgets
                .inactive
                .bg_fill
                .lerp_to_gamma(Color32::RED, 0.3);

            fn shortcut_ui(ui: &Ui, key: &Key, upper_response: &Response) -> ShortcutCard {
                let vis = ui.visuals();
                let uvis = ui.style().interact(upper_response);
                ShortcutCard::new(map_key_to_shortcut(*key))
                    .text_color(vis.strong_text_color())
                    .fill_color(vis.gray_out(uvis.bg_fill))
                    .margin(Margin::symmetric(5, 2))
                    .text_size(12.)
            }

            fn add_parameter_btn(ui: &mut Ui, key: Key, action_override: bool) -> Response {
                ui.scope_builder(UiBuilder::new().id_salt(key).sense(Sense::click()), |ui| {
                    let mut visuals = *ui.style().interact(&ui.response());

                    // override the visuals if the button is pressed
                    if action_override {
                        visuals = ui.visuals().widgets.active;
                    }

                    let shortcut_card = shortcut_ui(ui, &key, &ui.response());

                    Frame::canvas(ui.style())
                        .inner_margin(Margin::symmetric(4, 2))
                        .outer_margin(0)
                        .corner_radius(ui.visuals().noninteractive().corner_radius)
                        .fill(visuals.bg_fill)
                        .stroke(Stroke::new(1., Color32::TRANSPARENT))
                        .show(ui, |ui| {
                            ui.set_height(ui.available_height());
                            ui.horizontal_centered(|ui| {
                                ui.set_height(21.);
                                ui.add_space(1.);
                                Label::new(
                                    RichText::new("SET").size(16.).color(visuals.text_color()),
                                )
                                .selectable(false)
                                .ui(ui);
                                shortcut_card.ui(ui);
                            });
                        });
                })
                .response
            }

            // set aperture and timing buttons
            fn show_aperture_btn(
                state: &ValveViewState,
                action: &mut Option<WindowAction>,
                ui: &mut Ui,
            ) -> Response {
                let res = match state {
                    ValveViewState::Open => Some(add_parameter_btn(ui, FOCUS_APERTURE_KEY, false)),
                    ValveViewState::ApertureFocused => Some(add_parameter_btn(
                        ui,
                        SET_PAR_KEY,
                        action.is_some_and(|a| a == WindowAction::SetAperture),
                    )),
                    ValveViewState::TimingFocused | ValveViewState::Closed => None,
                };
                if let Some(res) = &res {
                    if res.clicked() {
                        // set the focus on the aperture field
                        action.replace(WindowAction::SetAperture);
                    }
                }
                res.unwrap_or_else(|| ui.response())
            }

            // set timing button
            fn show_timing_btn(
                state: &ValveViewState,
                action: &mut Option<WindowAction>,
                ui: &mut Ui,
            ) -> Response {
                let res = match state {
                    ValveViewState::Open => Some(add_parameter_btn(ui, FOCUS_TIMING_KEY, false)),
                    ValveViewState::TimingFocused => Some(add_parameter_btn(
                        ui,
                        SET_PAR_KEY,
                        action.is_some_and(|a| a == WindowAction::SetTiming),
                    )),
                    ValveViewState::ApertureFocused | ValveViewState::Closed => None,
                };
                if let Some(res) = &res {
                    if res.clicked() {
                        // set the focus on the aperture field
                        action.replace(WindowAction::SetTiming);
                    }
                }
                res.unwrap_or_else(|| ui.response())
            }

            // wiggle button with shortcut
            fn wiggle_btn(ui: &mut Ui, action: &mut Option<WindowAction>) {
                let res = ui
                    .scope_builder(
                        UiBuilder::new().id_salt(WIGGLE_KEY).sense(Sense::click()),
                        |ui| {
                            let mut visuals = *ui.style().interact(&ui.response());

                            // override the visuals if the button is pressed
                            if let Some(WindowAction::Wiggle) = action.as_ref() {
                                visuals = ui.visuals().widgets.active;
                            }

                            let shortcut_card = shortcut_ui(ui, &WIGGLE_KEY, &ui.response());

                            Frame::canvas(ui.style())
                                .inner_margin(Margin::symmetric(4, 2))
                                .outer_margin(0)
                                .corner_radius(ui.visuals().noninteractive().corner_radius)
                                .fill(visuals.bg_fill)
                                .stroke(Stroke::new(1., Color32::TRANSPARENT))
                                .show(ui, |ui| {
                                    ui.set_height(ui.available_height());
                                    ui.horizontal_centered(|ui| {
                                        ui.set_height(21.);
                                        ui.add_space(1.);
                                        Label::new(
                                            RichText::new("WIGGLE")
                                                .size(16.)
                                                .color(visuals.text_color()),
                                        )
                                        .selectable(false)
                                        .ui(ui);
                                        ui.add(
                                            Icon::Wiggle
                                                .as_image(ui.ctx().theme())
                                                .fit_to_exact_size(Vec2::splat(22.)),
                                        );
                                        shortcut_card.ui(ui);
                                    });
                                });
                        },
                    )
                    .response;

                if res.clicked() {
                    // set the focus on the aperture field
                    action.replace(WindowAction::Wiggle);
                }
            }

            fn show_parameter_label(ui: &mut Ui, label: &str, fill_color: Color32) {
                Frame::canvas(ui.style())
                    .outer_margin(0)
                    .inner_margin(Margin::symmetric(0, 3))
                    .corner_radius(ui.visuals().noninteractive().corner_radius)
                    .fill(fill_color)
                    .stroke(Stroke::new(1., Color32::TRANSPARENT))
                    .show(ui, |ui| {
                        Label::new(RichText::new(label).size(14.).strong()).ui(ui);
                    });
            }

            // valve header
            let valve_header = |ui: &mut Ui| {
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    Label::new(
                        RichText::new(self.valve.to_string().to_uppercase())
                            .color(ui.visuals().strong_text_color())
                            .size(16.),
                    )
                    .ui(ui);
                    Label::new(RichText::new("VALVE: ").size(16.))
                        .selectable(false)
                        .ui(ui);
                });
            };

            ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                ui.set_max_width(410.);
                ui.set_min_height(50.);
                StripBuilder::new(ui)
                    .size(Size::exact(5.))
                    .sizes(Size::initial(5.), 3)
                    .vertical(|mut strip| {
                        strip.empty();
                        strip.strip(|builder| {
                            builder
                                .size(Size::exact(252.))
                                .size(Size::initial(50.))
                                .horizontal(|mut strip| {
                                    strip.strip(|builder| {
                                        builder
                                            .size(Size::remainder())
                                            .size(Size::initial(5.))
                                            .size(Size::remainder())
                                            .vertical(|mut strip| {
                                                strip.empty();
                                                strip.cell(valve_header);
                                                strip.empty();
                                            });
                                    });
                                    strip.cell(|ui| wiggle_btn(ui, action));
                                });
                        });
                        strip.strip(|builder| {
                            builder
                                .sizes(Size::initial(100.), 4)
                                .horizontal(|mut strip| {
                                    strip.strip(|builder| {
                                        builder
                                            .size(Size::remainder())
                                            .size(Size::initial(5.))
                                            .size(Size::remainder())
                                            .vertical(|mut strip| {
                                                strip.empty();
                                                strip.cell(|ui| {
                                                    ui.with_layout(
                                                        Layout::right_to_left(Align::Min),
                                                        |ui| {
                                                            Label::new(
                                                                RichText::new("APERTURE:")
                                                                    .size(16.),
                                                            )
                                                            .selectable(false)
                                                            .ui(ui);
                                                        },
                                                    );
                                                });
                                                strip.empty();
                                            });
                                    });
                                    strip.cell(|ui| {
                                        let parameter = valve_state.get_aperture_for(self.valve);
                                        let (label, fill_color) = match parameter {
                                            ParameterValue::Valid(value) => {
                                                (format!("{}%", value * 100.), valid_fill)
                                            }
                                            ParameterValue::Missing => {
                                                (parameter.to_string(), missing_fill)
                                            }
                                            ParameterValue::Invalid(_) => {
                                                (parameter.to_string(), invalid_fill)
                                            }
                                        };
                                        show_parameter_label(ui, &label, fill_color);
                                    });
                                    strip.cell(|ui| {
                                        Frame::canvas(ui.style())
                                            .inner_margin(Margin::symmetric(0, 3))
                                            .outer_margin(0)
                                            .corner_radius(
                                                ui.visuals().noninteractive().corner_radius,
                                            )
                                            .fill(ui.visuals().widgets.inactive.bg_fill)
                                            .stroke(Stroke::new(1., Color32::TRANSPARENT))
                                            .show(ui, |ui| {
                                                // caveat used to clear the field and fill with the current value
                                                if let Some(WindowAction::SetAperture) =
                                                    action.as_ref()
                                                {
                                                    ui.ctx().input_mut(|input| {
                                                        input.events.push(egui::Event::Key {
                                                            key: Key::A,
                                                            physical_key: None,
                                                            pressed: true,
                                                            repeat: false,
                                                            modifiers: Modifiers::COMMAND,
                                                        });
                                                        input.events.push(egui::Event::Text(
                                                            self.aperture_perc.to_string(),
                                                        ));
                                                        input.events.push(egui::Event::Key {
                                                            key: Key::A,
                                                            physical_key: None,
                                                            pressed: true,
                                                            repeat: false,
                                                            modifiers: Modifiers::COMMAND,
                                                        });
                                                    });
                                                }

                                                let res = ui.add_sized(
                                                    Vec2::new(ui.available_width(), 0.0),
                                                    DragValue::new(&mut self.aperture_perc)
                                                        .speed(0.5)
                                                        .range(0.0..=100.0)
                                                        .fixed_decimals(0)
                                                        .update_while_editing(true)
                                                        .suffix("%"),
                                                );

                                                let command_focus = ui.ctx().memory(|m| {
                                                    m.data.get_temp(aperture_field_focus)
                                                });

                                                // needed for making sure the state changes even
                                                // if the pointer clicks inside the field
                                                if res.gained_focus() {
                                                    action.replace(WindowAction::FocusOnAperture);
                                                } else if res.lost_focus() {
                                                    action.replace(WindowAction::LooseFocus);
                                                }

                                                match (command_focus, res.has_focus()) {
                                                    (Some(true), false) => {
                                                        res.request_focus();
                                                    }
                                                    (Some(false), true) => {
                                                        res.surrender_focus();
                                                    }
                                                    _ => {}
                                                }
                                            });
                                    });
                                    strip.cell(|ui| {
                                        show_aperture_btn(&self.state, action, ui);
                                    });
                                });
                        });
                        strip.strip(|builder| {
                            builder
                                .sizes(Size::initial(100.), 4)
                                .horizontal(|mut strip| {
                                    strip.strip(|builder| {
                                        builder
                                            .size(Size::remainder())
                                            .size(Size::initial(10.))
                                            .size(Size::remainder())
                                            .vertical(|mut strip| {
                                                strip.empty();
                                                strip.cell(|ui| {
                                                    ui.with_layout(
                                                        Layout::right_to_left(Align::Min),
                                                        |ui| {
                                                            Label::new(
                                                                RichText::new("TIMING:").size(16.),
                                                            )
                                                            .selectable(false)
                                                            .ui(ui);
                                                        },
                                                    );
                                                });
                                                strip.empty();
                                            });
                                    });
                                    strip.cell(|ui| {
                                        let parameter = valve_state.get_timing_for(self.valve);
                                        let (label, fill_color) = match parameter {
                                            ParameterValue::Valid(value) => {
                                                (format!("{value}ms"), valid_fill)
                                            }
                                            ParameterValue::Missing => {
                                                (parameter.to_string(), missing_fill)
                                            }
                                            ParameterValue::Invalid(_) => {
                                                (parameter.to_string(), invalid_fill)
                                            }
                                        };
                                        show_parameter_label(ui, &label, fill_color);
                                    });
                                    strip.cell(|ui| {
                                        Frame::canvas(ui.style())
                                            .inner_margin(Margin::same(4))
                                            .corner_radius(
                                                ui.visuals().noninteractive().corner_radius,
                                            )
                                            .fill(ui.visuals().widgets.inactive.bg_fill)
                                            .stroke(Stroke::new(1., Color32::TRANSPARENT))
                                            .show(ui, |ui| {
                                                // caveat used to clear the field and fill with the current value
                                                if let Some(WindowAction::SetTiming) =
                                                    action.as_ref()
                                                {
                                                    ui.ctx().input_mut(|input| {
                                                        input.events.push(egui::Event::Key {
                                                            key: Key::A,
                                                            physical_key: None,
                                                            pressed: true,
                                                            repeat: false,
                                                            modifiers: Modifiers::COMMAND,
                                                        });
                                                        input.events.push(egui::Event::Text(
                                                            self.timing_ms.to_string(),
                                                        ));
                                                        input.events.push(egui::Event::Key {
                                                            key: Key::A,
                                                            physical_key: None,
                                                            pressed: true,
                                                            repeat: false,
                                                            modifiers: Modifiers::COMMAND,
                                                        });
                                                    });
                                                }

                                                let res = ui.add_sized(
                                                    Vec2::new(ui.available_width(), 0.0),
                                                    DragValue::new(&mut self.timing_ms)
                                                        .speed(1)
                                                        .range(1..=10000000)
                                                        .fixed_decimals(0)
                                                        .update_while_editing(true)
                                                        .suffix(" [ms]"),
                                                );

                                                let command_focus = ui.ctx().memory(|m| {
                                                    m.data.get_temp(timing_field_focus)
                                                });

                                                // needed for making sure the state changes even
                                                // if the pointer clicks inside the field
                                                if res.gained_focus() {
                                                    action.replace(WindowAction::FocusOnTiming);
                                                } else if res.lost_focus() {
                                                    action.replace(WindowAction::LooseFocus);
                                                }

                                                match (command_focus, res.has_focus()) {
                                                    (Some(true), false) => {
                                                        res.request_focus();
                                                    }
                                                    (Some(false), true) => {
                                                        res.surrender_focus();
                                                    }
                                                    _ => {}
                                                }
                                            });
                                    });
                                    strip.cell(|ui| {
                                        show_timing_btn(&self.state, action, ui);
                                    });
                                });
                        });
                    });
            });
        }
    }

    fn handle_actions(&mut self, action: Option<WindowAction>, ctx: &Context) -> Option<Command> {
        match action {
            // If the action close is called, close the window
            Some(WindowAction::CloseWindow) => {
                self.state = ValveViewState::Closed;
                None
            }
            Some(WindowAction::LooseFocus) => {
                self.state = ValveViewState::Open;
                let aperture_field_focus = self.id.with("aperture_field_focus");
                let timing_field_focus = self.id.with("timing_field_focus");
                ctx.memory_mut(|m| {
                    m.data.insert_temp(aperture_field_focus, false);
                    m.data.insert_temp(timing_field_focus, false);
                });
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
                self.handle_actions(Some(WindowAction::LooseFocus), ctx);
                Some(Command::set_atomic_valve_timing(self.valve, self.timing_ms))
            }
            Some(WindowAction::SetAperture) => {
                info!(
                    "Issued command to set aperture for valve {:?} to {}%",
                    self.valve, self.aperture_perc
                );
                self.handle_actions(Some(WindowAction::LooseFocus), ctx);
                Some(Command::set_valve_maximum_aperture(
                    self.valve,
                    self.aperture_perc / 100.,
                ))
            }
            Some(WindowAction::FocusOnTiming) => {
                self.state = ValveViewState::TimingFocused;
                let timing_field_focus = self.id.with("timing_field_focus");
                ctx.memory_mut(|m| {
                    m.data.insert_temp(timing_field_focus, true);
                });
                None
            }
            Some(WindowAction::FocusOnAperture) => {
                self.state = ValveViewState::ApertureFocused;
                let aperture_field_focus = self.id.with("aperture_field_focus");
                ctx.memory_mut(|m| {
                    m.data.insert_temp(aperture_field_focus, true);
                });
                None
            }
            _ => None,
        }
    }
}

impl ValveControlView {
    #[profiling::function]
    fn keyboard_actions(
        &self,
        id: Id,
        shortcut_handler: &mut ShortcutHandler,
    ) -> Option<WindowAction> {
        shortcut_handler.capture_actions(id, Box::new(()), |s| {
            let mut actions = Vec::new();
            if s.is_operation_mode() && !s.is_command_switch_active {
                match self.state {
                    ValveViewState::Open => {
                        // A window is open, so we can map the keys to control the valve
                        actions.push((Modifiers::NONE, WIGGLE_KEY, WindowAction::Wiggle));
                        actions.push((
                            #[cfg(not(feature = "conrig"))]
                            Modifiers::ALT,
                            #[cfg(feature = "conrig")]
                            Modifiers::NONE,
                            FOCUS_TIMING_KEY,
                            WindowAction::FocusOnTiming,
                        ));
                        actions.push((
                            #[cfg(not(feature = "conrig"))]
                            Modifiers::ALT,
                            #[cfg(feature = "conrig")]
                            Modifiers::NONE,
                            FOCUS_APERTURE_KEY,
                            WindowAction::FocusOnAperture,
                        ));
                        actions.push((Modifiers::NONE, Key::Escape, WindowAction::CloseWindow));
                        actions.push((Modifiers::NONE, Key::Backspace, WindowAction::CloseWindow));
                    }
                    ValveViewState::TimingFocused => {
                        // The timing field is focused, so we can map the keys to control the timing
                        actions.push((Modifiers::NONE, SET_PAR_KEY, WindowAction::SetTiming));
                        actions.push((Modifiers::NONE, Key::Escape, WindowAction::LooseFocus));
                    }
                    ValveViewState::ApertureFocused => {
                        // The aperture field is focused, so we can map the keys to control the aperture
                        actions.push((Modifiers::NONE, SET_PAR_KEY, WindowAction::SetAperture));
                        actions.push((Modifiers::NONE, Key::Escape, WindowAction::LooseFocus));
                    }
                    ValveViewState::Closed => {}
                }
            };
            actions
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValveViewState {
    Closed,
    Open,
    TimingFocused,
    ApertureFocused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
