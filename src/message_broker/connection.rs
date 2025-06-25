use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use egui::mutex::RwLock;
use tracing::{trace, warn};

use crate::communication::{
    Connection, ConnectionError, EthernetConfiguration, SerialConfiguration, TransceiverConfig,
};

/// The `ConneactionHandler` handles and manages the connection to the Mavlink listener.
///
/// If the connection is lost, it will attempt to reconnect automatically.
pub struct ConnectionHandler {
    /// Polling interval for the connection handler
    polling_interval: Duration,
    /// Connection configuration settings
    connection_config: Arc<RwLock<Option<ConnectionConfig>>>,
    /// Connection to the Mavlink listener
    pub connection: Arc<RwLock<Option<Connection>>>,
    /// Stable reconnection thread handle
    thread_handle: Option<JoinHandle<()>>,
    /// Flag to indicate if the connection is currently active
    open: Arc<AtomicBool>,
}

impl ConnectionHandler {
    pub fn new(polling_interval: Duration) -> Self {
        Self {
            polling_interval,
            connection_config: Arc::new(RwLock::new(None)),
            connection: Arc::new(RwLock::new(None)),
            thread_handle: None,
            open: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Spawn a thread that keeps reconnecting to the Mavlink listener.
    pub fn spawn_handler(&mut self) {
        if self.thread_handle.is_none() {
            self.thread_handle = Some(thread::spawn({
                let config = self.connection_config.clone();
                let connection = self.connection.clone();
                let open = self.open.clone();
                let polling_interval = self.polling_interval;
                move || {
                    loop {
                        let connection_is_open = connection.read().is_some();
                        match (
                            config.read().as_ref(),
                            connection_is_open,
                            open.load(Ordering::Relaxed),
                        ) {
                            (Some(cfg), false, true) => match cfg.open_connection() {
                                Ok(conn) => {
                                    trace!("Connection opened successfully.");
                                    connection.write().replace(conn);
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to open connection: {:?}. Will retry in {} seconds.",
                                        e,
                                        polling_interval.as_secs()
                                    );
                                }
                            },
                            (Some(_), true, true) => {
                                // If we already have a connection, we can skip opening it again
                                trace!("Connection already established, waiting for next poll.");
                            }
                            (_, _, true) => {
                                // No configuration set, cannot open connection
                                trace!("No connection configuration set. Waiting for next poll.");
                            }
                            (_, _, false) => {
                                // Connection is closed, do not attempt to reconnect
                            }
                        }
                        thread::sleep(polling_interval);
                    }
                }
            }));
        }
    }

    pub fn open_connection(&mut self, config: ConnectionConfig) {
        self.connection_config.write().replace(config);
        self.open.store(true, Ordering::Relaxed);
    }

    pub fn close_connection(&mut self) {
        self.open.store(false, Ordering::Relaxed);
        self.connection.write().take();
    }

    pub fn is_connected(&self) -> bool {
        self.connection.read().is_some() && self.open.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionConfig {
    Ethernet(EthernetConfiguration),
    Serial(SerialConfiguration),
}

impl From<EthernetConfiguration> for ConnectionConfig {
    fn from(config: EthernetConfiguration) -> Self {
        ConnectionConfig::Ethernet(config)
    }
}

impl From<SerialConfiguration> for ConnectionConfig {
    fn from(config: SerialConfiguration) -> Self {
        ConnectionConfig::Serial(config)
    }
}

impl ConnectionConfig {
    fn open_connection(&self) -> Result<Connection, ConnectionError> {
        match self {
            ConnectionConfig::Ethernet(config) => config.open_connection(),
            ConnectionConfig::Serial(config) => config.open_connection(),
        }
    }
}
