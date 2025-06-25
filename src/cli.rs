use clap::{
    Error, Parser,
    builder::TypedValueParser,
    error::{ContextKind, ContextValue, ErrorKind},
};

use crate::{
    communication::{EthernetConfiguration, SerialConfiguration},
    message_broker::ConnectionConfig,
    ui::AppConfig,
};

/// Command-line interface for the application
#[derive(Debug, Clone, Parser)]
pub struct Cli {
    /// Use Ethernet interface for communication.
    ///
    /// Provide the address in the format `IP:RX_PORT:TX_PORT`
    /// e.g. `--ethernet 169.254.0.12:14550:14550`
    #[arg(long, value_parser = EthernetValueParser)]
    ethernet: Option<EthernetConfiguration>,

    /// Use Serial interface for communication.
    ///
    /// Provide the serial port name, e.g. `--serial /dev/TTY_PORT:BAUD_RATE`
    /// e.g. `--serial /dev/ttyUSB0:115200`
    #[arg(long, value_parser = SerialValueParser)]
    serial: Option<SerialConfiguration>,
}

#[derive(Debug, Clone)]
struct EthernetValueParser;

impl clap::builder::TypedValueParser for EthernetValueParser {
    type Value = EthernetConfiguration;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, Error> {
        let mut err = Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
        if let Some(arg) = arg {
            err.insert(
                ContextKind::InvalidArg,
                ContextValue::String(arg.to_string()),
            );
        }
        let Some(value_str) = value.to_str() else {
            err.insert(
                ContextKind::Custom,
                ContextValue::String("Invalid UTF-8 in value".to_string()),
            );
            return Err(err);
        };

        let parts: Vec<&str> = value_str.split(':').collect();
        if parts.len() != 3 {
            err.insert(
                ContextKind::InvalidValue,
                ContextValue::String(value_str.to_string()),
            );
            return Err(err);
        }

        let Ok(ip_address) = parts[0].parse::<std::net::IpAddr>() else {
            err.insert(
                ContextKind::SuggestedValue,
                ContextValue::String("255.255.255.255".to_string()),
            );
            err.insert(
                ContextKind::InvalidValue,
                ContextValue::String(parts[0].to_string()),
            );
            return Err(err);
        };
        let Ok(receive_port) = parts[1].parse::<u16>() else {
            err.insert(
                ContextKind::SuggestedValue,
                ContextValue::String("42069".to_string()),
            );
            err.insert(
                ContextKind::InvalidValue,
                ContextValue::String(parts[1].to_string()),
            );
            return Err(err);
        };
        let Ok(send_port) = parts[2].parse::<u16>() else {
            err.insert(
                ContextKind::SuggestedValue,
                ContextValue::String("42069".to_string()),
            );
            err.insert(
                ContextKind::InvalidValue,
                ContextValue::String(parts[2].to_string()),
            );
            return Err(err);
        };

        Ok(EthernetConfiguration {
            ip_address,
            send_port,
            receive_port,
        })
    }
}

#[derive(Debug, Clone)]
struct SerialValueParser;

impl TypedValueParser for SerialValueParser {
    type Value = SerialConfiguration;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, Error> {
        let mut err = Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
        if let Some(arg) = arg {
            err.insert(
                ContextKind::InvalidArg,
                ContextValue::String(arg.to_string()),
            );
        }
        let Some(value_str) = value.to_str() else {
            err.insert(
                ContextKind::Custom,
                ContextValue::String("Invalid UTF-8 in value".to_string()),
            );
            return Err(err);
        };
        let parts: Vec<&str> = value_str.split(':').collect();
        if parts.len() != 2 {
            err.insert(
                ContextKind::SuggestedValue,
                ContextValue::String("PORT_NAME:BAUD_RATE".to_string()),
            );
            err.insert(
                ContextKind::InvalidValue,
                ContextValue::String(value_str.to_string()),
            );
            return Err(err);
        }
        let port_name = parts[0].to_string();
        let baud_rate = parts[1].parse::<u32>().map_err(|_| {
            err.insert(
                ContextKind::InvalidValue,
                ContextValue::String(parts[1].to_string()),
            );
            err
        })?;

        Ok(SerialConfiguration {
            port_name,
            baud_rate,
        })
    }
}

impl From<Cli> for AppConfig {
    fn from(value: Cli) -> Self {
        let connection_config = if let Some(eth) = value.ethernet {
            Some(ConnectionConfig::Ethernet(eth))
        } else {
            value.serial.map(ConnectionConfig::Serial)
        };
        AppConfig { connection_config }
    }
}
