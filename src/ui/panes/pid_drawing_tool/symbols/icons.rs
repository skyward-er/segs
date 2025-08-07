mod motor_valve;

use std::fmt::Display;

use egui::{ImageSource, Theme, Ui};
use glam::Vec2;
use motor_valve::MotorValve;
use serde::{Deserialize, Serialize};

use crate::{mavlink::MavMessage, ui::utils::glam_to_egui};

use super::SymbolBehavior;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug, Default)]
pub enum Icon {
    #[default]
    Arrow,
    BurstDisk,
    CheckValve,
    FlexibleConnection,
    ManualValve,
    MotorValve(MotorValve),
    PressureGauge,
    PressureRegulator,
    PressureTransducer,
    QuickConnector,
    ReliefValve,
    Vessel,
}

impl Icon {
    pub fn iter() -> Vec<Self> {
        vec![
            Icon::Arrow,
            Icon::BurstDisk,
            Icon::CheckValve,
            Icon::FlexibleConnection,
            Icon::ManualValve,
            Icon::MotorValve(MotorValve::default_two_way()),
            Icon::MotorValve(MotorValve::default_three_way()),
            Icon::PressureGauge,
            Icon::PressureRegulator,
            Icon::PressureTransducer,
            Icon::QuickConnector,
            Icon::ReliefValve,
            Icon::Vessel,
        ]
    }

    pub fn get_image(&self, theme: Theme) -> ImageSource {
        match (&self, theme) {
            (Icon::Arrow, Theme::Light) => {
                egui::include_image!("../../../../../icons/pid_symbols/light/arrow.svg")
            }
            (Icon::Arrow, Theme::Dark) => {
                egui::include_image!("../../../../../icons/pid_symbols/dark/arrow.svg")
            }
            (Icon::BurstDisk, Theme::Light) => {
                egui::include_image!("../../../../../icons/pid_symbols/light/burst_disk.svg")
            }
            (Icon::BurstDisk, Theme::Dark) => {
                egui::include_image!("../../../../../icons/pid_symbols/dark/burst_disk.svg")
            }
            (Icon::ManualValve, Theme::Light) => {
                egui::include_image!("../../../../../icons/pid_symbols/light/manual_valve.svg")
            }
            (Icon::ManualValve, Theme::Dark) => {
                egui::include_image!("../../../../../icons/pid_symbols/dark/manual_valve.svg")
            }
            (Icon::CheckValve, Theme::Light) => {
                egui::include_image!("../../../../../icons/pid_symbols/light/check_valve.svg")
            }
            (Icon::CheckValve, Theme::Dark) => {
                egui::include_image!("../../../../../icons/pid_symbols/dark/check_valve.svg")
            }
            (Icon::ReliefValve, Theme::Light) => {
                egui::include_image!("../../../../../icons/pid_symbols/light/relief_valve.svg")
            }
            (Icon::ReliefValve, Theme::Dark) => {
                egui::include_image!("../../../../../icons/pid_symbols/dark/relief_valve.svg")
            }
            (Icon::MotorValve(state), theme) => state.get_sprite(theme),
            (Icon::PressureRegulator, Theme::Light) => {
                egui::include_image!(
                    "../../../../../icons/pid_symbols/light/pressure_regulator.svg"
                )
            }
            (Icon::PressureRegulator, Theme::Dark) => {
                egui::include_image!("../../../../../icons/pid_symbols/dark/pressure_regulator.svg")
            }
            (Icon::QuickConnector, Theme::Light) => {
                egui::include_image!("../../../../../icons/pid_symbols/light/quick_connector.svg")
            }
            (Icon::QuickConnector, Theme::Dark) => {
                egui::include_image!("../../../../../icons/pid_symbols/dark/quick_connector.svg")
            }
            (Icon::PressureTransducer, Theme::Light) => {
                egui::include_image!(
                    "../../../../../icons/pid_symbols/light/pressure_transducer.svg"
                )
            }
            (Icon::PressureTransducer, Theme::Dark) => {
                egui::include_image!(
                    "../../../../../icons/pid_symbols/dark/pressure_transducer.svg"
                )
            }
            (Icon::PressureGauge, Theme::Light) => {
                egui::include_image!("../../../../../icons/pid_symbols/light/pressure_gauge.svg")
            }
            (Icon::PressureGauge, Theme::Dark) => {
                egui::include_image!("../../../../../icons/pid_symbols/dark/pressure_gauge.svg")
            }
            (Icon::FlexibleConnection, Theme::Light) => {
                egui::include_image!(
                    "../../../../../icons/pid_symbols/light/flexible_connection.svg"
                )
            }
            (Icon::FlexibleConnection, Theme::Dark) => {
                egui::include_image!(
                    "../../../../../icons/pid_symbols/dark/flexible_connection.svg"
                )
            }
            (Icon::Vessel, Theme::Light) => {
                egui::include_image!("../../../../../icons/pid_symbols/light/vessel.svg")
            }
            (Icon::Vessel, Theme::Dark) => {
                egui::include_image!("../../../../../icons/pid_symbols/dark/vessel.svg")
            }
        }
    }
}

impl Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Icon::Arrow => write!(f, "Arrow"),
            Icon::BurstDisk => write!(f, "Burst Disk"),
            Icon::CheckValve => write!(f, "Check Valve"),
            Icon::FlexibleConnection => write!(f, "Flexible Connection"),
            Icon::ManualValve => write!(f, "Manual Valve"),
            Icon::MotorValve(internal) => match internal.variant {
                motor_valve::MotorValveVariant::TwoWay(_) => write!(f, "Two-Way Motor Valve"),
                motor_valve::MotorValveVariant::ThreeWay(_) => write!(f, "Three-Way Motor Valve"),
            },
            Icon::PressureGauge => write!(f, "Pressure Gauge"),
            Icon::PressureRegulator => write!(f, "Pressure Regulator"),
            Icon::PressureTransducer => write!(f, "Pressure Transducer"),
            Icon::QuickConnector => write!(f, "Quick Connector"),
            Icon::ReliefValve => write!(f, "Relief Valve"),
            Icon::Vessel => write!(f, "Vessel"),
        }
    }
}

impl SymbolBehavior for Icon {
    fn update(&mut self, message: &MavMessage, subscribed_msg_ids: &[u32]) {
        if let Icon::MotorValve(state) = self {
            state.update(message, subscribed_msg_ids)
        }
    }

    fn reset_subscriptions(&mut self) {
        if let Icon::MotorValve(state) = self {
            state.reset_subscriptions()
        }
    }

    fn paint(&mut self, ui: &mut Ui, theme: Theme, pos: glam::Vec2, size: f32, rotation: f32) {
        let center = glam_to_egui(pos).to_pos2();
        let image_rect = egui::Rect::from_min_size(center, glam_to_egui(self.size() * size));
        egui::Image::new(self.get_image(theme))
            .rotate(rotation, egui::Vec2::splat(0.0))
            .paint_at(ui, image_rect);
    }

    fn subscriptions_ui(&mut self, ui: &mut Ui, mavlink_ids: &[u32]) {
        if let Icon::MotorValve(state) = self {
            state.subscriptions_ui(ui, mavlink_ids)
        }
    }

    fn context_menu(&mut self, ui: &mut Ui) {
        if let Icon::MotorValve(state) = self {
            if ui.button("Icon subscription settings…").clicked() {
                state.is_subs_window_visible = true;
                ui.close_menu();
            }
        }
    }

    fn anchor_points(&self) -> Option<Vec<glam::Vec2>> {
        Some(
            match self {
                Icon::Arrow => vec![(0.0, 2.0), (4.0, 2.0)],
                Icon::BurstDisk => vec![(0.0, 3.0), (4.0, 3.0)],
                Icon::CheckValve => vec![(0.0, 2.5), (10.0, 2.5)],
                Icon::FlexibleConnection => vec![(0.0, 3.0), (10.0, 3.0)],
                Icon::ManualValve => vec![(0.0, 2.5), (10.0, 2.5)],
                Icon::MotorValve(mv) => match mv.variant {
                    motor_valve::MotorValveVariant::TwoWay(_) => vec![(0.0, 5.0), (10.0, 5.0)],
                    motor_valve::MotorValveVariant::ThreeWay(_) => {
                        vec![(0.0, 3.0), (10.0, 3.0), (5.0, 8.0)]
                    }
                },
                Icon::PressureGauge => vec![(3.5, 7.0)],
                Icon::PressureRegulator => vec![(0.0, 7.0), (10.0, 7.0)],
                Icon::PressureTransducer => vec![(3.5, 7.0)],
                Icon::QuickConnector => vec![(0.0, 2.5), (6.0, 2.5)],
                Icon::ReliefValve => vec![(3.0, 10.0)],
                Icon::Vessel => vec![(0.0, 7.6), (8.2, 7.6), (4.1, 0.0), (4.1, 15.1)],
            }
            .iter()
            .map(|&p| p.into())
            .collect(),
        )
    }

    fn size(&self) -> Vec2 {
        match self {
            Icon::Arrow => (4.0, 4.0),
            Icon::BurstDisk => (4.0, 6.0),
            Icon::CheckValve => (10.0, 5.0),
            Icon::FlexibleConnection => (10.0, 6.0),
            Icon::ManualValve => (10.0, 5.0),
            Icon::MotorValve(mv) => match mv.variant {
                motor_valve::MotorValveVariant::TwoWay(_) => (10.0, 8.0),
                motor_valve::MotorValveVariant::ThreeWay(_) => (10.0, 8.0),
            },
            Icon::PressureGauge => (7.0, 7.0),
            Icon::PressureRegulator => (10.0, 10.0),
            Icon::PressureTransducer => (7.0, 7.0),
            Icon::QuickConnector => (6.0, 5.0),
            Icon::ReliefValve => (6.0, 10.0),
            Icon::Vessel => (8.2, 15.2),
        }
        .into()
    }
}
