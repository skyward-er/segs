use std::net::SocketAddrV4;

use argh::FromArgValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromArgValue)]
pub enum TransportType {
    #[default]
    Ethernet,
    Serial,
}

/// Enum representing the different types of data transport mechanisms that can be used to receive raw data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataTransport {
    Ethernet {
        recv_socket: SocketAddrV4,
        send_socket: SocketAddrV4,
    },
    Serial {
        tty: String,
        baud_rate: u32,
    },
}
