use std::cell::Cell;

use tracing::debug;
use uuid::Uuid;

use super::MESSAGE_BROKER_INSTANCE;
use crate::error::ErrInstrument;
use crate::mavlink::TimedMessage;

/// A subscription to a message ID in the MessageBroker. This is an opaque
/// handle for a subscription to a specific message ID, which can then be used
/// to handle the received messages.
#[derive(Debug, Clone)]
pub struct MessageSubscription {
    /// The ID of the subscription
    pub(super) id: Uuid,
    /// The message ID that the subscription is interested in
    pub(super) message_id: u32,
    /// Whether the subscription has been initialized or not
    /// Used to know whether to feed all received messages to the subscriber
    /// the first time after subscription
    init: Cell<bool>,
}

impl MessageSubscription {
    /// Creates a new `MessageSubscription` for the given message ID.
    pub(super) fn new(message_id: u32) -> Self {
        Self {
            id: Uuid::now_v7(),
            message_id,
            init: Cell::new(false),
        }
    }

    /// Handles all received messages for this subscription, consuming them.
    pub fn handle_messages(&self, f: impl FnOnce(&[TimedMessage])) {
        let mut broker = MESSAGE_BROKER_INSTANCE
            .get()
            .log_expect("Unable to get MessageBroker")
            .lock();

        // Send all messages the first time after subscription
        if !self.init.get() {
            let messages = broker.messages.get(&self.message_id);

            if let Some(messages) = messages {
                debug!(
                    "Initializing subscriber {} with {} messages",
                    self.id,
                    messages.len()
                );
                f(messages);
            }

            // Some messages might have been received between sub creation and this call
            // hence some messages might end up both in the global message map and in the
            // subscriber's queue. We need to clear the subscriber's queue to avoid duplicates.
            if let Some((_, queue)) = broker.subscriber_queues.get_mut(&self.id) {
                queue.clear();
            }

            self.init.set(true);
            return;
        }

        if let Some((_, queue)) = broker.subscriber_queues.get_mut(&self.id) {
            if queue.is_empty() {
                // No messages to handle
                return;
            }

            debug!("Sending {} messages to subscriber {}", queue.len(), self.id);
            f(queue);
            queue.clear();
        }
    }
}

/// Unsubscribe the subscription automatically when it is dropped.
impl Drop for MessageSubscription {
    fn drop(&mut self) {
        let mut broker = MESSAGE_BROKER_INSTANCE
            .get()
            .log_expect("Unable to get MessageBroker")
            .lock();

        broker.unsubscribe(self);
    }
}
