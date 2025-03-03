use crate::mavlink::{Message, TimedMessage};

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
    storage: Vec<(u32, Vec<TimedMessage>)>,
    count: u32,
}

impl MessageBundle {
    /// Returns all messages of the given ID contained in the bundle.
    pub fn get(&self, id: u32) -> &[TimedMessage] {
        self.storage
            .iter()
            .find(|&&(queue_id, _)| queue_id == id)
            .map_or(&[], |(_, messages)| messages.as_slice())
    }

    /// Inserts a new message into the bundle.
    pub fn insert(&mut self, message: TimedMessage) {
        let message_id = message.message.message_id();

        // Retrieve the queue for the ID, if it exists
        let maybe_queue = self
            .storage
            .iter_mut()
            .find(|&&mut (queue_id, _)| queue_id == message_id)
            .map(|(_, queue)| queue);

        if let Some(queue) = maybe_queue {
            queue.push(message);
        } else {
            self.storage.push((message_id, vec![message]));
        }

        self.count += 1;
    }

    /// Returns the number of messages in the bundle.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Resets the content of the bundle, preparing it to be efficiently reused.
    /// Effectively, it clears the content of the bundle, but with lower
    /// allocation cost the next time the bundle is reused.
    pub fn reset(&mut self) {
        // Clear the individual queues instead of the full storage, to avoid
        // the allocation cost of the already used per-id queues.
        for (_, queue) in &mut self.storage {
            queue.clear();
        }

        self.count = 0;
    }
}
