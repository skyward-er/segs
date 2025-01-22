//! Message broker module, responsible for managing the messages received from
//! the Mavlink listener.
//!
//! The `MessageBroker` struct is the main entry point for this module, and it
//! is responsible for listening to incoming messages from the Mavlink listener,
//! storing them in a map, and updating the views that are interested in them.

use std::{
    collections::{HashMap, VecDeque},
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{Context, Result};
use ring_channel::{ring_channel, RingReceiver, RingSender};
use serde::{Deserialize, Serialize};
use tokio::{net::UdpSocket, task::JoinHandle};
use tracing::{debug, trace};
use uuid::Uuid;

use crate::mavlink::byte_parser;

use super::{MavlinkResult, Message, TimedMessage};

/// Maximum size of the UDP buffer
const UDP_BUFFER_SIZE: usize = 65527;

/// Trait for a view that fetch Mavlink messages.
///
/// This trait should be implemented by any view that wants to interact with the
/// `MessageBroker` and get updates on the messages it is interested in.
pub trait MessageView {
    /// Returns an hashable value as widget identifier
    fn view_id(&self) -> ViewId;
    /// Returns the message ID of interest for the view
    fn id_of_interest(&self) -> u32;
    /// Returns whether the view is cache valid or not, i.e. if it can be
    /// updated or needs to be re-populated from scratch
    fn is_valid(&self) -> bool;
    /// Populates the view with the initial messages. This method is called when
    /// the cache is invalid and the view needs to be populated from the stored
    /// map of messages
    fn populate_view(&mut self, msg_slice: &[TimedMessage]) -> MavlinkResult<()>;
    /// Updates the view with new messages. This method is called when the cache
    /// is valid, hence the view only needs to be updated with the new messages
    fn update_view(&mut self, msg_slice: &[TimedMessage]) -> MavlinkResult<()>;
}

/// Responsible for storing & dispatching the Mavlink message received.
///
/// It listens to incoming messages, stores them in a map, and updates the views
/// that are interested in them. It should be used as a singleton in the
/// application.
#[derive(Debug)]
pub struct MessageBroker {
    // == Messages ==
    /// map(message ID -> vector of messages received so far)
    messages: HashMap<u32, Vec<TimedMessage>>,
    /// map(widget ID -> queue of messages left for update)
    update_queues: HashMap<ViewId, (u32, VecDeque<TimedMessage>)>,
    // == Internal ==
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
            update_queues: HashMap::new(),
            tx,
            rx,
            ctx,
            running_flag: Arc::new(AtomicBool::new(false)),
            task: None,
        }
    }

    /// Refreshes the view given as argument. It handles automatically the cache
    /// validity based on `is_valid` method of the view.
    pub fn refresh_view<V: MessageView>(&mut self, view: &mut V) -> MavlinkResult<()> {
        self.process_incoming_msgs();
        if !view.is_valid() || !self.is_view_subscribed(view.view_id()) {
            self.init_view(view)?;
        } else {
            self.update_view(view)?;
        }
        Ok(())
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
                    ctx.request_repaint();
                }
            }

            Ok::<(), anyhow::Error>(())
        });
        self.task = Some(handle);
    }

    pub fn unsubscribe_all_views(&mut self) {
        self.update_queues.clear();
    }

    /// Clears all the messages stored in the broker. Useful in message replay
    /// scenarios.
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    fn is_view_subscribed(&self, view_id: ViewId) -> bool {
        self.update_queues.contains_key(&view_id)
    }

    /// Init a view in case of cache invalidation or first time initialization.
    fn init_view<V: MessageView>(&mut self, view: &mut V) -> MavlinkResult<()> {
        trace!("initializing view: {:?}", view.view_id());
        if let Some(messages) = self.messages.get(&view.id_of_interest()) {
            view.populate_view(messages)?;
        }
        self.update_queues
            .insert(view.view_id(), (view.id_of_interest(), VecDeque::new()));
        Ok(())
    }

    /// Update a view with new messages, used when the cache is valid.
    fn update_view<V: MessageView>(&mut self, view: &mut V) -> MavlinkResult<()> {
        trace!("updating view: {:?}", view.view_id());
        if let Some((_, queue)) = self.update_queues.get_mut(&view.view_id()) {
            while let Some(msg) = queue.pop_front() {
                view.update_view(&[msg])?;
            }
        }
        Ok(())
    }

    /// Process the incoming messages from the Mavlink listener, storing them in
    /// the messages map and updating the update queues.
    fn process_incoming_msgs(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            debug!(
                "processing received message: {:?}",
                message.message.message_name()
            );
            // first update the update queues
            for (_, (id, queue)) in self.update_queues.iter_mut() {
                if *id == message.message.message_id() {
                    queue.push_back(message.clone());
                }
            }
            // then store the message in the messages map
            self.messages
                .entry(message.message.message_id())
                .or_default()
                .push(message);
        }
    }

    // TODO: Implement a scheduler removal of old messages (configurable, must not hurt performance)
    // TODO: Add a Dashmap if performance is a problem (Personally don't think it will be)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ViewId(Uuid);

impl ViewId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for ViewId {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}
