use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, Sender};
use mavlink_bindgen::parser::{MavProfile, MavType};
use skyward_mavlink::{
    lyra::MavMessage,
    mavlink::{peek_reader::PeekReader, read_v1_msg, MavHeader, Message},
};
use tokio::{net::UdpSocket, task::JoinHandle};
use tracing::debug;

pub const DEFAULT_ETHERNET_PORT: u16 = 42069;
const UDP_BUFFER_SIZE: usize = 65527;

#[derive(Debug)]
pub struct MessageManager {
    messages: HashMap<u32, Vec<TimedMessage>>,
    tx: Sender<MavMessage>,
    rx: Receiver<MavMessage>,
    ctx: egui::Context,
    running_flag: Arc<AtomicBool>,
    task: Option<JoinHandle<Result<()>>>,
}

impl MessageManager {
    pub fn new(channel_size: usize, ctx: egui::Context) -> Self {
        let (tx, rx) = crossbeam_channel::bounded(channel_size);
        Self {
            messages: HashMap::new(),
            tx,
            rx,
            ctx,
            running_flag: Arc::new(AtomicBool::new(false)),
            task: None,
        }
    }

    pub fn get_message(&mut self, message_id: u32) -> Option<&[TimedMessage]> {
        while let Ok(message) = self.rx.try_recv() {
            self.add_message(message);
        }
        self.messages.get(&message_id).map(|v| v.as_slice())
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
                for (_, mav_message) in iter_messages(&buf[..len]) {
                    tx.send(mav_message).context("Failed to send message")?;
                    ctx.request_repaint();
                }
                // buf.iter_mut().for_each(|b| *b = 0);
            }

            Ok::<(), anyhow::Error>(())
        });
        self.task = Some(handle);
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    fn add_message(&mut self, message: MavMessage) {
        self.messages
            .entry(message.message_id())
            .or_default()
            .push(TimedMessage::just_received(message));
    }

    // TODO: Implement a scheduler removal of old messages (configurable, must not hurt performance)
    // TODO: Add a Dashmap if performance is a problem (Personally don't think it will be)
}

#[derive(Debug, Clone)]
pub struct TimedMessage {
    pub message: MavMessage,
    pub time: Instant,
}

impl TimedMessage {
    fn just_received(message: MavMessage) -> Self {
        Self {
            message,
            time: Instant::now(),
        }
    }
}

/// Helper function to read a stream of bytes and return an iterator of MavLink messages
fn iter_messages(buf: &[u8]) -> impl Iterator<Item = (MavHeader, MavMessage)> + '_ {
    let mut reader = PeekReader::new(buf);
    std::iter::from_fn(move || read_v1_msg(&mut reader).ok())
}

pub struct ReflectionContext {
    mavlink_profile: MavProfile,
    id_name_map: HashMap<u32, String>,
}

impl ReflectionContext {
    pub fn new() -> Self {
        let profile: MavProfile =
            serde_json::from_str(skyward_mavlink::reflection::LYRA_MAVLINK_PROFILE_SERIALIZED)
                .expect("Failed to deserialize MavProfile");
        let id_name_map = profile
            .messages
            .iter()
            .map(|(name, m)| (m.id, name.clone()))
            .collect();
        Self {
            mavlink_profile: profile,
            id_name_map,
        }
    }

    pub fn get_name_from_id(&self, message_id: u32) -> Option<&str> {
        self.id_name_map.get(&message_id).map(|s| s.as_str())
    }

    pub fn messages(&self) -> Vec<&str> {
        self.mavlink_profile
            .messages
            .keys()
            .map(|s| s.as_str())
            .collect()
    }

    pub fn get_fields_by_id(&self, message_id: u32) -> Vec<&str> {
        self.mavlink_profile
            .messages
            .iter()
            .find(|(_, m)| m.id == message_id)
            .map(|(_, m)| &m.fields)
            .unwrap_or_else(|| {
                panic!("Message ID {} not found in profile", message_id);
            })
            .iter()
            .map(|f| f.name.as_str())
            .collect()
    }

    pub fn get_plottable_fields_by_id(&self, message_id: u32) -> Vec<&str> {
        self.mavlink_profile
            .messages
            .iter()
            .find(|(_, m)| m.id == message_id)
            .map(|(_, m)| &m.fields)
            .unwrap_or_else(|| {
                panic!("Message ID {} not found in profile", message_id);
            })
            .iter()
            .filter(|f| {
                matches!(
                    f.mavtype,
                    MavType::UInt8
                        | MavType::UInt16
                        | MavType::UInt32
                        | MavType::UInt64
                        | MavType::Int8
                        | MavType::Int16
                        | MavType::Int32
                        | MavType::Int64
                        | MavType::Float
                        | MavType::Double
                )
            })
            .map(|f| f.name.as_str())
            .collect()
    }

    pub fn get_fields_by_name(&self, message_name: &str) -> Vec<&str> {
        self.mavlink_profile
            .messages
            .iter()
            .find(|(_, m)| m.name == message_name)
            .map(|(_, m)| &m.fields)
            .unwrap_or_else(|| {
                panic!("Message {} not found in profile", message_name);
            })
            .iter()
            .map(|f| f.name.as_str())
            .collect()
    }
}
