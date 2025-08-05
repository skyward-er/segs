//! Ethernet utilities module.
//!
//! Provides functionality to connect via Ethernet using UDP, allowing message
//! transmission and reception over a network.

use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

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

pub const DEFAULT_ETHERNET_BROADCAST_IP: IpAddr = IpAddr::V4(Ipv4Addr::from_bits(0x00000000));

/// Configuration for an Ethernet connection.
#[derive(Debug, Clone)]
pub struct EthernetConfiguration {
    pub ip_address: IpAddr,
    pub send_port: u16,
    pub receive_port: u16,
}

impl Connectable for EthernetConfiguration {
    type Connected = EthernetTransceiver;

    /// Binds to the specified UDP port to create a network connection.
    #[profiling::function]
    fn connect(&self) -> Result<Self::Connected, ConnectionError> {
        let incoming_addr = format!("udpin:0.0.0.0:{}", self.receive_port);
        let outgoing_addr = format!("udpcast:{}:{}", self.ip_address, self.send_port);
        let mut incoming_conn: BoxedConnection = mavlink::connect(&incoming_addr)?;
        let mut outgoing_conn: BoxedConnection = mavlink::connect(&outgoing_addr)?;
        incoming_conn.set_protocol_version(MavlinkVersion::V1);
        incoming_conn.set_read_timeout(Some(Duration::from_millis(100)))?;
        outgoing_conn.set_protocol_version(MavlinkVersion::V1);
        outgoing_conn.set_write_timeout(Some(Duration::from_millis(100)))?;
        debug!("Receiving Ethernet set up on port {}", self.receive_port);
        debug!("Sending Ethernet set up on port {}", self.send_port);
        Ok(EthernetTransceiver {
            incoming_conn,
            outgoing_conn,
        })
    }
}

/// Manages a connection over Ethernet.
pub struct EthernetTransceiver {
    incoming_conn: BoxedConnection,
    outgoing_conn: BoxedConnection,
}

impl MessageTransceiver for EthernetTransceiver {
    /// Waits for a message over Ethernet, blocking until a valid message arrives.
    #[profiling::function]
    fn wait_for_message(&self) -> Result<TimedMessage, MessageReadError> {
        let (_, msg) = self.incoming_conn.recv()?;
        debug!("Received message: {:?}", &msg);
        Ok(TimedMessage::just_received(msg))
    }

    /// Transmits a message using the UDP socket.
    #[profiling::function]
    fn transmit_message(&self, msg: MavFrame<MavMessage>) -> Result<usize, MessageWriteError> {
        let written = self.outgoing_conn.send_frame(&msg)?;
        debug!("Sent message: {:?}", msg);
        trace!("Sent {} bytes via Ethernet", written);
        Ok(written)
    }
}
