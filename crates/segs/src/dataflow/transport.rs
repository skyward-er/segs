use std::net::SocketAddrV4;

use argh::FromArgValue;

#[derive(FromArgValue)]
pub enum TransportType {
    Ethernet,
    Serial,
}

/// Enum representing the different types of data transport mechanisms that can be used to receive raw data.
#[derive(Debug)]
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
