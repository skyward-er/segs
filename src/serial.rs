//! Serial port utilities
//!
//! This module provides utilities for working with serial ports, such as listing all available serial ports and finding the first serial port that contains "STM32" or "ST-LINK" in its product name.

use anyhow::Context;

use crate::error::ErrInstrument;

/// Get the first serial port that contains "STM32" or "ST-LINK" in its product name
pub fn get_first_stm32_serial_port() -> Option<String> {
    let ports = serialport::available_ports().log_expect("Serial ports cannot be listed!");
    for port in ports {
        if let serialport::SerialPortType::UsbPort(info) = port.port_type {
            if let Some(p) = info.product {
                if p.contains("STM32") || p.contains("ST-LINK") {
                    return Some(port.port_name);
                }
            }
        }
    }
    None
}

/// Get a list of all serial ports available on the system
pub fn list_all_serial_ports() -> anyhow::Result<Vec<String>> {
    let ports = serialport::available_ports().context("No serial ports found!")?;
    Ok(ports.iter().map(|p| p.port_name.clone()).collect())
}
