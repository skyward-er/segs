//! Message broker module, responsible for managing the messages received from
//! the Mavlink listener.
//!
//! The `MessageBroker` struct is the main entry point for this module, and it
//! is responsible for listening to incoming messages from the Mavlink listener,
//! storing them in a map, and updating the views that are interested in them.

mod message_bundle;
mod reception_queue;
pub use message_bundle::MessageBundle;
use reception_queue::ReceptionQueue;

use crate::{
    error::ErrInstrument,
    mavlink::{Message, TimedMessage, byte_parser},
    utils::RingBuffer,
};
use anyhow::{Context, Result};
use ring_channel::{RingReceiver, RingSender, ring_channel};
use std::{
    collections::HashMap,
    io::Write,
    num::NonZeroUsize,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{net::UdpSocket, task::JoinHandle};
use tracing::{debug, trace};

/// Maximum size of the UDP buffer
const UDP_BUFFER_SIZE: usize = 65527;

/// The MessageBroker struct contains the state of the message broker.
///
/// It is responsible for receiving messages from the Mavlink listener and
/// dispatching them to the views that are interested in them.
#[derive(Debug)]
pub struct MessageBroker {
    /// A map of all messages received so far, indexed by message ID
    messages: HashMap<u32, Vec<TimedMessage>>,
    /// instant queue used for frequency calculation and reception time
    last_receptions: Arc<Mutex<ReceptionQueue>>,
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
            // TODO: make this configurable
            last_receptions: Arc::new(Mutex::new(ReceptionQueue::new(Duration::from_secs(1)))),
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
        let last_receptions = Arc::clone(&self.last_receptions);

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
                    trace!("Received message: {:?}", mav_message);
                    tx.send(TimedMessage::just_received(mav_message))
                        .context("Failed to send message")?;
                    last_receptions.lock().unwrap().push(Instant::now());
                    ctx.request_repaint();
                }
            }

            Ok::<(), anyhow::Error>(())
        });
        self.task = Some(handle);
    }

    /// Start a listener task that listens to incoming messages from the given
    /// serial port and stores them in a ring buffer.
    pub fn listen_from_serial_port(&mut self, port: String, baud_rate: u32) {
        // Stop the current listener if it exists
        self.stop_listening();
        self.running_flag.store(true, Ordering::Relaxed);
        let last_receptions = Arc::clone(&self.last_receptions);

        let tx = self.tx.clone();
        let ctx = self.ctx.clone();

        let running_flag = self.running_flag.clone();

        debug!("Spawning listener task at {}", port);
        let handle = tokio::task::spawn_blocking(move || {
            let mut serial_port = serialport::new(port, baud_rate)
                .timeout(std::time::Duration::from_millis(100))
                .open()
                .context("Failed to open serial port")?;
            debug!("Listening on serial port");

            let mut ring_buf = RingBuffer::<1024>::new();
            let mut temp_buf = [0; 512];
            // need to do a better error handling for this (need toast errors)
            while running_flag.load(Ordering::Relaxed) {
                let result = serial_port
                    .read(&mut temp_buf)
                    .log_expect("Failed to read from serial port");
                debug!("Read {} bytes from serial port", result);
                trace!("data read from serial: {:?}", &temp_buf[..result]);
                ring_buf
                    .write(&temp_buf[..result])
                    .log_expect("Failed to write to ring buffer, check buffer size");
                for (_, mav_message) in byte_parser(&mut ring_buf) {
                    debug!("Received message: {:?}", mav_message);
                    tx.send(TimedMessage::just_received(mav_message))
                        .context("Failed to send message")?;
                    last_receptions.lock().unwrap().push(Instant::now());
                    ctx.request_repaint();
                }
            }

            Ok::<(), anyhow::Error>(())
        });
        self.task = Some(handle);
    }

    /// Returns the time since the last message was received.
    pub fn time_since_last_reception(&self) -> Option<Duration> {
        self.last_receptions
            .lock()
            .unwrap()
            .time_since_last_reception()
    }

    /// Returns the frequency of messages received in the last second.
    pub fn reception_frequency(&self) -> f64 {
        self.last_receptions.lock().unwrap().frequency()
    }

    pub fn get(&self, id: u32) -> &[TimedMessage] {
        self.messages.get(&id).map_or(&[], |v| v.as_slice())
    }

    /// Processes incoming network messages. New messages are added to the
    /// given `MessageBundle`.
    pub fn process_messages(&mut self, bundle: &mut MessageBundle) {
        while let Ok(message) = self.rx.try_recv() {
            bundle.insert(message.clone());

            // Store the message in the broker
            self.messages
                .entry(message.message.message_id())
                .or_default()
                .push(message);
        }
    }

    // TODO: Implement a scheduler removal of old messages (configurable, must not hurt performance)
    // TODO: Add a Dashmap if performance is a problem (Personally don't think it will be)
}
