//! Ethernet utilities module.
//!
//! Provides functionality to connect via Ethernet using UDP, allowing message
//! transmission and reception over a network.

use std::net::UdpSocket;

use skyward_mavlink::mavlink::{
    MavFrame,
    error::{MessageReadError, MessageWriteError},
    read_v1_msg, write_v1_msg,
};
use tracing::{debug, trace};

use crate::mavlink::{MAX_MSG_SIZE, MavMessage, TimedMessage, peek_reader::PeekReader};

use super::{
    ConnectionError,
    sealed::{Connectable, MessageTransceiver},
};

/// Configuration for an Ethernet connection.
#[derive(Debug, Clone)]
pub struct EthernetConfiguration {
    pub port: u16,
}

impl Connectable for EthernetConfiguration {
    type Connected = EthernetTransceiver;

    /// Binds to the specified UDP port to create a network connection.
    #[profiling::function]
    fn connect(&self) -> Result<Self::Connected, ConnectionError> {
        let recv_addr = format!("0.0.0.0:{}", self.port);
        let server_socket = UdpSocket::bind(recv_addr)?;
        debug!("Bound to Ethernet port on port {}", self.port);
        let send_addr = "0.0.0.0:0";
        let cast_addr = format!("255.255.255.255:{}", self.port);
        let client_socket = UdpSocket::bind(send_addr)?;
        client_socket.set_broadcast(true)?;
        client_socket.connect(&cast_addr)?;
        debug!("Created Ethernet connection to {}", cast_addr);
        Ok(EthernetTransceiver {
            server_socket,
            client_socket,
        })
    }
}

/// Manages a connection over Ethernet.
pub struct EthernetTransceiver {
    server_socket: UdpSocket,
    client_socket: UdpSocket,
}

impl MessageTransceiver for EthernetTransceiver {
    /// Waits for a message over Ethernet, blocking until a valid message arrives.
    #[profiling::function]
    fn wait_for_message(&self) -> Result<TimedMessage, MessageReadError> {
        let mut buf = [0; MAX_MSG_SIZE];
        let read = self.server_socket.recv(&mut buf)?;
        trace!("Received {} bytes", read);
        let mut reader = PeekReader::new(&buf[..read]);
        let (_, res) = read_v1_msg(&mut reader)?;
        debug!("Received message: {:?}", res);
        Ok(TimedMessage::just_received(res))
    }

    /// Transmits a message using the UDP socket.
    #[profiling::function]
    fn transmit_message(&self, msg: MavFrame<MavMessage>) -> Result<usize, MessageWriteError> {
        let MavFrame { header, msg, .. } = msg;
        let mut write_buf = Vec::new();
        write_v1_msg(&mut write_buf, header, &msg)?;
        let written = self.client_socket.send(&write_buf)?;
        debug!("Sent message: {:?}", msg);
        trace!("Sent {} bytes via Ethernet", written);
        Ok(written)
    }
}
