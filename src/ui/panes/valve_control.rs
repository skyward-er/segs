mod commands;
mod icons;
mod valves;

use std::time::{Duration, Instant};

use egui::{
    Color32, DragValue, Frame, Grid, Label, Layout, Rect, RichText, Sense, Stroke, Ui, UiBuilder,
    Vec2, Widget, vec2,
};
use egui_extras::{Size, StripBuilder};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use skyward_mavlink::{
    mavlink::MessageData,
    orion::{ACK_TM_DATA, NACK_TM_DATA, WACK_TM_DATA},
};
use strum::IntoEnumIterator;
use tracing::{info, warn};

use crate::{
    mavlink::{MavMessage, TimedMessage},
    ui::app::PaneResponse,
};

use super::PaneBehavior;

use commands::CommandSM;
use icons::Icon;
use valves::{Valve, ValveStateManager};

const DEFAULT_AUTO_REFRESH_RATE: Duration = Duration::from_secs(1);

#[derive(Clone, PartialEq, Default, Serialize, Deserialize, Debug)]
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
}

impl PaneBehavior for ValveControlPane {
    fn ui(&mut self, ui: &mut Ui) -> PaneResponse {
        let mut pane_response = PaneResponse::default();

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

        egui::Window::new("Settings")
            .id(ui.auto_id_with("settings"))
            .auto_sized()
            .collapsible(true)
            .movable(true)
            .open(&mut self.is_settings_window_open)
            .show(ui.ctx(), Self::window_ui(&mut self.auto_refresh));

        pane_response
    }

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
                if let Some((valve, parameter)) = cmd.consume_response() {
                    self.valves_state.set_parameter_of(valve, parameter);
                }
            }

            // Remove consumed commands
            self.commands.retain(|cmd| !cmd.is_consumed());
        }

        self.reset_last_refresh();
    }

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
impl ValveControlPane {
    fn pane_ui(&mut self) -> impl FnOnce(&mut Ui) {
        |ui| {
            let valve_chunks = Valve::iter().chunks(3);
            Grid::new("valves_grid")
                .num_columns(3)
                .spacing(Vec2::splat(5.))
                .show(ui, |ui| {
                    for chunk in &valve_chunks {
                        for valve in chunk {
                            ui.scope(self.valve_frame_ui(valve));
                        }
                        ui.end_row();
                    }
                });
        }
    }

    fn menu_ui(&mut self) -> impl FnOnce(&mut Ui) {
        |ui| {
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

    fn window_ui(auto_refresh_setting: &mut Option<Duration>) -> impl FnOnce(&mut Ui) {
        |ui| {
            // Display auto refresh setting
            let mut auto_refresh = auto_refresh_setting.is_some();
            ui.horizontal(|ui| {
                ui.checkbox(&mut auto_refresh, "Auto Refresh");
                if auto_refresh {
                    let auto_refresh_duration =
                        auto_refresh_setting.get_or_insert(DEFAULT_AUTO_REFRESH_RATE);
                    let mut auto_refresh_rate = 1. / auto_refresh_duration.as_secs_f32();
                    DragValue::new(&mut auto_refresh_rate)
                        .speed(0.1)
                        .range(0.1..=10.0)
                        .fixed_decimals(1)
                        .update_while_editing(false)
                        .suffix(" Hz")
                        .ui(ui);
                    *auto_refresh_duration = Duration::from_secs_f32(1. / auto_refresh_rate);
                } else {
                    *auto_refresh_setting = None;
                }
            });
        }
    }

    fn valve_frame_ui(&self, valve: Valve) -> impl FnOnce(&mut Ui) {
        move |ui| {
            let valve_str = valve.to_string();
            let timing = self.valves_state.get_timing_for(valve);
            let aperture = self.valves_state.get_aperture_for(valve);

            let timing_str = match timing {
                valves::ParameterValue::Valid(value) => {
                    format!("{} [ms]", value)
                }
                valves::ParameterValue::Missing => "N/A".to_owned(),
                valves::ParameterValue::Invalid(err_id) => {
                    format!("ERROR: {}", err_id)
                }
            };
            let aperture_str = match aperture {
                valves::ParameterValue::Valid(value) => {
                    format!("{}", value)
                }
                valves::ParameterValue::Missing => "N/A".to_owned(),
                valves::ParameterValue::Invalid(err_id) => {
                    format!("ERROR: {}", err_id)
                }
            };

            let inside_frame = |ui: &mut Ui| {
                let text_color = ui.visuals().text_color();
                let icon_size = Vec2::splat(17.);

                StripBuilder::new(ui)
                    .sizes(Size::exact(20.), 2)
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            Label::new(
                                RichText::new(valve_str.to_ascii_uppercase())
                                    .color(text_color)
                                    .size(16.0),
                            )
                            .selectable(false)
                            .ui(ui);
                        });
                        strip.cell(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    let rect = Rect::from_min_size(ui.cursor().min, icon_size);
                                    Icon::Timing.paint(ui, rect);
                                    ui.allocate_rect(rect, Sense::hover());
                                    Label::new(format!("Timing: {}", timing_str))
                                        .selectable(false)
                                        .ui(ui);
                                });
                                ui.horizontal(|ui| {
                                    let rect = Rect::from_min_size(ui.cursor().min, icon_size);
                                    Icon::Aperture.paint(ui, rect);
                                    ui.allocate_rect(rect, Sense::hover());
                                    Label::new(format!("Aperture: {}", aperture_str))
                                        .selectable(false)
                                        .ui(ui);
                                });
                            });
                        });
                    });
            };

            ui.scope_builder(
                UiBuilder::new()
                    .id_salt("valve_".to_owned() + &valve_str)
                    .sense(Sense::click()),
                |ui| {
                    ui.set_width(200.);
                    ui.set_height(80.);
                    let response = ui.response();
                    let visuals = ui.style().interact(&response);

                    let fill_color = if response.hovered() {
                        visuals.bg_fill
                    } else {
                        visuals.bg_fill.gamma_multiply(0.3)
                    };

                    Frame::canvas(ui.style())
                        .fill(fill_color)
                        .stroke(Stroke::NONE)
                        .inner_margin(ui.spacing().menu_margin)
                        .corner_radius(visuals.corner_radius)
                        .show(ui, inside_frame);

                    if response.clicked() {
                        info!("Clicked!");
                    }
                },
            );
        }
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
