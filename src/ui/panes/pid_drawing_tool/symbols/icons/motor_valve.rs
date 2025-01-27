use crate::mavlink::{extract_from_message, MavlinkResult, MessageView, TimedMessage, ViewId};

use super::MavlinkValue;

use serde::{Deserialize, Serialize};
use skyward_mavlink::{mavlink::MessageData, orion};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct MotorValve {
    source: MavlinkValue,

    /// false = closed, true = open
    pub last_value: Option<bool>,
}

impl Default for MotorValve {
    fn default() -> Self {
        Self {
            source: MavlinkValue {
                msg_id: orion::GSE_TM_DATA::ID,
                field: "n2o_filling_valve_state".to_string(),
                view_id: ViewId::new(),
            },
            last_value: None,
        }
    }
}

impl MessageView for MotorValve {
    fn view_id(&self) -> crate::mavlink::ViewId {
        self.source.view_id
    }

    fn id_of_interest(&self) -> u32 {
        self.source.msg_id
    }

    fn is_valid(&self) -> bool {
        self.last_value.is_some()
    }

    fn populate_view(&mut self, msg_slice: &[TimedMessage]) -> MavlinkResult<()> {
        self.update_view(msg_slice)
    }

    fn update_view(&mut self, msg_slice: &[TimedMessage]) -> MavlinkResult<()> {
        if let Some(msg) = msg_slice.last() {
            let values: MavlinkResult<Vec<Option<u8>>> =
                extract_from_message(&msg.message, [&self.source.field]);
            if let Ok(values) = values {
                if !values.is_empty() {
                    if let Some(value) = values[0].map(|v| v != 0) {
                        self.last_value = Some(value);
                    }
                }
            }
        }
        Ok(())
    }
}
