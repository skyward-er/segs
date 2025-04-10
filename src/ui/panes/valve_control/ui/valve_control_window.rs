use egui::{
    Align, Button, Color32, Direction, DragValue, FontId, Frame, Grid, Key, Label, Layout, Margin,
    Modifiers, Response, RichText, Sense, Stroke, TextEdit, Ui, UiBuilder, Vec2, Widget,
};
use egui_extras::{Size, Strip, StripBuilder};
use tracing::info;

use crate::ui::shortcuts::{ShortcutHandler, ShortcutMode};

use super::{
    commands::Command, icons::Icon, map_key_to_shortcut, shortcut_widget::ShortcutCard,
    valves::Valve,
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
}

impl ValveControlView {
    pub fn new(valve: Valve) -> ValveControlView {
        ValveControlView {
            valve,
            state: ValveViewState::Open,
            timing_ms: 0,
            aperture_perc: 0.0,
        }
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.state, ValveViewState::Closed)
    }

    #[profiling::function]
    pub fn ui(&mut self, ui: &mut Ui, shortcut_handler: &mut ShortcutHandler) -> Option<Command> {
        // Show only if the window is open
        if self.is_closed() {
            return None;
        }

        // Capture the keyboard shortcuts
        let mut action = self.keyboard_actions(shortcut_handler);

        // Draw the view inside the pane
        ui.scope(self.draw_view_ui(&mut action));

        // Handle the actions
        self.handle_actions(action)
    }

    fn draw_view_ui(&mut self, action: &mut Option<WindowAction>) -> impl FnOnce(&mut Ui) {
        |ui: &mut Ui| {
            let icon_size = Vec2::splat(20.);
            let text_size = 14.;

            fn btn_ui<R>(
                window_state: &ValveViewState,
                key: Key,
                add_contents: impl FnOnce(&mut Ui) -> R,
            ) -> impl FnOnce(&mut Ui) -> Response {
                move |ui| {
                    let btn = Frame::canvas(ui.style())
                        .inner_margin(Margin::same(4))
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

                        btn.fill(fill_color)
                            .stroke(stroke)
                            .stroke(stroke)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| add_contents(ui))
                            });

                        if response.clicked() {
                            info!("Clicked!");
                        }
                    })
                    .response
                }
            }

            let valid_fill = ui
                .visuals()
                .widgets
                .inactive
                .bg_fill
                .lerp_to_gamma(Color32::GREEN, 0.3);
            let invalid_fill = ui
                .visuals()
                .widgets
                .inactive
                .bg_fill
                .lerp_to_gamma(Color32::RED, 0.3);

            fn shortcut_ui(ui: &Ui, key: &Key) -> ShortcutCard {
                let vis = ui.visuals();
                ShortcutCard::new(map_key_to_shortcut(*key))
                    .text_color(vis.strong_text_color())
                    .fill_color(vis.gray_out(vis.widgets.inactive.bg_fill))
                    .margin(Margin::symmetric(5, 2))
                    .text_size(12.)
            }

            fn add_parameter_btn(ui: &mut Ui, key: Key) -> Response {
                ui.scope_builder(UiBuilder::new().id_salt(key).sense(Sense::click()), |ui| {
                    Frame::canvas(ui.style())
                        .inner_margin(Margin::symmetric(4, 2))
                        .outer_margin(0)
                        .corner_radius(ui.visuals().noninteractive().corner_radius)
                        .fill(ui.visuals().widgets.inactive.bg_fill)
                        .stroke(Stroke::new(1., Color32::TRANSPARENT))
                        .show(ui, |ui| {
                            ui.set_height(ui.available_height());
                            ui.horizontal_centered(|ui| {
                                ui.set_height(21.);
                                ui.add_space(1.);
                                Label::new(
                                    RichText::new("SET")
                                        .size(16.)
                                        .color(ui.visuals().widgets.inactive.text_color()),
                                )
                                .selectable(false)
                                .ui(ui);
                                shortcut_ui(ui, &key).ui(ui);
                            });
                        });
                })
                .response
            }

            // set aperture and timing buttons
            let aperture_btn: Box<dyn FnOnce(&mut Ui) -> Response> = match self.state {
                ValveViewState::Open => Box::new(|ui| add_parameter_btn(ui, FOCUS_APERTURE_KEY)),
                ValveViewState::ApertureFocused => {
                    Box::new(|ui| add_parameter_btn(ui, SET_PAR_KEY))
                }
                ValveViewState::TimingFocused | ValveViewState::Closed => {
                    Box::new(|ui| ui.response())
                }
            };

            // set timing button
            let timing_btn: Box<dyn FnOnce(&mut Ui) -> Response> = match self.state {
                ValveViewState::Open => Box::new(|ui| add_parameter_btn(ui, FOCUS_TIMING_KEY)),
                ValveViewState::TimingFocused => Box::new(|ui| add_parameter_btn(ui, SET_PAR_KEY)),
                ValveViewState::ApertureFocused | ValveViewState::Closed => {
                    Box::new(|ui| ui.response())
                }
            };

            // wiggle button with shortcut
            let wiggle_btn = |ui: &mut Ui| {
                ui.scope_builder(
                    UiBuilder::new().id_salt(WIGGLE_KEY).sense(Sense::click()),
                    |ui| {
                        Frame::canvas(ui.style())
                            .inner_margin(Margin::symmetric(4, 2))
                            .outer_margin(0)
                            .corner_radius(ui.visuals().noninteractive().corner_radius)
                            .fill(ui.visuals().widgets.inactive.bg_fill)
                            .stroke(Stroke::new(1., Color32::TRANSPARENT))
                            .show(ui, |ui| {
                                ui.set_height(ui.available_height());
                                ui.horizontal_centered(|ui| {
                                    ui.set_height(21.);
                                    ui.add_space(1.);
                                    Label::new(
                                        RichText::new("WIGGLE")
                                            .size(16.)
                                            .color(ui.visuals().widgets.inactive.text_color()),
                                    )
                                    .selectable(false)
                                    .ui(ui);
                                    ui.add(
                                        Icon::Wiggle
                                            .as_image(ui.ctx().theme())
                                            .fit_to_exact_size(Vec2::splat(22.)),
                                    );
                                    shortcut_ui(ui, &WIGGLE_KEY).ui(ui);
                                });
                            });
                    },
                );
            };

            ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                ui.set_max_width(300.);
                ui.set_min_height(50.);
                StripBuilder::new(ui)
                    .size(Size::exact(5.))
                    .sizes(Size::initial(5.), 3)
                    .vertical(|mut strip| {
                        strip.empty();
                        // strip.cell(|ui| {
                        //     // ui.add_sized(
                        //     //     Vec2::new(ui.available_width(), 0.0),
                        //     //     Button::new("Wiggle"),
                        //     // );
                        //     let wiggle_btn_response = btn_ui(&self.state, WIGGLE_KEY, |ui| {
                        //         shortcut_ui(ui, &WIGGLE_KEY).ui(ui);
                        //         ui.add(
                        //             Icon::Wiggle
                        //                 .as_image(ui.ctx().theme())
                        //                 .fit_to_exact_size(icon_size),
                        //         );
                        //         ui.add(
                        //             Label::new(RichText::new("WIGGLE").size(text_size))
                        //                 .selectable(false),
                        //         );
                        //     })(ui);
                        // });
                        strip.strip(|builder| {
                            builder
                                // .size(Size::exact(230.))
                                .size(Size::initial(10.))
                                .size(Size::exact(25.))
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
                                                                RichText::new(
                                                                    self.valve
                                                                        .to_string()
                                                                        .to_uppercase(),
                                                                )
                                                                .color(
                                                                    ui.visuals()
                                                                        .strong_text_color(),
                                                                )
                                                                .size(16.),
                                                            )
                                                            .ui(ui);
                                                            Label::new(
                                                                RichText::new("VALVE: ").size(16.),
                                                            )
                                                            .selectable(false)
                                                            .ui(ui);
                                                        },
                                                    );
                                                });
                                                strip.empty();
                                            });
                                    });
                                    strip.cell(wiggle_btn);
                                });
                        });
                        strip.strip(|builder| {
                            builder
                                .sizes(Size::initial(85.), 4)
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
                                        Frame::canvas(ui.style())
                                            .outer_margin(0)
                                            .inner_margin(Margin::symmetric(0, 3))
                                            .corner_radius(
                                                ui.visuals().noninteractive().corner_radius,
                                            )
                                            .fill(invalid_fill)
                                            .stroke(Stroke::new(1., Color32::TRANSPARENT))
                                            .show(ui, |ui| {
                                                Label::new(
                                                    RichText::new("0.813").size(14.).strong(),
                                                )
                                                .ui(ui);
                                            });
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
                                                let res = ui.add_sized(
                                                    Vec2::new(ui.available_width(), 0.0),
                                                    DragValue::new(&mut self.aperture_perc)
                                                        .speed(0.5)
                                                        .range(0.0..=100.0)
                                                        .fixed_decimals(0)
                                                        .update_while_editing(false)
                                                        .suffix("%"),
                                                );
                                                if res.gained_focus() {
                                                    self.state = ValveViewState::ApertureFocused;
                                                }
                                            });
                                    });
                                    strip.cell(|ui| {
                                        aperture_btn(ui);
                                    });
                                });
                        });
                        strip.strip(|builder| {
                            builder
                                .sizes(Size::initial(85.), 4)
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
                                        Frame::canvas(ui.style())
                                            .inner_margin(Margin::same(4))
                                            .corner_radius(
                                                ui.visuals().noninteractive().corner_radius,
                                            )
                                            .fill(valid_fill)
                                            .stroke(Stroke::new(1., Color32::TRANSPARENT))
                                            .show(ui, |ui| {
                                                Label::new(
                                                    RichText::new("650ms").size(14.).strong(),
                                                )
                                                .ui(ui);
                                            });
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
                                                ui.add_sized(
                                                    Vec2::new(ui.available_width(), 0.0),
                                                    DragValue::new(&mut self.timing_ms)
                                                        .speed(1)
                                                        .range(1..=10000)
                                                        .fixed_decimals(0)
                                                        .update_while_editing(false)
                                                        .suffix(" [ms]"),
                                                );
                                            });
                                    });
                                    strip.cell(|ui| {
                                        timing_btn(ui);
                                    });
                                });
                        });
                    });
            });

            // ui.horizontal(|ui| {
            //     let wiggle_btn_response = btn_ui(&self.state, WIGGLE_KEY, |ui| {
            //         shortcut_ui(ui, &WIGGLE_KEY).ui(ui);
            //         ui.add(
            //             Icon::Wiggle
            //                 .as_image(ui.ctx().theme())
            //                 .fit_to_exact_size(icon_size),
            //         );
            //         ui.add(Label::new(RichText::new("Wiggle").size(text_size)).selectable(false));
            //     })(ui);

            //     let aperture_btn_response = btn_ui(&self.state, APERTURE_KEY, |ui| {
            //         shortcut_ui(ui, &APERTURE_KEY).ui(ui);
            //         ui.add(
            //             Icon::Aperture
            //                 .as_image(ui.ctx().theme())
            //                 .fit_to_exact_size(icon_size),
            //         );
            //         ui.add(
            //             Label::new(RichText::new("Aperture: ").size(text_size)).selectable(false),
            //         );
            //         let drag_value_id = ui.next_auto_id();
            //         ui.add(
            //             DragValue::new(&mut self.aperture_perc)
            //                 .speed(0.5)
            //                 .range(0.0..=100.0)
            //                 .fixed_decimals(0)
            //                 .update_while_editing(false)
            //                 .suffix("%"),
            //         );
            //         if matches!(&self.state, ValveViewState::ApertureFocused) {
            //             ui.ctx().memory_mut(|m| {
            //                 m.request_focus(drag_value_id);
            //             });
            //         }
            //     })(ui);

            //     let timing_btn_response = btn_ui(&self.state, TIMING_KEY, |ui| {
            //         shortcut_ui(ui, &TIMING_KEY).ui(ui);
            //         ui.add(
            //             Icon::Timing
            //                 .as_image(ui.ctx().theme())
            //                 .fit_to_exact_size(icon_size),
            //         );
            //         ui.add(Label::new(RichText::new("Timing: ").size(text_size)).selectable(false));
            //         let drag_value_id = ui.next_auto_id();
            //         ui.add(
            //             DragValue::new(&mut self.timing_ms)
            //                 .speed(1)
            //                 .range(1..=10000)
            //                 .fixed_decimals(0)
            //                 .update_while_editing(false)
            //                 .suffix(" [ms]"),
            //         );
            //         if matches!(&self.state, ValveViewState::TimingFocused) {
            //             ui.ctx().memory_mut(|m| {
            //                 m.request_focus(drag_value_id);
            //             });
            //         }
            //     })(ui);

            //     // consider that action may be different that null if a keyboard shortcut was captured
            //     if wiggle_btn_response.clicked() {
            //         action.replace(WindowAction::Wiggle);
            //     } else if aperture_btn_response.clicked() {
            //         action.replace(WindowAction::SetAperture);
            //     } else if timing_btn_response.clicked() {
            //         action.replace(WindowAction::SetTiming);
            //     }
            // });
        }
    }

    fn handle_actions(&mut self, action: Option<WindowAction>) -> Option<Command> {
        match action {
            // If the action close is called, close the window
            Some(WindowAction::CloseWindow) => {
                self.state = ValveViewState::Closed;
                None
            }
            Some(WindowAction::LooseFocus) => {
                self.state = ValveViewState::Open;
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
                self.state = ValveViewState::Open;
                Some(Command::set_atomic_valve_timing(self.valve, self.timing_ms))
            }
            Some(WindowAction::SetAperture) => {
                info!(
                    "Issued command to set aperture for valve {:?} to {}%",
                    self.valve, self.aperture_perc
                );
                self.state = ValveViewState::Open;
                Some(Command::set_valve_maximum_aperture(
                    self.valve,
                    self.aperture_perc / 100.,
                ))
            }
            Some(WindowAction::FocusOnTiming) => {
                self.state = ValveViewState::TimingFocused;
                None
            }
            Some(WindowAction::FocusOnAperture) => {
                self.state = ValveViewState::ApertureFocused;
                None
            }
            _ => None,
        }
    }
}

impl ValveControlView {
    #[profiling::function]
    fn keyboard_actions(&self, shortcut_handler: &mut ShortcutHandler) -> Option<WindowAction> {
        let mut key_action_pairs = Vec::new();

        shortcut_handler.activate_mode(ShortcutMode::valve_control());
        match self.state {
            ValveViewState::Open => {
                // A window is open, so we can map the keys to control the valve
                key_action_pairs.push((Modifiers::NONE, WIGGLE_KEY, WindowAction::Wiggle));
                // key_action_pairs.push((Modifiers::NONE, TIMING_KEY, WindowAction::FocusOnTiming));
                // key_action_pairs.push((
                //     Modifiers::NONE,
                //     APERTURE_KEY,
                //     WindowAction::FocusOnAperture,
                // ));
                key_action_pairs.push((Modifiers::NONE, Key::Escape, WindowAction::CloseWindow));
            }
            ValveViewState::TimingFocused => {
                // The timing field is focused, so we can map the keys to control the timing
                key_action_pairs.push((Modifiers::NONE, Key::Enter, WindowAction::SetTiming));
                key_action_pairs.push((Modifiers::NONE, Key::Escape, WindowAction::LooseFocus));
            }
            ValveViewState::ApertureFocused => {
                // The aperture field is focused, so we can map the keys to control the aperture
                key_action_pairs.push((Modifiers::NONE, Key::Enter, WindowAction::SetAperture));
                key_action_pairs.push((Modifiers::NONE, Key::Escape, WindowAction::LooseFocus));
            }
            ValveViewState::Closed => {}
        }
        shortcut_handler.consume_if_mode_is(ShortcutMode::valve_control(), &key_action_pairs[..])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValveViewState {
    Closed,
    Open,
    TimingFocused,
    ApertureFocused,
}

impl ValveViewState {
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
