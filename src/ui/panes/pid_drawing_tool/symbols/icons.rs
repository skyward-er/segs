mod motor_valve;

use egui::{ImageSource, Theme};
use glam::Vec2;
use motor_valve::MotorValve;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

use crate::{mavlink::MavMessage, ui::utils::glam_to_egui};

use super::SymbolBehavior;

#[derive(Clone, Serialize, Deserialize, PartialEq, EnumIter, Display, Debug, Default)]
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
    ThreeWayValve,
    Vessel,
}

impl Icon {
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
            (Icon::MotorValve(state), Theme::Light) => match state.last_value {
                None => {
                    egui::include_image!("../../../../../icons/pid_symbols/light/motor_valve.svg")
                }
                Some(true) => {
                    egui::include_image!(
                        "../../../../../icons/pid_symbols/light/motor_valve_green.svg"
                    )
                }
                Some(false) => {
                    egui::include_image!(
                        "../../../../../icons/pid_symbols/light/motor_valve_red.svg"
                    )
                }
            },
            (Icon::MotorValve(state), Theme::Dark) => match state.last_value {
                None => {
                    egui::include_image!("../../../../../icons/pid_symbols/dark/motor_valve.svg")
                }
                Some(true) => {
                    egui::include_image!(
                        "../../../../../icons/pid_symbols/dark/motor_valve_green.svg"
                    )
                }
                Some(false) => {
                    egui::include_image!(
                        "../../../../../icons/pid_symbols/dark/motor_valve_red.svg"
                    )
                }
            },
            (Icon::ThreeWayValve, Theme::Light) => {
                egui::include_image!("../../../../../icons/pid_symbols/light/three_way_valve.svg")
            }
            (Icon::ThreeWayValve, Theme::Dark) => {
                egui::include_image!("../../../../../icons/pid_symbols/dark/three_way_valve.svg")
            }
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

impl SymbolBehavior for Icon {
    fn paint(
        &mut self,
        ui: &egui::Ui,
        theme: egui::Theme,
        pos: glam::Vec2,
        size: f32,
        rotation: f32,
    ) {
        let center = glam_to_egui(pos).to_pos2();
        let image_rect = egui::Rect::from_min_size(center, glam_to_egui(self.size() * size));
        egui::Image::new(self.get_image(theme))
            .rotate(rotation, egui::Vec2::splat(0.0))
            .paint_at(ui, image_rect);
    }

    fn update(&mut self, message: &MavMessage) {
        if let Icon::MotorValve(state) = self {
            state.update(message)
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
                Icon::MotorValve(_) => vec![(0.0, 5.0), (10.0, 5.0)],
                Icon::PressureGauge => vec![(3.5, 7.0)],
                Icon::PressureRegulator => vec![(0.0, 7.0), (10.0, 7.0)],
                Icon::PressureTransducer => vec![(3.5, 7.0)],
                Icon::QuickConnector => vec![(0.0, 2.5), (6.0, 2.5)],
                Icon::ReliefValve => vec![(3.0, 10.0)],
                Icon::ThreeWayValve => vec![(0.0, 3.0), (10.0, 3.0), (5.0, 8.0)],
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
            Icon::MotorValve(_) => (10.0, 8.0),
            Icon::PressureGauge => (7.0, 7.0),
            Icon::PressureRegulator => (10.0, 10.0),
            Icon::PressureTransducer => (7.0, 7.0),
            Icon::QuickConnector => (6.0, 5.0),
            Icon::ReliefValve => (6.0, 10.0),
            Icon::ThreeWayValve => (10.0, 8.0),
            Icon::Vessel => (8.2, 15.2),
        }
        .into()
    }
}
