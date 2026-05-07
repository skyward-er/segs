use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    str::FromStr,
    time::Duration,
};

use tracing::debug;

use crate::mavlink::TimedMessage;

use super::{
    ConnectionError,
    sealed::{Connectable, MessageTransceiver},
};

pub const DEFAULT_ETHERNET_BROADCAST_IP: IpAddr =
    IpAddr::V4(Ipv4Addr::from_octets([10, 20, 72, 187]));
pub const DEFAULT_RCV_ETHERNET_PORT: u16 = 8081;
pub const DEFAULT_SEND_ETHERNET_PORT: u16 = 21002;

#[derive(Debug, Clone)]
pub struct EthernetConfiguration {
    pub ip_address: IpAddr,
    pub send_port: u16,
    pub receive_port: u16,
}

impl Connectable for EthernetConfiguration {
    type Connected = EthernetTransceiver;

    #[profiling::function]
    fn connect(&self) -> Result<Self::Connected, ConnectionError> {
        let recv_socket = UdpSocket::bind(format!("0.0.0.0:{}", self.receive_port))?;
        recv_socket.set_read_timeout(Some(Duration::from_millis(100)))?;

        let send_socket = UdpSocket::bind("0.0.0.0:0")?;
        send_socket.set_broadcast(true)?;
        send_socket.set_write_timeout(Some(Duration::from_millis(100)))?;

        let send_addr = SocketAddr::new(self.ip_address, self.send_port);

        debug!("Receiving Ethernet set up on port {}", self.receive_port);
        debug!(
            "Sending Ethernet set up on {}:{}",
            self.ip_address, self.send_port
        );

        Ok(EthernetTransceiver {
            recv_socket,
            send_socket,
            send_addr,
        })
    }
}

pub struct EthernetTransceiver {
    recv_socket: UdpSocket,
    send_socket: UdpSocket,
    send_addr: SocketAddr,
}

impl MessageTransceiver for EthernetTransceiver {
    #[profiling::function]
    fn wait_for_message(&self) -> Result<TimedMessage, std::io::Error> {
        let mut buf = [0u8; 65535];
        let n = self.recv_socket.recv(&mut buf)?;
        debug!("Received {} bytes via Ethernet", n);
        TimedMessage::from_ccsds_bytes(&buf[..n])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    #[profiling::function]
    fn transmit_message(&self, data: &[u8]) -> Result<(), std::io::Error> {
        self.send_socket.send_to(data, self.send_addr)?;
        debug!("Sent {} bytes via Ethernet", data.len());
        Ok(())
    }
}
