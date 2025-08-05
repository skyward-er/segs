use std::time::{Duration, Instant};

use skyward_mavlink::orion::{GET_VALVE_INFO_TC_DATA, WIGGLE_SERVO_TC_DATA};

use crate::mavlink::{
    ACK_TM_DATA, MavMessage, MessageData, NACK_TM_DATA, SET_ATOMIC_VALVE_TIMING_TC_DATA,
    SET_VALVE_MAXIMUM_APERTURE_TC_DATA, WACK_TM_DATA,
};

use super::valves::{ParameterValue, Valve, ValveParameter};

#[derive(Debug, Clone, PartialEq)]
pub enum CommandSM {
    Request(Command),
    WaitingForResponse((Instant, Command)),
    Response((Valve, Option<ValveParameter>)),
    Consumed,
}

impl CommandSM {
    pub fn pack_and_wait(&mut self) -> Option<MavMessage> {
        match self {
            Self::Request(command) => {
                let message = MavMessage::from(command.clone());
                *self = CommandSM::WaitingForResponse((Instant::now(), command.clone()));
                Some(message)
            }
            _ => None,
        }
    }

    pub fn cancel_expired(&mut self, timeout: Duration) {
        if let Self::WaitingForResponse((instant, cmd)) = self {
            if instant.elapsed() > timeout {
                let Command { kind, valve } = cmd;
                *self = Self::Response((*valve, kind.to_missing_parameter()));
            }
        }
    }

    pub fn capture_response(&mut self, message: &MavMessage) {
        if let Self::WaitingForResponse((_, Command { kind, valve })) = self {
            let id = kind.message_id() as u8;
            match message {
                MavMessage::ACK_TM(ACK_TM_DATA { recv_msgid, .. }) if *recv_msgid == id => {
                    *self = CommandSM::Response((*valve, kind.to_valid_parameter()));
                }
                MavMessage::NACK_TM(NACK_TM_DATA {
                    err_id, recv_msgid, ..
                }) if *recv_msgid == id => {
                    *self = CommandSM::Response((*valve, kind.to_invalid_parameter(*err_id)));
                }
                MavMessage::WACK_TM(WACK_TM_DATA {
                    err_id, recv_msgid, ..
                }) if *recv_msgid == id => {
                    *self = CommandSM::Response((*valve, kind.to_invalid_parameter(*err_id)));
                }
                _ => {}
            }
        }
    }

    pub fn consume_response(&mut self) -> Option<(Valve, Option<ValveParameter>)> {
        match self {
            Self::Response((valve, parameter)) => {
                let res = Some((*valve, parameter.clone()));
                *self = CommandSM::Consumed;
                res
            }
            _ => None,
        }
    }

    pub fn is_waiting_for_response(&self) -> bool {
        matches!(self, Self::WaitingForResponse(_))
    }

    pub fn is_consumed(&self) -> bool {
        matches!(self, Self::Consumed)
    }
}

impl From<Command> for CommandSM {
    fn from(value: Command) -> Self {
        Self::Request(value)
    }
}

trait ControllableValves {
    fn set_atomic_valve_timing(self, timing: u32) -> Command;
    fn set_valve_maximum_aperture(self, aperture: f32) -> Command;
}

impl ControllableValves for Valve {
    fn set_atomic_valve_timing(self, timing: u32) -> Command {
        Command {
            kind: CommandKind::SetAtomicValveTiming(timing),
            valve: self,
        }
    }

    fn set_valve_maximum_aperture(self, aperture: f32) -> Command {
        Command {
            kind: CommandKind::SetValveMaximumAperture(aperture),
            valve: self,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    kind: CommandKind,
    valve: Valve,
}

impl Command {
    pub fn wiggle(valve: Valve) -> Self {
        Self {
            kind: CommandKind::Wiggle,
            valve,
        }
    }

    pub fn refresh(valve: Valve) -> Self {
        Self {
            kind: CommandKind::Refresh,
            valve,
        }
    }

    pub fn set_atomic_valve_timing(valve: Valve, timing: u32) -> Self {
        valve.set_atomic_valve_timing(timing)
    }

    pub fn set_valve_maximum_aperture(valve: Valve, aperture: f32) -> Self {
        valve.set_valve_maximum_aperture(aperture)
    }
}

impl From<Command> for MavMessage {
    fn from(value: Command) -> Self {
        match value.kind {
            CommandKind::Wiggle => Self::WIGGLE_SERVO_TC(WIGGLE_SERVO_TC_DATA {
                servo_id: value.valve.into(),
            }),
            CommandKind::Refresh => Self::GET_VALVE_INFO_TC(GET_VALVE_INFO_TC_DATA {
                servo_id: value.valve.into(),
            }),
            CommandKind::SetAtomicValveTiming(timing) => {
                Self::SET_ATOMIC_VALVE_TIMING_TC(SET_ATOMIC_VALVE_TIMING_TC_DATA {
                    servo_id: value.valve.into(),
                    maximum_timing: timing,
                })
            }
            CommandKind::SetValveMaximumAperture(aperture) => {
                Self::SET_VALVE_MAXIMUM_APERTURE_TC(SET_VALVE_MAXIMUM_APERTURE_TC_DATA {
                    servo_id: value.valve.into(),
                    maximum_aperture: aperture,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CommandKind {
    Wiggle,
    Refresh,
    SetAtomicValveTiming(u32),
    SetValveMaximumAperture(f32),
}

impl CommandKind {
    fn message_id(&self) -> u32 {
        match self {
            Self::Wiggle => WIGGLE_SERVO_TC_DATA::ID,
            Self::Refresh => GET_VALVE_INFO_TC_DATA::ID,
            Self::SetAtomicValveTiming(_) => SET_ATOMIC_VALVE_TIMING_TC_DATA::ID,
            Self::SetValveMaximumAperture(_) => SET_VALVE_MAXIMUM_APERTURE_TC_DATA::ID,
        }
    }

    fn to_valid_parameter(self) -> Option<ValveParameter> {
        self.try_into().ok()
    }

    fn to_missing_parameter(self) -> Option<ValveParameter> {
        match self {
            Self::Wiggle | Self::Refresh => None,
            Self::SetAtomicValveTiming(_) => {
                Some(ValveParameter::AtomicValveTiming(ParameterValue::Missing))
            }
            Self::SetValveMaximumAperture(_) => Some(ValveParameter::ValveMaximumAperture(
                ParameterValue::Missing,
            )),
        }
    }

    fn to_invalid_parameter(self, error: u16) -> Option<ValveParameter> {
        match self {
            Self::Wiggle | Self::Refresh => None,
            Self::SetAtomicValveTiming(_) => Some(ValveParameter::AtomicValveTiming(
                ParameterValue::Invalid(error),
            )),
            Self::SetValveMaximumAperture(_) => Some(ValveParameter::ValveMaximumAperture(
                ParameterValue::Invalid(error),
            )),
        }
    }
}

impl TryFrom<CommandKind> for ValveParameter {
    type Error = ();

    fn try_from(value: CommandKind) -> Result<Self, Self::Error> {
        match value {
            CommandKind::Wiggle | CommandKind::Refresh => Err(()),
            CommandKind::SetAtomicValveTiming(timing) => {
                Ok(Self::AtomicValveTiming(ParameterValue::Valid(timing)))
            }
            CommandKind::SetValveMaximumAperture(aperture) => {
                Ok(Self::ValveMaximumAperture(ParameterValue::Valid(aperture)))
            }
        }
    }
}
