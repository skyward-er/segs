//! Valve Control Pane
//!
//! NOTE: We assume that no more than one entity will sent messages to control valves at a time.

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{error::ErrInstrument, mavlink::Servoslist};

#[derive(Clone, Debug, PartialEq)]
pub struct ValveStateManager {
    timing_settings: Vec<(Valve, ParameterValue<u32, u16>)>,
    aperture_settings: Vec<(Valve, ParameterValue<f32, u16>)>,
}

impl Default for ValveStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ValveStateManager {
    pub fn new() -> Self {
        let aperture_settings = Valve::iter()
            .map(|valve| (valve, ParameterValue::default()))
            .collect();
        let timing_settings = Valve::iter()
            .map(|valve| (valve, ParameterValue::default()))
            .collect();
        Self {
            aperture_settings,
            timing_settings,
        }
    }

    pub fn set_parameter_of(&mut self, valve: Valve, parameter: ValveParameter) {
        match parameter {
            ValveParameter::AtomicValveTiming(parameter) => {
                if let Some((_, par)) = self.timing_settings.iter_mut().find(|(v, _)| *v == valve) {
                    *par = parameter;
                }
            }
            ValveParameter::ValveMaximumAperture(parameter) => {
                if let Some((_, par)) = self.aperture_settings.iter_mut().find(|(v, _)| *v == valve)
                {
                    *par = parameter;
                }
            }
        }
    }

    pub fn get_timing_for(&self, valve: Valve) -> ParameterValue<u32, u16> {
        let (_, par) = self
            .timing_settings
            .iter()
            .find(|(v, _)| *v == valve)
            .log_unwrap();
        par.clone()
    }

    pub fn get_aperture_for(&self, valve: Valve) -> ParameterValue<f32, u16> {
        let (_, par) = self
            .aperture_settings
            .iter()
            .find(|(v, _)| *v == valve)
            .log_unwrap();
        par.clone()
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, Hash, Serialize, Deserialize)]
pub enum Valve {
    OxFilling,
    OxRelease,
    OxVenting,
    N2Filling,
    N2Release,
    N2Quenching,
    N23Way,
    Main,
    Nitrogen,
}

impl From<Valve> for Servoslist {
    fn from(valve: Valve) -> Servoslist {
        match valve {
            Valve::OxFilling => Servoslist::OX_FILLING_VALVE,
            Valve::OxRelease => Servoslist::OX_RELEASE_VALVE,
            Valve::OxVenting => Servoslist::OX_VENTING_VALVE,
            Valve::N2Filling => Servoslist::N2_FILLING_VALVE,
            Valve::N2Release => Servoslist::N2_RELEASE_VALVE,
            Valve::N2Quenching => Servoslist::N2_QUENCHING_VALVE,
            Valve::N23Way => Servoslist::N2_3WAY_VALVE,
            Valve::Main => Servoslist::MAIN_VALVE,
            Valve::Nitrogen => Servoslist::NITROGEN_VALVE,
        }
    }
}

impl From<Valve> for u8 {
    fn from(valve: Valve) -> u8 {
        Servoslist::from(valve) as u8
    }
}

impl Display for Valve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Valve::OxFilling => write!(f, "Oxidizer Filling"),
            Valve::OxRelease => write!(f, "Oxidizer Release"),
            Valve::OxVenting => write!(f, "Oxidizer Venting"),
            Valve::N2Filling => write!(f, "Nitrogen Filling"),
            Valve::N2Release => write!(f, "Nitrogen Release"),
            Valve::N2Quenching => write!(f, "Nitrogen Quenching"),
            Valve::N23Way => write!(f, "Nitrogen 3-Way"),
            Valve::Main => write!(f, "Main"),
            Valve::Nitrogen => write!(f, "Nitrogen"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, EnumIter)]
pub enum ValveParameter {
    AtomicValveTiming(ParameterValue<u32, u16>),
    ValveMaximumAperture(ParameterValue<f32, u16>),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum ParameterValue<T, E> {
    Valid(T), // T is the valid parameter value
    #[default]
    Missing, // The parameter is missing
    Invalid(E), // E is the reason why the parameter is invalid
}

impl<T, E> ParameterValue<T, E> {
    pub fn valid_or(self, default: T) -> T {
        match self {
            Self::Valid(value) => value,
            Self::Missing => default,
            Self::Invalid(_) => default,
        }
    }
}

impl<T: Display, E: Display> Display for ParameterValue<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Valid(value) => write!(f, "{}", value),
            Self::Missing => write!(f, "MISSING"),
            Self::Invalid(error) => write!(f, "INVALID: {}", error),
        }
    }
}
