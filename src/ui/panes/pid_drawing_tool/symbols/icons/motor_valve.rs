use serde::{Deserialize, Serialize};

use crate::{
    MAVLINK_PROFILE,
    error::ErrInstrument,
    mavlink::{
        GSE_TM_DATA, MavMessage, Message, MessageData,
        reflection::{FieldLike, IndexedField},
    },
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct MotorValve {
    mavlink_field: IndexedField,

    /// false = closed, true = open
    pub last_value: Option<bool>,
}

impl MotorValve {
    pub(super) fn update(&mut self, msg: &MavMessage) {
        if msg.message_id() == GSE_TM_DATA::ID {
            let value = self.mavlink_field.extract_as_f64(msg).log_unwrap();
            self.last_value = Some(value != 0.0);
        }
    }
}

impl Default for MotorValve {
    fn default() -> Self {
        Self {
            mavlink_field: 19
                .to_mav_field(GSE_TM_DATA::ID, &MAVLINK_PROFILE)
                .log_unwrap(), // n2_filling_valve_state for GSE_TM_DATA
            last_value: None,
        }
    }
}
