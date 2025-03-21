use crate::mavlink::TimedMessage;

/// A bundle of messages, indexed by their ID.
/// Allows for efficient storage and retrieval of messages by ID.
///
/// # Note
///
/// This structure performed best when reusing the same instance for multiple
/// iterations, instead of creating a new instance every time. Use the [`reset`]
/// method to clear the content of the bundle and prepare it for reuse.
#[derive(Default)]
pub struct MessageBundle {
    storage: Vec<TimedMessage>,
    count: u32,
}

impl MessageBundle {
    /// Returns all messages of the given ID contained in the bundle.
    pub fn get(&self, ids: &[u32]) -> Vec<&TimedMessage> {
        self.storage
            .iter()
            .filter(|msg| ids.contains(&msg.id()))
            .collect()
    }

    /// Inserts a new message into the bundle.
    pub fn insert(&mut self, message: TimedMessage) {
        self.storage.push(message);
        self.count += 1;
    }

    /// Returns the number of messages in the bundle.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Resets the content of the bundle, preparing it to be efficiently reused.
    /// Effectively, it clears the content of the bundle.
    pub fn reset(&mut self) {
        self.storage.clear();
        self.count = 0;
    }
}
