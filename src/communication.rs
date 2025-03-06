//! Main communication module.
//!
//! Provides a unified interface for handling message transmission and reception
//! through different physical connection types (e.g., serial, Ethernet).
//! It also manages connections and message buffering.

mod error;
pub mod ethernet;
pub mod serial;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use ring_channel::{RingReceiver, TryRecvError};
use sealed::MessageTransceiver;
use skyward_mavlink::mavlink::MavFrame;

use crate::mavlink::{MavMessage, TimedMessage};

// Re-exports
pub use error::{CommunicationError, ConnectionError};
pub use ethernet::EthernetConfiguration;
pub use serial::SerialConfiguration;

const MAX_STORED_MSGS: usize = 1000; // e.g., 192 bytes each = 192 KB

mod sealed {
    use std::{
        num::NonZeroUsize,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
    };

    use enum_dispatch::enum_dispatch;
    use ring_channel::ring_channel;
    use skyward_mavlink::mavlink::{
        MavFrame,
        error::{MessageReadError, MessageWriteError},
    };

    use crate::{
        error::ErrInstrument,
        mavlink::{MavMessage, TimedMessage},
    };

    use super::{
        CommunicationError, Connection, ConnectionError, MAX_STORED_MSGS,
        ethernet::EthernetTransceiver, serial::SerialTransceiver,
    };

    pub trait TransceiverConfigSealed {}

    /// Trait representing an entity that can be connected.
    pub trait Connectable {
        type Connected: MessageTransceiver;

        /// Establishes a connection based on the configuration.
        fn connect(&self) -> Result<Self::Connected, ConnectionError>;
    }

    /// Trait representing a message transceiver.
    /// This trait abstracts the common operations for message transmission and reception.
    /// It also provides a default implementation for opening a listening connection, while
    /// being transparent to the actual Transceiver type.
    #[enum_dispatch(Transceivers)]
    pub trait MessageTransceiver: Send + Sync + Into<Transceivers> {
        /// Blocks until a valid message is received.
        fn wait_for_message(&self) -> Result<TimedMessage, MessageReadError>;

        /// Transmits a message using the connection.
        fn transmit_message(&self, msg: MavFrame<MavMessage>) -> Result<usize, MessageWriteError>;

        /// Opens a listening connection and spawns a thread for message handling.
        #[profiling::function]
        fn open_listening_connection(self) -> Connection {
            let running_flag = Arc::new(AtomicBool::new(true));
            let (tx, rx) = ring_channel(NonZeroUsize::new(MAX_STORED_MSGS).log_unwrap());
            let endpoint_inner = Arc::new(self.into());

            {
                let running_flag = running_flag.clone();
                let endpoint_inner = endpoint_inner.clone();
                // Detached thread for message handling; errors are logged.
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
                transceiver: endpoint_inner,
                rx_ring_channel: rx,
                running_flag,
            }
        }
    }

    impl<T: Connectable> TransceiverConfigSealed for T {}

    /// Enum representing the different types of transceivers.
    #[enum_dispatch]
    pub(super) enum Transceivers {
        Serial(SerialTransceiver),
        Ethernet(EthernetTransceiver),
    }
}

/// Trait to abstract common configuration types.
pub trait TransceiverConfig: sealed::TransceiverConfigSealed {}
impl<T: sealed::TransceiverConfigSealed> TransceiverConfig for T {}

/// Extension trait to open a connection directly from a configuration.
pub trait TransceiverConfigExt: sealed::Connectable {
    /// Opens a connection and returns a handle to it.
    fn open_connection(&self) -> Result<Connection, ConnectionError> {
        Ok(self.connect()?.open_listening_connection())
    }
}
impl<T: sealed::Connectable> TransceiverConfigExt for T {}

/// Represents an active connection with buffered messages.
pub struct Connection {
    transceiver: Arc<sealed::Transceivers>,
    rx_ring_channel: RingReceiver<TimedMessage>,
    running_flag: Arc<AtomicBool>,
}

impl Connection {
    /// Retrieves and clears stored messages.
    #[profiling::function]
    pub fn retrieve_messages(&self) -> Result<Vec<TimedMessage>, CommunicationError> {
        let mut stored_msgs = Vec::new();
        loop {
            match self.rx_ring_channel.try_recv() {
                Ok(msg) => stored_msgs.push(msg),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return Err(CommunicationError::ConnectionClosed);
                }
            }
        }
        Ok(stored_msgs)
    }

    /// Sends a message over the connection.
    #[profiling::function]
    pub fn send_message(&self, msg: MavFrame<MavMessage>) -> Result<(), CommunicationError> {
        self.transceiver.transmit_message(msg)?;
        Ok(())
    }
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("running_flag", &self.running_flag)
            .finish()
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.running_flag.store(false, Ordering::Relaxed);
    }
}
