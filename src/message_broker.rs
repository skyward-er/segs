mod connection;
mod message_bundle;
mod reception_queue;

use egui::mutex::Mutex;
use reception_queue::ReceptionQueue;

use std::{sync::Arc, time::Duration};

use tracing::error;

use crate::mavlink::{CommandPacket, TimedMessage};
pub use connection::ConnectionConfig;
use connection::ConnectionHandler;
pub use message_bundle::MessageBundle;

const RECEPTION_QUEUE_INTERVAL: Duration = Duration::from_secs(3);
const RECONNECT_INTERVAL: Duration = Duration::from_secs(1);

pub struct MessageBroker {
    /// All telemetry messages received so far (used for plot history replay).
    history: Vec<TimedMessage>,
    last_receptions: Arc<Mutex<ReceptionQueue>>,
    connection: ConnectionHandler,
    ctx: egui::Context,
    /// Monotonically increasing sequence counter for outgoing commands.
    cmd_seq: u16,
}

impl MessageBroker {
    pub fn new(ctx: egui::Context) -> Self {
        Self {
            history: Vec::new(),
            last_receptions: Arc::new(Mutex::new(ReceptionQueue::new(RECEPTION_QUEUE_INTERVAL))),
            connection: ConnectionHandler::new(RECONNECT_INTERVAL),
            ctx,
            cmd_seq: 0,
        }
    }

    pub fn open_connection(&mut self, config: ConnectionConfig) {
        self.connection.open_connection(config);
        self.connection.spawn_handler();
    }

    pub fn close_connection(&mut self) {
        self.connection.close_connection();
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_connected()
    }

    pub fn time_since_last_reception(&self) -> Option<Duration> {
        self.last_receptions.lock().time_since_last_reception()
    }

    pub fn reception_frequency(&self) -> f64 {
        self.last_receptions.lock().frequency()
    }

    /// All historical telemetry messages (for panes that need to replay history).
    pub fn get_history(&self) -> &[TimedMessage] {
        &self.history
    }

    #[profiling::function]
    pub fn process_incoming_messages(&mut self, bundle: &mut MessageBundle) {
        if let Some(connection) = self.connection.connection.read().as_ref() {
            match connection.retrieve_messages() {
                Ok(messages) => {
                    for message in messages {
                        bundle.insert(message.clone());
                        self.last_receptions.lock().push(message.time);
                        self.history.push(message);
                    }
                    self.ctx.request_repaint();
                }
                Err(e) => {
                    error!("Error while receiving messages: {:?}", e);
                }
            }
        }
    }

    /// Encode and transmit each command packet over the active connection.
    #[profiling::function]
    pub fn process_outgoing_messages(&mut self, commands: Vec<CommandPacket>) {
        use crate::ccsds::encode_command;
        let registry = match crate::mavlink::MAVLINK_PROFILE.get() {
            Some(r) => r,
            None => return,
        };
        // Commands are sent to APID 0 (conventional for commands without
        // a specific APID — adjust if the target firmware requires a different value).
        const CMD_APID: u16 = 0;

        if let Some(connection) = self.connection.connection.read().as_ref() {
            for cmd in commands {
                let def = registry
                    .commands
                    .iter()
                    .find(|c| c.command_id == cmd.command_id);
                let Some(def) = def else {
                    error!("Unknown command id {:#010x}", cmd.command_id);
                    continue;
                };
                let bytes = encode_command(&cmd, def, CMD_APID, self.cmd_seq);
                self.cmd_seq = self.cmd_seq.wrapping_add(1);
                if let Err(e) = connection.send_message(&bytes) {
                    error!("Error transmitting command: {:?}", e);
                }
            }
        }
    }
}
