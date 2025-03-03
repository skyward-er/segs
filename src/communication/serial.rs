//! Serial port utilities
//!
//! This module provides utilities for working with serial ports, such as
//! listing all available serial ports and finding the first serial port that
//! contains "STM32" or "ST-LINK" in its product name.

use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};

use anyhow::Context;
use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use skyward_mavlink::mavlink::error::MessageReadError;
use tracing::error;

use crate::{
    error::ErrInstrument,
    mavlink::{
        MavHeader, MavMessage, MavlinkVersion, TimedMessage, peek_reader::PeekReader,
        read_versioned_msg,
    },
};

const MAX_STORED_MSGS: usize = 100; // 192 bytes each = 19.2 KB
const SERIAL_PORT_TIMEOUT_MS: u64 = 100;

/// Represents a candidate serial port device.
#[derive(Debug, Clone)]
pub struct SerialPortCandidate {
    port_name: String,
    info: SerialPortInfo,
}

impl PartialEq for SerialPortCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.port_name == other.port_name
    }
}

impl SerialPortCandidate {
    /// Connects to the serial port with the given baud rate.
    pub fn connect(self, baud_rate: u32) -> Result<SerialConnection, serialport::Error> {
        let serial_port = serialport::new(&self.port_name, baud_rate)
            .timeout(std::time::Duration::from_millis(SERIAL_PORT_TIMEOUT_MS))
            .open()?;
        Ok(SerialConnection {
            serial_port_reader: Arc::new(Mutex::new(PeekReader::new(serial_port))),
            stored_msgs: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_STORED_MSGS))),
            running_flag: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        })
    }

    /// Get a list of all serial USB ports available on the system
    pub fn list_all_usb_ports() -> anyhow::Result<Vec<Self>> {
        let ports = serialport::available_ports().context("No serial ports found!")?;
        Ok(ports
            .into_iter()
            .filter(|p| matches!(p.port_type, SerialPortType::UsbPort(_)))
            .map(|p| SerialPortCandidate {
                port_name: p.port_name.clone(),
                info: p,
            })
            .collect())
    }

    /// Finds the first USB serial port with "STM32" or "ST-LINK" in its product name.
    /// Renamed from get_first_stm32_serial_port.
    pub fn find_first_stm32_port() -> Option<Self> {
        let ports = Self::list_all_usb_ports().log_unwrap();
        for port in ports {
            if let serialport::SerialPortType::UsbPort(info) = &port.info.port_type {
                if let Some(p) = &info.product {
                    if p.contains("STM32") || p.contains("ST-LINK") {
                        return Some(port);
                    }
                }
            }
        }
        None
    }
}

impl AsRef<String> for SerialPortCandidate {
    fn as_ref(&self) -> &String {
        &self.port_name
    }
}

/// Manages a connection to a serial port.
pub struct SerialConnection {
    serial_port_reader: Arc<Mutex<PeekReader<Box<dyn SerialPort>>>>,
    stored_msgs: Arc<Mutex<VecDeque<TimedMessage>>>,
    running_flag: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl SerialConnection {
    /// Starts receiving messages asynchronously.
    pub fn start_receiving(&mut self) {
        let running_flag = self.running_flag.clone();
        let serial_port = self.serial_port_reader.clone();
        let stored_msgs = self.stored_msgs.clone();
        let thread_handle = std::thread::spawn(move || {
            while running_flag.load(Ordering::Relaxed) {
                let res: Result<(MavHeader, MavMessage), MessageReadError> =
                    read_versioned_msg(&mut serial_port.lock().log_unwrap(), MavlinkVersion::V1);
                match res {
                    Ok((_, msg)) => {
                        // Store the message in the buffer.
                        stored_msgs
                            .lock()
                            .log_unwrap()
                            .push_back(TimedMessage::just_received(msg));
                    }
                    Err(MessageReadError::Io(e)) => {
                        // Ignore timeouts.
                        if e.kind() == std::io::ErrorKind::TimedOut {
                            continue;
                        } else {
                            error!("Error reading message: {:?}", e);
                            running_flag.store(false, Ordering::Relaxed);
                        }
                    }
                    Err(e) => {
                        error!("Error reading message: {:?}", e);
                    }
                };
            }
        });
        self.thread_handle.replace(thread_handle);
    }

    /// Stops receiving messages.
    pub fn stop_receiving(&mut self) {
        self.running_flag.store(false, Ordering::Relaxed);
        if let Some(handle) = self.thread_handle.take() {
            handle.join().log_unwrap();
        }
    }

    /// Retrieves and clears the stored messages.
    pub fn retrieve_messages(&self) -> Vec<TimedMessage> {
        self.stored_msgs.lock().log_unwrap().drain(..).collect()
    }

    /// Transmits a message over the serial connection.
    pub fn transmit_message(&mut self, msg: &[u8]) {
        todo!()
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, io::Read};

    use rand::prelude::*;
    use skyward_mavlink::{mavlink::*, orion::*};

    use super::*;

    struct ChunkedMessageStreamGenerator {
        rng: SmallRng,
        buffer: VecDeque<u8>,
    }

    impl ChunkedMessageStreamGenerator {
        const KINDS: [u32; 2] = [ACK_TM_DATA::ID, NACK_TM_DATA::ID];

        fn new() -> Self {
            Self {
                rng: SmallRng::seed_from_u64(42),
                buffer: VecDeque::new(),
            }
        }

        fn msg_push(&mut self, msg: &MavMessage, header: MavHeader) -> std::io::Result<()> {
            write_v1_msg(&mut self.buffer, header, msg).unwrap();
            Ok(())
        }

        fn fill_buffer(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            while buf.len() > self.buffer.len() {
                self.add_next_rand();
            }
            let n = buf.len();
            buf.iter_mut()
                .zip(self.buffer.drain(..n))
                .for_each(|(a, b)| *a = b);
            Ok(n)
        }

        fn add_next_rand(&mut self) {
            let i = self.rng.random_range(0..Self::KINDS.len());
            let id = Self::KINDS[i];
            let msg = MavMessage::default_message_from_id(id).unwrap();
            let header = MavHeader {
                system_id: 1,
                component_id: 1,
                sequence: 0,
            };
            self.msg_push(&msg, header).unwrap();
        }
    }

    impl Read for ChunkedMessageStreamGenerator {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            // fill buffer with sequence of byte of random length
            if buf.len() == 1 {
                self.fill_buffer(&mut buf[..1])
            } else if !buf.is_empty() {
                let size = self.rng.random_range(1..buf.len());
                self.fill_buffer(&mut buf[..size])
            } else {
                Ok(0)
            }
        }
    }

    #[test]
    fn test_peek_reader_with_chunked_transmission() {
        let mut gms = ChunkedMessageStreamGenerator::new();
        let mut reader = PeekReader::new(&mut gms);
        let mut msgs = Vec::new();
        for _ in 0..100 {
            let (_, msg): (MavHeader, MavMessage) = read_v1_msg(&mut reader).unwrap();
            msgs.push(msg);
        }
        for msg in msgs {
            assert!(msg.message_id() == ACK_TM_DATA::ID || msg.message_id() == NACK_TM_DATA::ID);
        }
    }
}
