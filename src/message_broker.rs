//! Message broker module, responsible for managing the messages received from
//! the Mavlink listener.
//!
//! The `MessageBroker` struct is the main entry point for this module, and it
//! is responsible for listening to incoming messages from the Mavlink listener,
//! storing them in a map, and updating the views that are interested in them.

mod connection;
mod message_bundle;
mod reception_queue;

use egui::mutex::Mutex;
use reception_queue::ReceptionQueue;

use std::{sync::Arc, time::Duration};

use tracing::{error, warn};

use crate::mavlink::{MavFrame, MavHeader, MavMessage, MavlinkVersion, TimedMessage};
pub use connection::ConnectionConfig;
use connection::ConnectionHandler;
pub use message_bundle::MessageBundle;

const RECEPTION_QUEUE_INTERVAL: Duration = Duration::from_secs(3);
const RECONNECT_INTERVAL: Duration = Duration::from_secs(1);

/// The MessageBroker struct contains the state of the message broker.
///
/// It is responsible for receiving messages from the Mavlink listener and
/// dispatching them to the views that are interested in them.
pub struct MessageBroker {
    /// A map of all messages received so far, indexed by message ID
    messages: Vec<TimedMessage>,
    /// instant queue used for frequency calculation and reception time
    last_receptions: Arc<Mutex<ReceptionQueue>>,
    /// Connection to the Mavlink listener
    connection: ConnectionHandler,
    /// Egui context
    ctx: egui::Context,
}

impl MessageBroker {
    /// Creates a new `MessageBroker` with the given channel size and Egui context.
    pub fn new(ctx: egui::Context) -> Self {
        Self {
            messages: Vec::new(),
            // TODO: make this configurable
            last_receptions: Arc::new(Mutex::new(ReceptionQueue::new(RECEPTION_QUEUE_INTERVAL))),
            connection: ConnectionHandler::new(RECONNECT_INTERVAL),
            ctx,
        }
    }

    /// Start a listener task that listens to incoming messages from the given
    /// medium (Serial or Ethernet) and stores them in a ring buffer.
    pub fn open_connection(&mut self, config: ConnectionConfig) {
        self.connection.open_connection(config);
        self.connection.spawn_handler();
    }

    /// Stop the listener task from listening to incoming messages, if it is
    /// running.
    pub fn close_connection(&mut self) {
        self.connection.close_connection();
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_connected()
    }

    /// Returns the time since the last message was received.
    pub fn time_since_last_reception(&self) -> Option<Duration> {
        self.last_receptions.lock().time_since_last_reception()
    }

    /// Returns the frequency of messages received in the last second.
    pub fn reception_frequency(&self) -> f64 {
        self.last_receptions.lock().frequency()
    }

    pub fn get(&self, ids: &[u32]) -> Vec<&TimedMessage> {
        self.messages
            .iter()
            .filter(|msg| ids.contains(&msg.id()))
            .collect()
    }

    /// Processes incoming network messages. New messages are added to the
    /// given `MessageBundle`.
    #[profiling::function]
    pub fn process_incoming_messages(&mut self, bundle: &mut MessageBundle) {
        // process messages only if the connection is open
        if let Some(connection) = self.connection.connection.read().as_ref() {
            // check for communication errors, and log them
            match connection.retrieve_messages() {
                Ok(messages) => {
                    for message in messages {
                        bundle.insert(message.clone());

                        // Update the last reception time
                        self.last_receptions.lock().push(message.time);

                        // Store the message in the broker
                        self.messages.push(message);
                    }
                    self.ctx.request_repaint();
                }
                Err(e) => {
                    error!("Error while receiving messages: {:?}", e);
                    // TODO: user error handling, until them silently close the connection
                }
            }
        }
    }

    /// Processes outgoing messages.
    ///
    /// **WARNING**: This methods blocks the UI, thus a detailed profiling is needed.
    // FIXME
    #[profiling::function]
    pub fn process_outgoing_messages(&mut self, messages: Vec<(MavHeader, MavMessage)>) {
        if let Some(connection) = self.connection.connection.read().as_ref() {
            for (header, msg) in messages {
                let frame = MavFrame {
                    header,
                    msg,
                    protocol_version: MavlinkVersion::V1,
                };
                if let Err(e) = connection.send_message(frame) {
                    error!("Error while transmitting message: {:?}", e);
                }
            }
        }
    }

    // TODO: Implement a scheduler removal of old messages (configurable, must not hurt performance)
    // TODO: Add a Dashmap if performance is a problem (Personally don't think it will be)
}
