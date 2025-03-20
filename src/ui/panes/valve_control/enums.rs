use std::fmt::Display;

use strum_macros::EnumIter;

use crate::mavlink::{
    MessageData, SET_ATOMIC_VALVE_TIMING_TC_DATA, SET_VALVE_MAXIMUM_APERTURE_TC_DATA, Servoslist,
};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
pub enum ValveCommands {
    AtomicValveTiming,
    ValveMaximumAperture,
}

impl From<ValveCommands> for u32 {
    fn from(command: ValveCommands) -> u32 {
        match command {
            ValveCommands::AtomicValveTiming => SET_ATOMIC_VALVE_TIMING_TC_DATA::ID,
            ValveCommands::ValveMaximumAperture => SET_VALVE_MAXIMUM_APERTURE_TC_DATA::ID,
        }
    }
}
