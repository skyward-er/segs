use std::net::UdpSocket;

use skyward_mavlink::mavlink::{
    MavFrame,
    error::{MessageReadError, MessageWriteError},
    read_v1_msg, write_v1_msg,
};
use tracing::{debug, trace};

use crate::mavlink::{MAX_MSG_SIZE, MavMessage, TimedMessage, peek_reader::PeekReader};

use super::{Connectable, ConnectionError, MessageTransceiver};

#[derive(Debug, Clone)]
pub struct EthernetConfiguration {
    pub port: u16,
}

impl Connectable for EthernetConfiguration {
    type Connected = EthernetTransceiver;

    #[profiling::function]
    fn connect(&self) -> Result<Self::Connected, ConnectionError> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", self.port))?;
        debug!("Connected to Ethernet port on port {}", self.port);
        Ok(EthernetTransceiver { socket })
    }
}

/// Manages a connection to a Ethernet port.
pub struct EthernetTransceiver {
    socket: UdpSocket,
}

impl MessageTransceiver for EthernetTransceiver {
    #[profiling::function]
    fn wait_for_message(&self) -> Result<TimedMessage, MessageReadError> {
        let mut buf = [0; MAX_MSG_SIZE];
        let read = self.socket.recv(&mut buf)?;
        trace!("Received {} bytes", read);
        let mut reader = PeekReader::new(&buf[..read]);
        let (_, res) = read_v1_msg(&mut reader)?;
        debug!("Received message: {:?}", res);
        Ok(TimedMessage::just_received(res))
    }

    #[profiling::function]
    fn transmit_message(&self, msg: MavFrame<MavMessage>) -> Result<usize, MessageWriteError> {
        let MavFrame { header, msg, .. } = msg;
        let mut write_buf = Vec::new();
        write_v1_msg(&mut write_buf, header, &msg)?;
        let written = self.socket.send(&write_buf)?;
        debug!("Sent message: {:?}", msg);
        trace!("Sent {} bytes via Ethernet", written);
        Ok(written)
    }
}
