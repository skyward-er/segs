mod error;
pub mod ethernet;
pub mod serial;

use std::{
    io::ErrorKind,
    num::NonZeroUsize,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use ring_channel::{RingReceiver, TryRecvError, ring_channel};

use crate::{error::ErrInstrument, mavlink::TimedMessage};

pub use error::{CommunicationError, ConnectionError};
pub use ethernet::EthernetConfiguration;
pub use serial::SerialConfiguration;

const MAX_STORED_MSGS: usize = 1000;

// ── Traits ────────────────────────────────────────────────────────────────────

pub(super) mod sealed {
    use super::ConnectionError;
    use crate::mavlink::TimedMessage;

    pub trait Connectable {
        type Connected: MessageTransceiver;
        fn connect(&self) -> Result<Self::Connected, ConnectionError>;
    }

    pub trait MessageTransceiver: Send + Sync + 'static {
        fn wait_for_message(&self) -> Result<TimedMessage, std::io::Error>;
        fn transmit_message(&self, data: &[u8]) -> Result<(), std::io::Error>;
    }
}

pub trait TransceiverConfig: sealed::Connectable {
    fn open_connection(&self) -> Result<Connection, ConnectionError> {
        Ok(Connection::new(self.connect()?))
    }
}
impl<T: sealed::Connectable> TransceiverConfig for T {}

// ── Connection ────────────────────────────────────────────────────────────────

/// Active connection: background thread fills a ring buffer; main thread drains it.
pub struct Connection {
    /// Type-erased send function; holds a strong ref to the transceiver.
    send_fn: Arc<dyn Fn(&[u8]) -> Result<(), std::io::Error> + Send + Sync>,
    rx_ring_channel: RingReceiver<TimedMessage>,
    running_flag: Arc<AtomicBool>,
}

impl Connection {
    fn new<T: sealed::MessageTransceiver>(transceiver: T) -> Self {
        let running_flag = Arc::new(AtomicBool::new(true));
        let (tx, rx) = ring_channel(NonZeroUsize::new(MAX_STORED_MSGS).log_unwrap());
        let transceiver = Arc::new(transceiver);

        // Listener thread
        {
            let flag = running_flag.clone();
            let t = transceiver.clone();
            let _ = std::thread::spawn(move || {
                loop {
                    if !flag.load(Ordering::Relaxed) {
                        break;
                    }
                    match t.wait_for_message() {
                        Ok(msg) => {
                            if tx.send(msg).is_err() {
                                break;
                            }
                        }
                        Err(e)
                            if e.kind() == ErrorKind::WouldBlock
                                || e.kind() == ErrorKind::TimedOut =>
                        {
                            // poll-timeout; check running_flag next iteration
                        }
                        Err(e) if e.kind() == ErrorKind::InvalidData => {
                            tracing::warn!("Packet decode error (skipping): {e}");
                        }
                        Err(e) => {
                            tracing::error!("Connection read error: {e}");
                            flag.store(false, Ordering::Relaxed);
                            break;
                        }
                    }
                }
            });
        }

        // Send closure captures the Arc so the transceiver stays alive
        let send_fn: Arc<dyn Fn(&[u8]) -> Result<(), std::io::Error> + Send + Sync> = {
            let t = transceiver;
            Arc::new(move |data: &[u8]| t.transmit_message(data))
        };

        Self {
            send_fn,
            rx_ring_channel: rx,
            running_flag,
        }
    }

    pub fn retrieve_messages(&self) -> Result<Vec<TimedMessage>, CommunicationError> {
        let mut stored = Vec::new();
        loop {
            match self.rx_ring_channel.try_recv() {
                Ok(msg) => stored.push(msg),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return Err(CommunicationError::ConnectionClosed);
                }
            }
        }
        Ok(stored)
    }

    pub fn send_message(&self, data: &[u8]) -> Result<(), CommunicationError> {
        (self.send_fn)(data).map_err(CommunicationError::Io)
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
