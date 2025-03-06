//! Serial port utilities module.
//!
//! Provides functions for listing USB serial ports, finding a STM32 port,
//! and handling serial connections including message transmission and reception.

use std::sync::Mutex;

use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use skyward_mavlink::mavlink::{
    MavFrame,
    error::{MessageReadError, MessageWriteError},
    read_v1_msg, write_v1_msg,
};
use tracing::{debug, trace};

use crate::{
    error::ErrInstrument,
    mavlink::{MavMessage, TimedMessage, peek_reader::PeekReader},
};

use super::{
    ConnectionError,
    sealed::{Connectable, MessageTransceiver},
};

const SERIAL_PORT_TIMEOUT_MS: u64 = 100;
pub const DEFAULT_BAUD_RATE: u32 = 115200;

/// Returns a list of all USB serial ports available on the system.
///
/// # Returns
/// * `Ok(Vec<SerialPortInfo>)` if ports are found or an error otherwise.
#[profiling::function]
pub fn list_all_usb_ports() -> Result<Vec<SerialPortInfo>, serialport::Error> {
    let ports = serialport::available_ports()?;
    Ok(ports
        .into_iter()
        .filter(|p| matches!(p.port_type, SerialPortType::UsbPort(_)))
        .collect())
}

/// Finds the first USB serial port whose product name contains "STM32" or "ST-LINK".
///
/// # Returns
/// * `Ok(Some(SerialPortInfo))` if a matching port is found, `Ok(None)` otherwise.
#[profiling::function]
pub fn find_first_stm32_port() -> Result<Option<SerialPortInfo>, serialport::Error> {
    let ports = list_all_usb_ports()?;
    for port in ports {
        if let serialport::SerialPortType::UsbPort(info) = &port.port_type {
            if let Some(p) = &info.product {
                if p.contains("STM32") || p.contains("ST-LINK") {
                    return Ok(Some(port));
                }
            }
        }
    }
    Ok(None)
}

/// Configuration for a serial connection.
#[derive(Debug, Clone)]
pub struct SerialConfiguration {
    pub port_name: String,
    pub baud_rate: u32,
}

impl Connectable for SerialConfiguration {
    type Connected = SerialTransceiver;

    /// Connects using the serial port configuration.
    #[profiling::function]
    fn connect(&self) -> Result<Self::Connected, ConnectionError> {
        let port = serialport::new(&self.port_name, self.baud_rate)
            .timeout(std::time::Duration::from_millis(SERIAL_PORT_TIMEOUT_MS))
            .open()?;
        debug!(
            "Connected to serial port {} with baud rate {}",
            self.port_name, self.baud_rate
        );
        Ok(SerialTransceiver {
            serial_reader: Mutex::new(Box::new(PeekReader::new(port.try_clone()?))),
            serial_writer: Mutex::new(port),
        })
    }
}

impl From<serialport::Error> for ConnectionError {
    fn from(e: serialport::Error) -> Self {
        let serialport::Error { kind, description } = e.clone();
        match kind {
            serialport::ErrorKind::NoDevice => ConnectionError::WrongConfiguration(description),
            serialport::ErrorKind::InvalidInput => ConnectionError::WrongConfiguration(description),
            serialport::ErrorKind::Unknown => ConnectionError::Unknown(description),
            serialport::ErrorKind::Io(e) => ConnectionError::Io(e.into()),
        }
    }
}

/// Manages a connection to a serial port.
pub struct SerialTransceiver {
    serial_reader: Mutex<Box<PeekReader<Box<dyn SerialPort>>>>,
    #[allow(dead_code)]
    serial_writer: Mutex<Box<dyn SerialPort>>,
}

impl MessageTransceiver for SerialTransceiver {
    /// Blocks until a valid message is received from the serial port.
    #[profiling::function]
    fn wait_for_message(&self) -> Result<TimedMessage, MessageReadError> {
        loop {
            let res: Result<(_, MavMessage), MessageReadError> =
                read_v1_msg(&mut self.serial_reader.lock().log_unwrap());
            match res {
                Ok((_, msg)) => {
                    return Ok(TimedMessage::just_received(msg));
                }
                Err(MessageReadError::Io(e)) if e.kind() == std::io::ErrorKind::TimedOut => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    /// Transmits a message via the serial connection.
    #[profiling::function]
    fn transmit_message(&self, msg: MavFrame<MavMessage>) -> Result<usize, MessageWriteError> {
        let MavFrame { header, msg, .. } = msg;
        let written = write_v1_msg(&mut *self.serial_writer.lock().log_unwrap(), header, &msg)?;
        debug!("Sent message: {:?}", msg);
        trace!("Sent {} bytes via serial", written);
        Ok(written)
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
