mod error;
pub mod ethernet;
pub mod serial;

use std::{
    num::NonZero,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use enum_dispatch::enum_dispatch;
use ring_channel::{RingReceiver, TryRecvError, ring_channel};
use skyward_mavlink::mavlink::{
    MavFrame,
    error::{MessageReadError, MessageWriteError},
};

use crate::{
    error::ErrInstrument,
    mavlink::{MavMessage, TimedMessage},
};

use ethernet::EthernetTransceiver;
use serial::SerialTransceiver;

// Re-exports
pub use error::{CommunicationError, ConnectionError};
pub use ethernet::EthernetConfiguration;
pub use serial::SerialConfiguration;

const MAX_STORED_MSGS: usize = 1000; // 192 bytes each = 192 KB

pub trait TransceiverConfigExt: Connectable {
    fn open_connection(&self) -> Result<Connection, ConnectionError> {
        Ok(self.connect()?.connect_transceiver())
    }
}

impl<T: Connectable> TransceiverConfigExt for T {}

trait Connectable {
    type Connected: MessageTransceiver;

    fn connect(&self) -> Result<Self::Connected, ConnectionError>;
}

#[enum_dispatch(Transceivers)]
trait MessageTransceiver: Send + Sync + Into<Transceivers> {
    /// Reads a message from the serial port, blocking until a valid message is received.
    /// This method ignores timeout errors and continues trying.
    fn wait_for_message(&self) -> Result<TimedMessage, MessageReadError>;

    /// Transmits a message over the serial connection.
    fn transmit_message(&self, msg: MavFrame<MavMessage>) -> Result<usize, MessageWriteError>;

    /// Opens a connection to the transceiver and returns a handle to it.
    fn connect_transceiver(self) -> Connection {
        let running_flag = Arc::new(AtomicBool::new(true));
        let (tx, rx) = ring_channel(NonZero::new(MAX_STORED_MSGS).log_unwrap());
        let endpoint_inner = Arc::new(self.into());

        {
            let running_flag = running_flag.clone();
            let endpoint_inner = endpoint_inner.clone();
            // detach the thread, to see errors rely on logs
            let _ = std::thread::spawn(move || {
                while running_flag.load(Ordering::Relaxed) {
                    match endpoint_inner.wait_for_message() {
                        Ok(msg) => {
                            tx.send(msg)
                                .map_err(|_| CommunicationError::ConnectionClosed)?;
                        }
                        Err(MessageReadError::Io(e)) => {
                            tracing::error!("Failed to read message: {e:#?}");
                            running_flag.store(false, Ordering::Relaxed);
                            return Err(CommunicationError::Io(e));
                        }
                        Err(MessageReadError::Parse(e)) => {
                            tracing::error!("Failed to read message: {e:#?}");
                        }
                    }
                }
                Ok(())
            });
        }

        Connection {
            endpoint: endpoint_inner,
            rx_ring_channel: rx,
            running_flag,
        }
    }
}

#[enum_dispatch]
enum Transceivers {
    Serial(SerialTransceiver),
    Ethernet(EthernetTransceiver),
}

pub struct Connection {
    endpoint: Arc<Transceivers>,
    rx_ring_channel: RingReceiver<TimedMessage>,
    running_flag: Arc<AtomicBool>,
}

impl Connection {
    /// Retrieves and clears the stored messages.
    pub fn retrieve_messages(&self) -> Result<Vec<TimedMessage>, CommunicationError> {
        // otherwise retrieve all messages from the buffer and return them
        let mut stored_msgs = Vec::new();
        loop {
            match self.rx_ring_channel.try_recv() {
                Ok(msg) => {
                    // Store the message in the buffer.
                    stored_msgs.push(msg);
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    return Err(CommunicationError::ConnectionClosed);
                }
            }
        }
        Ok(stored_msgs)
    }

    /// Send a message over the serial connection.
    pub fn send_message(&self, msg: MavFrame<MavMessage>) -> Result<(), CommunicationError> {
        self.endpoint.transmit_message(msg)?;
        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.running_flag.store(false, Ordering::Relaxed);
    }
}
