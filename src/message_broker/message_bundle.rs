use std::collections::HashMap;

use crate::mavlink::TimedMessage;

/// Holds the most-recent telemetry packet **per APID** received during a single UI frame.
///
/// Cleared at the end of each frame via [`reset`].
#[derive(Default)]
pub struct MessageBundle {
    latest: HashMap<u16, TimedMessage>,
}

impl MessageBundle {
    pub fn insert(&mut self, message: TimedMessage) {
        self.latest.insert(message.packet.apid, message);
    }

    pub fn has_new(&self) -> bool {
        !self.latest.is_empty()
    }

    /// Iterate over the latest message for each APID.
    pub fn iter_latest(&self) -> impl Iterator<Item = &TimedMessage> {
        self.latest.values()
    }

    pub fn reset(&mut self) {
        self.latest.clear();
    }
}
