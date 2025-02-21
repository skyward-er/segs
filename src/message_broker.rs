//! Message broker module, responsible for managing the messages received from
//! the Mavlink listener.
//!
//! The `MessageBroker` struct is the main entry point for this module, and it
//! is responsible for listening to incoming messages from the Mavlink listener,
//! storing them in a map, and updating the views that are interested in them.

mod message_subscription;

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use anyhow::{Context, Result};
use parking_lot::Mutex;
use ring_channel::{ring_channel, RingReceiver, RingSender};
use tokio::{net::UdpSocket, task::JoinHandle};
use tracing::{debug, trace};
use uuid::Uuid;

use crate::mavlink::{byte_parser, Message, TimedMessage};
pub use message_subscription::MessageSubscription;

/// Maximum size of the UDP buffer
const UDP_BUFFER_SIZE: usize = 65527;

/// MessageBroker singleton instance
pub static MESSAGE_BROKER_INSTANCE: OnceLock<Mutex<MessageBroker>> = OnceLock::new();

/// MessageBroker singleton instance getter macro
#[macro_export]
macro_rules! get_message_broker {
    () => {
        $crate::message_broker::MESSAGE_BROKER_INSTANCE
            .get()
            .log_expect("Failed to get MessageBroker instance")
            .lock()
    };
}

/// The MessageBroker struct contains the state of the message broker.
///
/// It is responsible for receiving messages from the Mavlink listener and
/// dispatching them to the views that are interested in them.
#[derive(Debug)]
pub struct MessageBroker {
    /// A map of all messages received so far, indexed by message ID
    pub(super) messages: HashMap<u32, Vec<TimedMessage>>,
    /// A map of all subscriber queues, indexed by subscription ID
    pub(super) subscriber_queues: HashMap<Uuid, (u32, Vec<TimedMessage>)>,

    /// Flag to stop the listener
    running_flag: Arc<AtomicBool>,
    /// Listener message sender
    tx: RingSender<TimedMessage>,
    /// Broker message receiver
    rx: RingReceiver<TimedMessage>,
    /// Task handle for the listener
    task: Option<JoinHandle<Result<()>>>,
    /// Egui context
    ctx: egui::Context,
}

impl MessageBroker {
    /// Creates a new `MessageBroker` with the given channel size and Egui context.
    pub fn new(channel_size: NonZeroUsize, ctx: egui::Context) -> Self {
        let (tx, rx) = ring_channel(channel_size);
        Self {
            messages: HashMap::new(),
            subscriber_queues: HashMap::new(),
            tx,
            rx,
            ctx,
            running_flag: Arc::new(AtomicBool::new(false)),
            task: None,
        }
    }

    /// Stop the listener task from listening to incoming messages, if it is
    /// running.
    pub fn stop_listening(&mut self) {
        self.running_flag.store(false, Ordering::Relaxed);
        if let Some(t) = self.task.take() {
            t.abort()
        }
    }

    /// Start a listener task that listens to incoming messages from the given
    /// Ethernet port, and accumulates them in a ring buffer, read only when
    /// views request a refresh.
    pub fn listen_from_ethernet_port(&mut self, port: u16) {
        // Stop the current listener if it exists
        self.stop_listening();
        self.running_flag.store(true, Ordering::Relaxed);

        let tx = self.tx.clone();
        let ctx = self.ctx.clone();

        let bind_address = format!("0.0.0.0:{}", port);
        let mut buf = Box::new([0; UDP_BUFFER_SIZE]);
        let running_flag = self.running_flag.clone();

        debug!("Spawning listener task at {}", bind_address);
        let handle = tokio::spawn(async move {
            let socket = UdpSocket::bind(bind_address)
                .await
                .context("Failed to bind socket")?;
            debug!("Listening on UDP");

            while running_flag.load(Ordering::Relaxed) {
                let (len, _) = socket
                    .recv_from(buf.as_mut_slice())
                    .await
                    .context("Failed to receive message")?;
                for (_, mav_message) in byte_parser(&buf[..len]) {
                    debug!("Received message: {:?}", mav_message);
                    tx.send(TimedMessage::just_received(mav_message))
                        .context("Failed to send message")?;
                    // TODO: request repaint after parsing all messages, not after each one
                    ctx.request_repaint();
                }
            }

            Ok::<(), anyhow::Error>(())
        });
        self.task = Some(handle);
    }

    /// Process the incoming messages from the Mavlink listener, storing them in
    /// the messages map and updating the subscriber queues.
    pub fn process_incoming_msgs(&mut self) {
        // TODO: remove this and move into the task loop
        while let Ok(message) = self.rx.try_recv() {
            debug!(
                "processing received message: {:?}",
                message.message.message_name()
            );

            // Update subscriber queues
            for (_, (id, queue)) in self.subscriber_queues.iter_mut() {
                if *id == message.message.message_id() {
                    queue.push(message.clone());
                }
            }

            // then store the message in the messages map
            self.messages
                .entry(message.message.message_id())
                .or_default()
                .push(message);
        }
    }

    /// Subscribe to a message ID, returning a `MessageSubscription` that can be
    /// used to handle the received messages.
    pub fn subscribe(&mut self, message_id: u32) -> MessageSubscription {
        let subscription = MessageSubscription::new(message_id);
        self.subscriber_queues
            .insert(subscription.id, (message_id, Vec::new()));
        trace!("Created subscription: {:?}", subscription);
        subscription
    }

    /// Unsubscribe a message subscription.
    /// This method is called automatically when the subscription is dropped,
    /// and should not be called manually.
    pub(super) fn unsubscribe(&mut self, subscription: &MessageSubscription) {
        trace!("Destroying subscription: {:?}", subscription);
        self.subscriber_queues.remove(&subscription.id);
    }

    /// Removes all the subscriptions from the broker.
    pub fn unsubscribe_all(&mut self) {
        self.subscriber_queues.clear();
    }

    // TODO: Implement a scheduler removal of old messages (configurable, must not hurt performance)
    // TODO: Add a Dashmap if performance is a problem (Personally don't think it will be)
}
