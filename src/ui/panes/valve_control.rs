mod commands;
mod icons;
mod ui;
mod valves;

use std::{
    collections::HashMap,
    ops::DerefMut,
    time::{Duration, Instant},
};

use egui::{
    Color32, DragValue, FontId, Frame, Grid, Id, Key, Label, Modifiers, Response, RichText, Sense,
    Stroke, TextFormat, Ui, UiBuilder, Vec2, Widget, Window, text::LayoutJob, vec2,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use skyward_mavlink::{
    mavlink::{MavHeader, MessageData},
    orion::{ACK_TM_DATA, GSE_TM_DATA, NACK_TM_DATA, VALVE_INFO_TM_DATA, WACK_TM_DATA},
};
use strum::IntoEnumIterator;
use tracing::{debug, info};

use crate::{
    mavlink::{MavMessage, TimedMessage},
    ui::{
        app::PaneResponse,
        panes::valve_control::valves::ParameterValue,
        shortcuts::{ShortcutHandler, ShortcutHandlerExt},
        widgets::ShortcutCard,
    },
};

use super::PaneBehavior;

use commands::CommandSM;
use icons::Icon;
use ui::{ValveControlView, map_key_to_shortcut};
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
    system_id: u8,

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
    valve_key_map: HashMap<Valve, Key>,
    safety_venting: SafetyVentingWatcher,
    /// Map storing the instant the valve will close based on the last ACK received
    #[serde(skip)]
    valve_times_to_close: HashMap<Valve, Instant>,
    #[serde(skip)]
    is_settings_window_open: bool,
    #[serde(skip)]
    valve_view: Option<ValveControlView>,
}

impl Default for ValveControlPane {
    fn default() -> Self {
        let symbols: Vec<char> = SYMBOL_LIST.chars().collect();
        let valve_key_map = Valve::iter()
            .zip(symbols.into_iter().map(map_symbol_to_key))
            .collect();
        Self {
            system_id: 1, // Default system ID, can be changed later
            valves_state: ValveStateManager::default(),
            commands: vec![],
            auto_refresh: None,
            manual_refresh: false,
            last_refresh: None,
            valve_key_map,
            safety_venting: SafetyVentingWatcher::default(),
            valve_times_to_close: HashMap::new(),
            is_settings_window_open: false,
            valve_view: None,
        }
    }
}

impl PaneBehavior for ValveControlPane {
    #[profiling::function]
    fn ui(&mut self, ui: &mut Ui) -> PaneResponse {
        let mut pane_response = PaneResponse::default();

        // process commands to update state
        self.process_commands();

        // Set this to at least double the maximum icon size used
        Icon::init_cache(ui.ctx(), (100, 100));

        if let Some(valve_view) = &mut self.valve_view {
            // A unique ID must be assigned to avoid breaking unique ID management in egui of text edit
            ui.scope_builder(UiBuilder::new().id_salt(valve_view.id()), |ui| {
                if let Some(command) = valve_view.ui(ui, &self.valves_state) {
                    self.commands.push(command.into());
                }
            });

            if valve_view.is_closed() {
                self.valve_view = None;
            }
        } else {
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
            let action = self.keyboard_actions(
                ui.id().with("shortcut_lease"),
                ui.ctx().shortcuts().lock().deref_mut(),
            );

            match action {
                // Open the valve control window if the action is to open it
                Some(PaneAction::OpenValveControl(valve)) => {
                    self.valve_view.replace(ValveControlView::new(
                        valve,
                        &self.valves_state,
                        ui.id().with(valve.to_string()),
                    ));
                }
                None => {}
            }

            Window::new("Settings")
                .id(ui.auto_id_with("settings"))
                .auto_sized()
                .collapsible(true)
                .movable(true)
                .open(&mut self.is_settings_window_open)
                .show(
                    ui.ctx(),
                    Self::settings_window_ui(
                        &mut self.system_id,
                        &mut self.auto_refresh,
                        &mut self.safety_venting,
                    ),
                );
        }

        pane_response
    }

    #[profiling::function]
    fn get_message_subscriptions(&self) -> Box<dyn Iterator<Item = u32>> {
        let mut subscriptions = vec![VALVE_INFO_TM_DATA::ID, GSE_TM_DATA::ID];
        if self.needs_refresh() {
            // TODO
            // subscriptions.push();
        }

        // Subscribe to ACK, NACK, WACK messages if any command is waiting for a response
        if self.commands.iter().any(CommandSM::is_waiting_for_response) {
            let ids = [ACK_TM_DATA::ID, NACK_TM_DATA::ID, WACK_TM_DATA::ID];
            for &id in &ids {
                subscriptions.push(id);
            }
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
            match &message.message {
                MavMessage::VALVE_INFO_TM(valve_info) => {
                    if let Ok(valve) = Valve::try_from(valve_info.servo_id) {
                        self.valves_state.set_timing_for(valve, valve_info.timing);
                        self.valves_state
                            .set_aperture_for(valve, valve_info.aperture as f32 / 100.);
                        if valve_info.state == 1 {
                            let closing_instant = Instant::now()
                                + Duration::from_millis(valve_info.time_to_close as u64);
                            self.valve_times_to_close.insert(valve, closing_instant);
                        } else {
                            self.valve_times_to_close.remove(&valve);
                        }
                    }
                }
                MavMessage::GSE_TM(gse_data) => {
                    macro_rules! update_valve_state {
                        ($field:ident, $valve:expr) => {
                            if gse_data.$field == 0 {
                                self.valve_times_to_close.remove(&$valve);
                            }
                            self.safety_venting
                                .update_valve_state($valve, gse_data.$field);
                        };
                    }

                    update_valve_state!(main_valve_state, Valve::Main);
                    update_valve_state!(nitrogen_valve_state, Valve::Nitrogen);
                    update_valve_state!(n2_filling_valve_state, Valve::N2Filling);
                    update_valve_state!(n2_quenching_valve_state, Valve::N2Quenching);
                    update_valve_state!(n2_release_valve_state, Valve::N2Release);
                    update_valve_state!(ox_filling_valve_state, Valve::OxFilling);
                    update_valve_state!(ox_release_valve_state, Valve::OxRelease);
                    update_valve_state!(ox_venting_valve_state, Valve::OxVenting);
                }
                MavMessage::ACK_TM(_) | MavMessage::NACK_TM(_) | MavMessage::WACK_TM(_) => {
                    for cmd in self.commands.iter_mut() {
                        // intercept all ACK/NACK/WACK messages
                        cmd.capture_response(&message.message);
                    }
                }
                _ => (),
            }
        }

        self.reset_last_refresh();
    }

    #[profiling::function]
    fn drain_outgoing_messages(&mut self) -> Vec<(MavHeader, MavMessage)> {
        let mut outgoing = vec![];

        // Pack and send the next command
        for cmd in self.commands.iter_mut() {
            if let Some(message) = cmd.pack_and_wait() {
                let header = MavHeader {
                    system_id: self.system_id,
                    ..Default::default()
                };
                outgoing.push((header, message));
            }
        }

        outgoing
    }
}

// ┌────────────────────────┐
// │  STATE UPDATE METHODS  │
// └────────────────────────┘
impl ValveControlPane {
    fn process_commands(&mut self) {
        // Process the commands
        for cmd in self.commands.iter_mut() {
            // If the command is waiting for a response, check if it has expired
            cmd.cancel_expired(Duration::from_secs(3));
            // If a response was captured, consume the command and update the valve state
            if let Some((valve, Some(parameter))) = cmd.consume_response() {
                debug!("Valve state updated: {:?}", parameter);
                self.valves_state.set_parameter_of(valve, parameter);
            }
        }

        // Remove consumed commands
        self.commands.retain(|cmd| !cmd.is_consumed());
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
                                self.valve_view = Some(ValveControlView::new(
                                    valve,
                                    &self.valves_state,
                                    ui.id().with(valve.to_string()),
                                ));
                            }
                        }
                        ui.end_row();
                    }
                });
            let time_left = self.safety_venting.time_left();
            let time_left_fmt = time_left
                .map(|d| {
                    let minutes = d.as_secs() / 60;
                    let seconds = d.as_secs() % 60;
                    format!("{:02}:{:02}", minutes, seconds)
                })
                .unwrap_or_else(|| "N/A".to_owned());
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                let mut rich_text =
                    RichText::new(format!("SAFETY VENTING IN {time_left_fmt}")).size(15.0);
                // make the text bold and red if less than 60 seconds left
                if let Some(t) = time_left
                    && t.as_secs() < 60
                {
                    let color = if t.as_secs() % 2 == 0 {
                        Color32::RED
                    } else {
                        Color32::DARK_RED
                    };
                    rich_text = rich_text.strong().color(color);
                }
                ui.add(Label::new(rich_text));
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

    fn settings_window_ui(
        system_id: &mut u8,
        auto_refresh_setting: &mut Option<Duration>,
        safety_venting: &mut SafetyVentingWatcher,
    ) -> impl FnOnce(&mut Ui) {
        |ui| {
            profiling::function_scope!("settings_window_ui");
            ui.set_max_width(300.0);
            // Display auto refresh setting
            let mut auto_refresh = auto_refresh_setting.is_some();
            ui.horizontal(|ui| {
                let label = ui.label("System ID:");
                ui.add(DragValue::new(system_id).range(1..=255))
                    .labelled_by(label.id);
            });
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
            ui.separator();
            let mut emergency_venting_secs = safety_venting.timeout.as_secs();
            ui.horizontal(|ui| {
                ui.label("Safety Venting timeout:");
                DragValue::new(&mut emergency_venting_secs)
                    .speed(1)
                    .range(10..=10800)
                    .update_while_editing(false)
                    .suffix(" s")
                    .ui(ui);
            });
            safety_venting.update_timeout(emergency_venting_secs);
            ui.label("Reset the timeout on actuation of the following:");
            for (valve, active) in safety_venting
                .reset_valves
                .iter_mut()
                .sorted_by(|(a, _), (b, _)| a.to_string().cmp(&b.to_string()))
            {
                ui.checkbox(active, valve.to_string());
            }
        }
    }

    fn valve_frame_ui(&self, valve: Valve, shortcut_key: Key) -> impl FnOnce(&mut Ui) -> Response {
        move |ui| {
            profiling::function_scope!("valve_frame_ui");
            let valve_str = valve.to_string();
            let timing = self.valves_state.get_timing_for(valve);
            let aperture = self.valves_state.get_aperture_for(valve);

            let closing_time_left = self
                .valve_times_to_close
                .get(&valve)
                .map(|i| i.duration_since(Instant::now()));
            let time_left_str = if let Some(closing_time_left) = closing_time_left {
                format!("{:.3} [s]", closing_time_left.as_secs_f32())
            } else {
                "N/A".to_owned()
            };
            let timing_str: String = match timing {
                ParameterValue::Valid(value) => format!("{value} [ms]"),
                ParameterValue::Missing => "N/A".to_owned(),
                ParameterValue::Invalid(err_id) => format!("ERROR({err_id})"),
            };
            let aperture_str = match aperture {
                ParameterValue::Valid(value) => {
                    format!("{:.0}%", value * 100.)
                }
                ParameterValue::Missing => "N/A".to_owned(),
                ParameterValue::Invalid(err_id) => {
                    format!("ERROR({err_id})")
                }
            };
            let text_color = ui.visuals().text_color();

            let valve_title_ui = |ui: &mut Ui| {
                ui.set_max_width(120.);
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
                            let layout_job = LayoutJob::single_section(
                                time_left_str.clone(),
                                text_format.clone(),
                            );
                            let galley = ui.fonts(|fonts| fonts.layout_job(layout_job));
                            Label::new(galley).selectable(false).ui(ui);
                        });
                    });
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
                    let visuals = ui.style().interact(&response);

                    let (fill_color, btn_fill_color, stroke) = if response.clicked() {
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
                                ui.vertical(|ui| {
                                    ui.add_space(8.);
                                    ShortcutCard::new(map_key_to_shortcut(shortcut_key))
                                        .text_color(text_color)
                                        .fill_color(btn_fill_color)
                                        .text_size(23.)
                                        .ui(ui);
                                });
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

    #[profiling::function]
    fn keyboard_actions(
        &self,
        id: Id,
        shortcut_handler: &mut ShortcutHandler,
    ) -> Option<PaneAction> {
        shortcut_handler.capture_actions(id, Box::new(()), |s| {
            let mut actions = Vec::new();
            if s.is_operation_mode() && !s.is_command_switch_active {
                // No window is open, so we can map the keys to open the valve control windows
                for (&valve, &key) in self.valve_key_map.iter() {
                    #[cfg(not(feature = "conrig"))]
                    let modifier = Modifiers::ALT;
                    #[cfg(feature = "conrig")]
                    let modifier = Modifiers::NONE;
                    actions.push((modifier, key, PaneAction::OpenValveControl(valve)));
                }
            }
            actions
        })
    }
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SafetyVentingWatcher {
    /// Last known state of the valves
    last_valve_state: HashMap<Valve, u8>,
    /// Valves that refresh the safety ventime timer
    reset_valves: HashMap<Valve, bool>,
    /// Timeout for safety venting
    timeout: Duration,
    /// Instant of the last valve actuation that reset the timer
    #[serde(skip)]
    last_reset: Option<Instant>,
}

impl Default for SafetyVentingWatcher {
    fn default() -> Self {
        let last_valve_state = Valve::iter().map(|v| (v, 0)).collect::<HashMap<_, _>>();
        let reset_valves = Valve::iter().map(|v| (v, false)).collect::<HashMap<_, _>>();
        let timeout = Duration::from_secs(300); // Default 5 minutes
        Self {
            last_valve_state,
            reset_valves,
            timeout,
            last_reset: None,
        }
    }
}

impl SafetyVentingWatcher {
    fn update_timeout(&mut self, seconds: u64) {
        self.timeout = Duration::from_secs(seconds);
    }

    fn update_valve_state(&mut self, valve: Valve, state: u8) {
        if let Some(last_state) = self.last_valve_state.get_mut(&valve)
            && *last_state != state
        {
            *last_state = state;
            if self.reset_valves[&valve] {
                self.last_reset = Some(Instant::now());
            }
        }
    }

    fn time_left(&self) -> Option<Duration> {
        if let Some(last_reset) = self.last_reset {
            let elapsed = last_reset.elapsed();
            if elapsed >= self.timeout {
                Some(Duration::ZERO)
            } else {
                Some(self.timeout - elapsed)
            }
        } else {
            None
        }
    }
}
