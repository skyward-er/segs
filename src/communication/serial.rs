use std::{io::Read, sync::Mutex, time::Duration};

use serialport::{SerialPortInfo, SerialPortType};
use tracing::debug;

use crate::mavlink::TimedMessage;

use super::{
    ConnectionError,
    sealed::{Connectable, MessageTransceiver},
};

pub const DEFAULT_BAUD_RATE: u32 = 115200;

pub fn list_all_usb_ports() -> Result<Vec<SerialPortInfo>, serialport::Error> {
    let ports = serialport::available_ports()?;
    Ok(ports
        .into_iter()
        .filter(|p| matches!(p.port_type, SerialPortType::UsbPort(_)))
        .collect())
}

pub fn find_first_stm32_port() -> Result<Option<SerialPortInfo>, serialport::Error> {
    for port in list_all_usb_ports()? {
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

pub mod cached {
    use egui::Context;

    use crate::ui::cache::RecentCallCache;

    use super::*;

    pub fn cached_list_all_usb_ports(
        ctx: &Context,
    ) -> Result<Vec<SerialPortInfo>, serialport::Error> {
        ctx.call_cached_short(&"list_usb_ports", list_all_usb_ports)
    }

    pub fn cached_first_stm32_port(
        ctx: &Context,
    ) -> Result<Option<SerialPortInfo>, serialport::Error> {
        ctx.call_cached_short(&"find_first_stm32_port", find_first_stm32_port)
    }
}

#[derive(Debug, Clone)]
pub struct SerialConfiguration {
    pub port_name: String,
    pub baud_rate: u32,
}

impl Connectable for SerialConfiguration {
    type Connected = SerialTransceiver;

    #[profiling::function]
    fn connect(&self) -> Result<Self::Connected, ConnectionError> {
        let port = serialport::new(&self.port_name, self.baud_rate)
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| ConnectionError::WrongConfiguration(e.to_string()))?;
        debug!(
            "Connected to serial port {} at {} baud",
            self.port_name, self.baud_rate
        );
        Ok(SerialTransceiver { port: Mutex::new(port) })
    }
}

pub struct SerialTransceiver {
    port: Mutex<Box<dyn serialport::SerialPort>>,
}

impl MessageTransceiver for SerialTransceiver {
    #[profiling::function]
    fn wait_for_message(&self) -> Result<TimedMessage, std::io::Error> {
        let mut port = self.port.lock().unwrap();
        let mut header_buf = [0u8; 6];
        port.read_exact(&mut header_buf)?;

        let header = crate::ccsds::CcsdsHeader::decode(&header_buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        let payload_len = header.data_field_len();

        let mut payload = vec![0u8; payload_len];
        port.read_exact(&mut payload)?;

        let mut full = header_buf.to_vec();
        full.extend_from_slice(&payload);

        debug!("Received {} bytes via serial", full.len());
        TimedMessage::from_ccsds_bytes(&full)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    #[profiling::function]
    fn transmit_message(&self, data: &[u8]) -> Result<(), std::io::Error> {
        use std::io::Write;
        let mut port = self.port.lock().unwrap();
        port.write_all(data)?;
        debug!("Sent {} bytes via serial", data.len());
        Ok(())
    }
}
