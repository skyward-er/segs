//! Serial port utilities module.
//!
//! Provides functions for listing USB serial ports, finding a STM32 port,
//! and handling serial connections including message transmission and reception.

use serialport::{SerialPortInfo, SerialPortType};
use skyward_mavlink::mavlink::{
    self,
    error::{MessageReadError, MessageWriteError},
};
use tracing::{debug, trace};

use crate::mavlink::{MavFrame, MavMessage, MavlinkVersion, TimedMessage};

use super::{
    BoxedConnection, ConnectionError,
    sealed::{Connectable, MessageTransceiver},
};

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

pub mod cached {
    use egui::Context;

    use crate::ui::cache::RecentCallCache;

    use super::*;

    /// Returns a cached list of all available USB ports.
    ///
    /// # Arguments
    /// * `ctx` - The egui context used for caching.
    ///
    /// # Returns
    /// * A Result containing a vector of `SerialPortInfo` or a `serialport::Error`.
    pub fn cached_list_all_usb_ports(
        ctx: &Context,
    ) -> Result<Vec<SerialPortInfo>, serialport::Error> {
        ctx.call_cached_short(&"list_usb_ports", list_all_usb_ports)
    }

    /// Returns the first cached STM32 port found, if any.
    ///
    /// # Arguments
    /// * `ctx` - The egui context used for caching.
    ///
    /// # Returns
    /// * A Result containing an Option of `SerialPortInfo` or a `serialport::Error`.
    pub fn cached_first_stm32_port(
        ctx: &Context,
    ) -> Result<Option<SerialPortInfo>, serialport::Error> {
        ctx.call_cached_short(&"find_first_stm32_port", find_first_stm32_port)
    }
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
        let serial_edpoint = format!("serial:{}:{}", self.port_name, self.baud_rate);
        let mut mav_connection: BoxedConnection = mavlink::connect(&serial_edpoint)?;
        mav_connection.set_protocol_version(MavlinkVersion::V1);
        debug!(
            "Connected to serial port {} with baud rate {}",
            self.port_name, self.baud_rate
        );
        Ok(SerialTransceiver { mav_connection })
    }
}

/// Manages a connection to a serial port.
pub struct SerialTransceiver {
    mav_connection: BoxedConnection,
}

impl MessageTransceiver for SerialTransceiver {
    /// Blocks until a valid message is received from the serial port.
    #[profiling::function]
    fn wait_for_message(&self) -> Result<TimedMessage, MessageReadError> {
        let (_, msg) = self.mav_connection.recv()?;
        debug!("Received message: {:?}", &msg);
        Ok(TimedMessage::just_received(msg))
    }

    /// Transmits a message via the serial connection.
    #[profiling::function]
    fn transmit_message(&self, msg: MavFrame<MavMessage>) -> Result<usize, MessageWriteError> {
        let written = self.mav_connection.send_frame(&msg)?;
        debug!("Sent message: {:?}", msg);
        trace!("Sent {} bytes via serial", written);
        Ok(written)
    }
}
