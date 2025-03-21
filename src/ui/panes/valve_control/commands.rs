use crate::mavlink::{
    ACK_TM_DATA, MavMessage, MessageData, NACK_TM_DATA, SET_ATOMIC_VALVE_TIMING_TC_DATA,
    SET_VALVE_MAXIMUM_APERTURE_TC_DATA, WACK_TM_DATA,
};

use super::valves::{ParameterValue, Valve, ValveParameter};

#[derive(Debug, Clone, PartialEq)]
pub enum CommandSM {
    Request(Command),
    WaitingForResponse(Command),
    Response((Valve, ValveParameter)),
    Consumed,
}

impl CommandSM {
    pub fn pack_and_wait(&mut self) -> Option<MavMessage> {
        match self {
            CommandSM::Request(command) => {
                let message = MavMessage::from(command.clone());
                *self = CommandSM::WaitingForResponse(command.clone());
                Some(message)
            }
            _ => None,
        }
    }

    pub fn capture_response(&mut self, message: &MavMessage) {
        if let CommandSM::WaitingForResponse(Command { kind, valve }) = self {
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

    pub fn consume_response(&mut self) -> Option<(Valve, ValveParameter)> {
        match self {
            CommandSM::Response((valve, parameter)) => {
                let res = Some((*valve, parameter.clone()));
                *self = CommandSM::Consumed;
                res
            }
            _ => None,
        }
    }

    pub fn is_waiting_for_response(&self) -> bool {
        matches!(self, CommandSM::WaitingForResponse(_))
    }

    pub fn is_consumed(&self) -> bool {
        matches!(self, CommandSM::Consumed)
    }
}

impl From<Command> for CommandSM {
    fn from(value: Command) -> Self {
        CommandSM::Request(value)
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

impl From<Command> for MavMessage {
    fn from(value: Command) -> Self {
        match value.kind {
            CommandKind::SetAtomicValveTiming(timing) => {
                MavMessage::SET_ATOMIC_VALVE_TIMING_TC(SET_ATOMIC_VALVE_TIMING_TC_DATA {
                    servo_id: value.valve.into(),
                    maximum_timing: timing,
                })
            }
            CommandKind::SetValveMaximumAperture(aperture) => {
                MavMessage::SET_VALVE_MAXIMUM_APERTURE_TC(SET_VALVE_MAXIMUM_APERTURE_TC_DATA {
                    servo_id: value.valve.into(),
                    maximum_aperture: aperture,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CommandKind {
    SetAtomicValveTiming(u32),
    SetValveMaximumAperture(f32),
}

impl CommandKind {
    fn message_id(&self) -> u32 {
        match self {
            CommandKind::SetAtomicValveTiming(_) => SET_ATOMIC_VALVE_TIMING_TC_DATA::ID,
            CommandKind::SetValveMaximumAperture(_) => SET_VALVE_MAXIMUM_APERTURE_TC_DATA::ID,
        }
    }

    fn to_valid_parameter(&self) -> ValveParameter {
        (*self).into()
    }

    fn to_invalid_parameter(&self, error: u16) -> ValveParameter {
        match self {
            CommandKind::SetAtomicValveTiming(_) => {
                ValveParameter::AtomicValveTiming(ParameterValue::Invalid(error))
            }
            CommandKind::SetValveMaximumAperture(_) => {
                ValveParameter::ValveMaximumAperture(ParameterValue::Invalid(error))
            }
        }
    }
}

impl From<CommandKind> for ValveParameter {
    fn from(value: CommandKind) -> Self {
        match value {
            CommandKind::SetAtomicValveTiming(timing) => {
                ValveParameter::AtomicValveTiming(ParameterValue::Valid(timing))
            }
            CommandKind::SetValveMaximumAperture(aperture) => {
                ValveParameter::ValveMaximumAperture(ParameterValue::Valid(aperture))
            }
        }
    }
}
