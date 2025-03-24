mod commands;
mod icons;
mod ui;
mod valves;

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use egui::{
    Color32, DragValue, FontId, Frame, Grid, Key, KeyboardShortcut, Label, Modal, Modifiers,
    Response, RichText, Sense, Stroke, TextFormat, Ui, UiBuilder, Vec2, Widget, Window,
    text::LayoutJob, vec2,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use skyward_mavlink::{
    mavlink::MessageData,
    orion::{ACK_TM_DATA, NACK_TM_DATA, WACK_TM_DATA},
};
use strum::IntoEnumIterator;
use tracing::{info, trace, warn};
use ui::ShortcutCard;

use crate::{
    mavlink::{MavMessage, TimedMessage},
    ui::{
        app::PaneResponse,
        shortcuts::{ShortcutHandler, ShortcutMode},
    },
};

use super::PaneBehavior;

use commands::{Command, CommandSM};
use icons::Icon;
use valves::{Valve, ValveStateManager};

const DEFAULT_AUTO_REFRESH_RATE: Duration = Duration::from_secs(1);
const SYMBOL_LIST: &str = "123456789-/.";

fn map_symbol_to_key(symbol: char) -> Key {
    match symbol {
        '1' => Key::Num1,
        '2' => Key::Num2,
        '3' => Key::Num3,
        '4' => Key::Num4,
        '5' => Key::Num5,
        '6' => Key::Num6,
        '7' => Key::Num7,
        '8' => Key::Num8,
        '9' => Key::Num9,
        '-' => Key::Minus,
        '/' => Key::Slash,
        '.' => Key::Period,
        _ => {
            unreachable!("Invalid symbol: {}", symbol);
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct ValveControlPane {
    // INTERNAL
    #[serde(skip)]
    valves_state: ValveStateManager,

    // VALVE COMMANDS LIST
    #[serde(skip)]
    commands: Vec<CommandSM>,

    // REFRESH SETTINGS
    auto_refresh: Option<Duration>,

    #[serde(skip)]
    manual_refresh: bool,

    #[serde(skip)]
    last_refresh: Option<Instant>,

    // UI SETTINGS
    #[serde(skip)]
    is_settings_window_open: bool,
    #[serde(skip)]
    valve_key_map: HashMap<Valve, Key>,
    #[serde(skip)]
    valve_window_states: HashMap<Valve, ValveWindowState>,
}

impl Default for ValveControlPane {
    fn default() -> Self {
        let symbols: Vec<char> = SYMBOL_LIST.chars().collect();
        let valve_key_map = Valve::iter()
            .zip(symbols.into_iter().map(map_symbol_to_key))
            .collect();
        let valve_window_states = Valve::iter()
            .map(|v| (v, ValveWindowState::Closed))
            .collect();
        Self {
            valves_state: ValveStateManager::default(),
            commands: vec![],
            auto_refresh: None,
            manual_refresh: false,
            last_refresh: None,
            is_settings_window_open: false,
            valve_key_map,
            valve_window_states,
        }
    }
}

impl PaneBehavior for ValveControlPane {
    #[profiling::function]
    fn ui(&mut self, ui: &mut Ui, shortcut_handler: &mut ShortcutHandler) -> PaneResponse {
        let mut pane_response = PaneResponse::default();

        // Set this to at least double the maximum icon size used
        Icon::init_cache(ui.ctx(), (100, 100));

        let res = ui
            .scope_builder(UiBuilder::new().sense(Sense::click_and_drag()), |ui| {
                self.pane_ui()(ui);
                ui.allocate_space(ui.available_size());
            })
            .response;

        // Show the menu when the user right-clicks the pane
        res.context_menu(self.menu_ui());

        // Check if the user started dragging the pane
        if res.drag_started() {
            pane_response.set_drag_started();
        }

        // capture actions from keyboard shortcuts
        let action_to_pass = self.keyboard_actions(shortcut_handler);

        match action_to_pass {
            // Open the valve control window if the action is to open it
            Some(PaneAction::OpenValveControl(valve)) => {
                self.valve_window_states
                    .insert(valve, ValveWindowState::Open);
            }
            // Close if the user requests so
            Some(PaneAction::CloseValveControls) => {
                warn!("closing all");
                for valve in Valve::iter() {
                    self.valve_window_states
                        .insert(valve, ValveWindowState::Closed);
                }
            }
            // Ignore otherwise
            _ => {}
        }

        Window::new("Settings")
            .id(ui.auto_id_with("settings"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut self.is_settings_window_open)
            .show(ui.ctx(), Self::settings_window_ui(&mut self.auto_refresh));

        if let Some(valve_window_open) = self
            .valve_window_states
            .iter()
            .find(|&(_, state)| !state.is_closed())
            .map(|(&v, _)| v)
        {
            trace!(
                "Valve control window for valve {} is open",
                valve_window_open
            );
            Modal::new(ui.auto_id_with(format!("valve_control {}", valve_window_open))).show(
                ui.ctx(),
                self.valve_control_window_ui(valve_window_open, action_to_pass),
            );
        }

        pane_response
    }

    #[profiling::function]
    fn get_message_subscriptions(&self) -> Box<dyn Iterator<Item = u32>> {
        let mut subscriptions = vec![];
        if self.needs_refresh() {
            // TODO
            // subscriptions.push();
        }

        // Subscribe to ACK, NACK, WACK messages if any command is waiting for a response
        if self.commands.iter().any(CommandSM::is_waiting_for_response) {
            subscriptions.push(ACK_TM_DATA::ID);
            subscriptions.push(NACK_TM_DATA::ID);
            subscriptions.push(WACK_TM_DATA::ID);
        }

        Box::new(subscriptions.into_iter())
    }

    #[profiling::function]
    fn update(&mut self, messages: &[&TimedMessage]) {
        if self.needs_refresh() {
            // TODO
        }

        // Capture any ACK/NACK/WACK messages and update the valve state
        for message in messages {
            for cmd in self.commands.iter_mut() {
                // intercept all ACK/NACK/WACK messages
                cmd.capture_response(&message.message);
                // If a response was captured, consume the command and update the valve state
                if let Some((valve, Some(parameter))) = cmd.consume_response() {
                    self.valves_state.set_parameter_of(valve, parameter);
                }
            }

            // Remove consumed commands
            self.commands.retain(|cmd| !cmd.is_consumed());
        }

        self.reset_last_refresh();
    }

    #[profiling::function]
    fn drain_outgoing_messages(&mut self) -> Vec<MavMessage> {
        let mut outgoing = vec![];

        // Pack and send the next command
        for cmd in self.commands.iter_mut() {
            if let Some(message) = cmd.pack_and_wait() {
                outgoing.push(message);
            }
        }

        outgoing
    }
}

// ┌────────────────────────┐
// │       UI METHODS       │
// └────────────────────────┘
const BTN_MAX_WIDTH: f32 = 125.;
impl ValveControlPane {
    fn pane_ui(&mut self) -> impl FnOnce(&mut Ui) {
        |ui| {
            profiling::function_scope!("pane_ui");
            ui.set_min_width(BTN_MAX_WIDTH);
            let n = (ui.max_rect().width() / BTN_MAX_WIDTH) as usize;
            let valve_chunks = SYMBOL_LIST.chars().zip(Valve::iter()).chunks(n.max(1));
            Grid::new("valves_grid")
                .num_columns(n)
                .spacing(Vec2::splat(5.))
                .show(ui, |ui| {
                    for chunk in &valve_chunks {
                        for (symbol, valve) in chunk {
                            let response = ui
                                .scope(self.valve_frame_ui(valve, map_symbol_to_key(symbol)))
                                .inner;

                            if response.clicked() {
                                info!("Clicked on valve: {:?}", valve);
                                self.valve_window_states
                                    .insert(valve, ValveWindowState::Open);
                            }
                        }
                        ui.end_row();
                    }
                });
        }
    }

    fn menu_ui(&mut self) -> impl FnOnce(&mut Ui) {
        |ui| {
            profiling::function_scope!("menu_ui");
            if ui.button("Refresh now").clicked() {
                self.manual_refresh = true;
                ui.close_menu();
            }
            if ui.button("Settings").clicked() {
                self.is_settings_window_open = true;
                ui.close_menu();
            }
        }
    }

    fn settings_window_ui(auto_refresh_setting: &mut Option<Duration>) -> impl FnOnce(&mut Ui) {
        |ui| {
            profiling::function_scope!("settings_window_ui");
            // Display auto refresh setting
            let mut auto_refresh = auto_refresh_setting.is_some();
            ui.horizontal(|ui| {
                ui.checkbox(&mut auto_refresh, "Auto Refresh");
                if auto_refresh {
                    let auto_refresh_duration =
                        auto_refresh_setting.get_or_insert(DEFAULT_AUTO_REFRESH_RATE);
                    let mut auto_refresh_period = auto_refresh_duration.as_secs_f32();
                    DragValue::new(&mut auto_refresh_period)
                        .speed(0.2)
                        .range(0.5..=5.0)
                        .fixed_decimals(1)
                        .update_while_editing(false)
                        .prefix("Every ")
                        .suffix(" s")
                        .ui(ui);
                    *auto_refresh_duration = Duration::from_secs_f32(auto_refresh_period);
                } else {
                    *auto_refresh_setting = None;
                }
            });
        }
    }

    fn valve_frame_ui(&self, valve: Valve, shortcut_key: Key) -> impl FnOnce(&mut Ui) -> Response {
        move |ui| {
            profiling::function_scope!("valve_frame_ui");
            let valve_str = valve.to_string();
            let timing = self.valves_state.get_timing_for(valve);
            let aperture = self.valves_state.get_aperture_for(valve);

            let timing_str = match timing {
                valves::ParameterValue::Valid(value) => {
                    format!("{} [ms]", value)
                }
                valves::ParameterValue::Missing => "N/A".to_owned(),
                valves::ParameterValue::Invalid(err_id) => {
                    format!("ERROR({})", err_id)
                }
            };
            let aperture_str = match aperture {
                valves::ParameterValue::Valid(value) => {
                    format!("{:.2}%", value * 100.)
                }
                valves::ParameterValue::Missing => "N/A".to_owned(),
                valves::ParameterValue::Invalid(err_id) => {
                    format!("ERROR({})", err_id)
                }
            };
            let text_color = ui.visuals().text_color();

            let valve_title_ui = |ui: &mut Ui| {
                ui.set_max_width(100.);
                Label::new(
                    RichText::new(valve_str.to_ascii_uppercase())
                        .color(text_color)
                        .strong()
                        .size(15.0),
                )
                .selectable(false)
                .wrap()
                .ui(ui);
            };

            let labels_ui = |ui: &mut Ui| {
                let icon_size = Vec2::splat(17.);
                let text_format = TextFormat {
                    font_id: FontId::proportional(12.),
                    extra_letter_spacing: 0.,
                    line_height: Some(12.),
                    color: text_color,
                    ..Default::default()
                };
                ui.vertical(|ui| {
                    ui.set_min_width(80.);
                    ui.horizontal_top(|ui| {
                        ui.add(
                            Icon::Timing
                                .as_image(ui.ctx().theme())
                                .fit_to_exact_size(icon_size)
                                .sense(Sense::hover()),
                        );
                        ui.allocate_ui(vec2(20., 10.), |ui| {
                            let layout_job =
                                LayoutJob::single_section(timing_str.clone(), text_format.clone());
                            let galley = ui.fonts(|fonts| fonts.layout_job(layout_job));
                            Label::new(galley).selectable(false).ui(ui);
                        });
                    });
                    ui.horizontal_top(|ui| {
                        ui.add(
                            Icon::Aperture
                                .as_image(ui.ctx().theme())
                                .fit_to_exact_size(icon_size)
                                .sense(Sense::hover()),
                        );
                        let layout_job =
                            LayoutJob::single_section(aperture_str.clone(), text_format);
                        let galley = ui.fonts(|fonts| fonts.layout_job(layout_job));
                        Label::new(galley).selectable(false).ui(ui);
                    });
                });
            };

            ui.scope_builder(
                UiBuilder::new()
                    .id_salt("valve_".to_owned() + &valve_str)
                    .sense(Sense::click()),
                |ui| {
                    let response = ui.response();
                    let shortcut_key_is_down = ui
                        .ctx()
                        .input(|input| input.key_down(self.valve_key_map[&valve]));
                    let visuals = ui.style().interact(&response);

                    let (fill_color, btn_fill_color, stroke) =
                        if response.clicked() || shortcut_key_is_down {
                            let visuals = ui.visuals().widgets.active;
                            (visuals.bg_fill, visuals.bg_fill, visuals.bg_stroke)
                        } else if response.hovered() {
                            (
                                visuals.bg_fill,
                                visuals.bg_fill.gamma_multiply(0.8).to_opaque(),
                                visuals.bg_stroke,
                            )
                        } else {
                            (
                                visuals.bg_fill.gamma_multiply(0.3),
                                visuals.bg_fill,
                                Stroke::new(1.0, Color32::TRANSPARENT),
                            )
                        };

                    let inside_frame = |ui: &mut Ui| {
                        ui.vertical(|ui| {
                            valve_title_ui(ui);
                            ui.horizontal(|ui| {
                                ShortcutCard::new(map_key_to_shortcut(shortcut_key))
                                    .text_color(text_color)
                                    .fill_color(btn_fill_color)
                                    .text_size(20.)
                                    .ui(ui);
                                labels_ui(ui);
                            });
                        });
                    };

                    Frame::canvas(ui.style())
                        .fill(fill_color)
                        .stroke(stroke)
                        .inner_margin(ui.spacing().menu_margin)
                        .corner_radius(visuals.corner_radius)
                        .show(ui, inside_frame);
                },
            )
            .response
        }
    }

    const WIGGLE_KEY: Key = Key::Minus;
    const TIMING_KEY: Key = Key::Slash;
    const APERTURE_KEY: Key = Key::Period;

    fn valve_control_window_ui(
        &mut self,
        valve: Valve,
        action: Option<PaneAction>,
    ) -> impl FnOnce(&mut Ui) {
        move |ui| {
            profiling::function_scope!("valve_control_window_ui");
            let icon_size = Vec2::splat(25.);
            let text_size = 16.;

            fn btn_ui<R>(
                valve: Valve,
                key: Key,
                add_contents: impl FnOnce(&mut Ui) -> R,
            ) -> impl FnOnce(&mut Ui) -> Response {
                move |ui| {
                    let wiggle_btn = Frame::canvas(ui.style())
                        .inner_margin(ui.spacing().menu_margin)
                        .corner_radius(ui.visuals().noninteractive().corner_radius);

                    ui.scope_builder(
                        UiBuilder::new()
                            .id_salt(format!("valve_control_window_{}_wiggle", valve))
                            .sense(Sense::click()),
                        |ui| {
                            let response = ui.response();

                            let clicked = response.clicked();
                            let shortcut_down = ui.ctx().input(|input| input.key_down(key));

                            let visuals = ui.style().interact(&response);
                            let stroke = if shortcut_down || clicked {
                                let visuals = ui.visuals().widgets.active;
                                visuals.bg_stroke
                            } else if response.hovered() {
                                visuals.bg_stroke
                            } else {
                                Stroke::new(1., Color32::TRANSPARENT)
                            };

                            wiggle_btn
                                .fill(visuals.bg_fill.gamma_multiply(0.3).to_opaque())
                                .stroke(stroke)
                                .stroke(stroke)
                                .show(ui, |ui| {
                                    ui.set_width(200.);
                                    ui.horizontal(|ui| add_contents(ui))
                                });

                            if response.clicked() {
                                info!("Clicked!");
                            }
                        },
                    )
                    .response
                }
            }

            let wiggle_btn_response = btn_ui(valve, Self::WIGGLE_KEY, |ui| {
                ShortcutCard::new(map_key_to_shortcut(Self::WIGGLE_KEY))
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

            let mut aperture = 0_u32;
            let aperture_btn_response = btn_ui(valve, Self::APERTURE_KEY, |ui| {
                ShortcutCard::new(map_key_to_shortcut(Self::APERTURE_KEY))
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
                ui.add(
                    DragValue::new(&mut aperture)
                        .speed(0.5)
                        .range(0.0..=100.0)
                        .fixed_decimals(1)
                        .update_while_editing(false)
                        .suffix("%"),
                );
            })(ui);

            let mut timing_ms = 0_u32;
            let timing_btn_response = btn_ui(valve, Self::TIMING_KEY, |ui| {
                ShortcutCard::new(map_key_to_shortcut(Self::TIMING_KEY))
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
                ui.add(
                    DragValue::new(&mut timing_ms)
                        .speed(1)
                        .range(1..=10000)
                        .fixed_decimals(0)
                        .update_while_editing(false)
                        .suffix(" [ms]"),
                );
            })(ui);

            if wiggle_btn_response.clicked() || matches!(action, Some(PaneAction::Wiggle)) {
                info!("Wiggle valve: {:?}", valve);
                self.commands.push(Command::wiggle(valve).into());
            }
            // self.valve_window_states
            //     .insert(valve, ValveWindowState::Closed);
        }
    }

    #[profiling::function]
    fn keyboard_actions(&self, shortcut_handler: &mut ShortcutHandler) -> Option<PaneAction> {
        let mut key_action_pairs = Vec::new();
        match self
            .valve_window_states
            .iter()
            .find(|&(_, open)| !open.is_closed())
        {
            Some((&valve, state)) => {
                shortcut_handler.activate_mode(ShortcutMode::valve_control());
                match state {
                    ValveWindowState::Open => {
                        // A window is open, so we can map the keys to control the valve
                        key_action_pairs.push((
                            Modifiers::NONE,
                            Self::WIGGLE_KEY,
                            PaneAction::Wiggle,
                        ));
                        key_action_pairs.push((
                            Modifiers::NONE,
                            Self::TIMING_KEY,
                            PaneAction::FocusOnTiming,
                        ));
                        key_action_pairs.push((
                            Modifiers::NONE,
                            Self::APERTURE_KEY,
                            PaneAction::FocusOnAperture,
                        ));
                        key_action_pairs.push((
                            Modifiers::NONE,
                            Key::Escape,
                            PaneAction::CloseValveControls,
                        ));
                    }
                    ValveWindowState::TimingFocused => {
                        // The timing field is focused, so we can map the keys to control the timing
                        key_action_pairs.push((Modifiers::NONE, Key::Enter, PaneAction::SetTiming));
                        key_action_pairs.push((
                            Modifiers::NONE,
                            Key::Escape,
                            PaneAction::OpenValveControl(valve),
                        ));
                    }
                    ValveWindowState::ApertureFocused => {
                        // The aperture field is focused, so we can map the keys to control the aperture
                        key_action_pairs.push((
                            Modifiers::NONE,
                            Key::Enter,
                            PaneAction::SetAperture,
                        ));
                        key_action_pairs.push((
                            Modifiers::NONE,
                            Key::Escape,
                            PaneAction::OpenValveControl(valve),
                        ));
                    }
                    ValveWindowState::Closed => unreachable!(),
                }
                shortcut_handler
                    .consume_if_mode_is(ShortcutMode::valve_control(), &key_action_pairs[..])
            }
            None => {
                shortcut_handler.deactivate_mode(ShortcutMode::valve_control());
                // No window is open, so we can map the keys to open the valve control windows
                for (&valve, &key) in self.valve_key_map.iter() {
                    key_action_pairs.push((
                        Modifiers::NONE,
                        key,
                        PaneAction::OpenValveControl(valve),
                    ));
                }
                shortcut_handler
                    .consume_if_mode_is(ShortcutMode::composition(), &key_action_pairs[..])
            }
        }
    }
}

#[inline]
fn map_key_to_shortcut(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::NONE, key)
}

// ┌───────────────────────────┐
// │       UTILS METHODS       │
// └───────────────────────────┘
impl ValveControlPane {
    fn needs_refresh(&self) -> bool {
        // manual trigger of refresh
        let manual_triggered = self.manual_refresh;
        // automatic trigger of refresh
        let auto_triggered = if let Some(auto_refresh) = self.auto_refresh {
            self.last_refresh
                .is_none_or(|last| last.elapsed() >= auto_refresh)
        } else {
            false
        };

        manual_triggered || auto_triggered
    }

    fn reset_last_refresh(&mut self) {
        self.last_refresh = Some(Instant::now());
        self.manual_refresh = false;
    }
}

#[derive(Debug, Clone, Copy)]
enum PaneAction {
    OpenValveControl(Valve),
    CloseValveControls,
    Wiggle,
    SetTiming,
    SetAperture,
    FocusOnTiming,
    FocusOnAperture,
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

    #[inline]
    fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

    #[inline]
    fn is_focused(&self) -> bool {
        matches!(self, Self::TimingFocused | Self::ApertureFocused)
    }
}
