use std::{
    collections::{HashMap, VecDeque},
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{Context, Result};
use egui::{ahash::HashMapExt, IdMap};
use ring_channel::{ring_channel, RingReceiver, RingSender};
use tokio::{net::UdpSocket, task::JoinHandle};
use tracing::debug;

use crate::mavlink::byte_parser;

use super::{MavMessage, Message, TimedMessage};

const UDP_BUFFER_SIZE: usize = 65527;

pub trait MessageView {
    fn widget_id(&self) -> &egui::Id;
    fn id_of_interest(&self) -> u32;
    fn is_valid(&self) -> bool;
    fn populate_view(&mut self, msg_slice: &[TimedMessage]);
    fn update_view(&mut self, msg_slice: &[TimedMessage]);
}

#[derive(Debug)]
pub struct MessageBroker {
    // == Messages ==
    /// map(message ID -> vector of messages received so far)
    messages: HashMap<u32, Vec<TimedMessage>>,
    /// map(widget ID -> queue of messages left for update)
    update_queues: IdMap<(u32, VecDeque<TimedMessage>)>,
    // == Internal ==
    /// Flag to stop the listener
    running_flag: Arc<AtomicBool>,
    /// Listener message sender
    tx: RingSender<MavMessage>,
    /// Broker message receiver
    rx: RingReceiver<MavMessage>,
    /// Task handle for the listener
    task: Option<JoinHandle<Result<()>>>,
    /// Egui context
    ctx: egui::Context,
}

impl MessageBroker {
    pub fn new(channel_size: NonZeroUsize, ctx: egui::Context) -> Self {
        let (tx, rx) = ring_channel(channel_size);
        Self {
            messages: HashMap::new(),
            update_queues: IdMap::new(),
            tx,
            rx,
            ctx,
            running_flag: Arc::new(AtomicBool::new(false)),
            task: None,
        }
    }

    pub fn refresh_view<V: MessageView>(&mut self, view: &mut V) {
        self.process_incoming_msgs();
        if !view.is_valid() {
            self.init_view(view);
        } else {
            self.update_view(view);
        }
    }

    pub fn stop_listening(&mut self) {
        self.running_flag.store(false, Ordering::Relaxed);
        if let Some(t) = self.task.take() {
            t.abort()
        }
    }

    pub fn listen_from_ethernet_port(&mut self, port: u16) {
        // Stop the current listener if it exists
        self.stop_listening();
        self.running_flag.store(true, Ordering::Relaxed);

        let tx = self.tx.clone();
        let ctx = self.ctx.clone();

        let bind_address = format!("0.0.0.0:{}", port);
        let mut buf = Box::new([0; UDP_BUFFER_SIZE]);
        let running_flag = self.running_flag.clone();

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
                    tx.send(mav_message).context("Failed to send message")?;
                    ctx.request_repaint();
                }
            }

            Ok::<(), anyhow::Error>(())
        });
        self.task = Some(handle);
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    fn init_view<V: MessageView>(&mut self, view: &mut V) {
        if let Some(messages) = self.messages.get(&view.id_of_interest()) {
            view.populate_view(messages);
        }
        self.update_queues
            .insert(*view.widget_id(), (view.id_of_interest(), VecDeque::new()));
    }

    fn update_view<V: MessageView>(&mut self, view: &mut V) {
        if let Some((_, queue)) = self.update_queues.get_mut(view.widget_id()) {
            while let Some(msg) = queue.pop_front() {
                view.update_view(&[msg]);
            }
        }
    }

    fn process_incoming_msgs(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            // first update the update queues
            for (_, (id, queue)) in self.update_queues.iter_mut() {
                if *id == message.message_id() {
                    queue.push_back(TimedMessage::just_received(message.clone()));
                }
            }
            // then store the message in the messages map
            self.messages
                .entry(message.message_id())
                .or_default()
                .push(TimedMessage::just_received(message));
        }
    }

    // TODO: Implement a scheduler removal of old messages (configurable, must not hurt performance)
    // TODO: Add a Dashmap if performance is a problem (Personally don't think it will be)
}
