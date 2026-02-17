pub mod direct_serial;
pub mod udp;

use std::{io, net::SocketAddrV4, sync::Arc};

use mavlink_core::{
    MavlinkVersion,
    error::{MessageReadError, MessageWriteError},
};

use self::{direct_serial::SerialConnection, udp::UdpConnection};
use crate::{MavFrame, MavProfile};

/// A MAVLink connection
pub trait MavConnection {
    /// Read whole frame
    fn recv_frame(&self) -> Result<MavFrame, MessageReadError>;

    /// Send a whole frame
    fn send_frame(&self, frame: MavFrame) -> Result<usize, MessageWriteError>;

    /// Sets the MAVLink version to use for receiving (when
    /// `allow_recv_any_version()` is `false`) and sending messages.
    fn set_protocol_version(&mut self, version: MavlinkVersion);
    /// Gets the currently used MAVLink version
    fn protocol_version(&self) -> MavlinkVersion;

    /// Set wether MAVLink messages of either version may be received.
    ///
    /// If set to false only messages of the version configured with
    /// `set_protocol_version()` are received.
    fn set_allow_recv_any_version(&mut self, allow: bool);
    /// Wether messages of any MAVLink version may be received
    fn allow_recv_any_version(&self) -> bool;
}

/// Concrete MAVLink connection returned by [`connect`].
pub struct Connection(ConnectionInner);

enum ConnectionInner {
    Udp(UdpConnection),
    Serial(SerialConnection),
}

impl Connection {
    pub fn udp(
        listen_addr: impl Into<SocketAddrV4>,
        send_addr: impl Into<SocketAddrV4>,
        profile: Arc<MavProfile>,
    ) -> io::Result<Self> {
        Ok(Self(ConnectionInner::Udp(UdpConnection::new(
            listen_addr.into(),
            send_addr.into(),
            profile,
        )?)))
    }

    pub fn serial(address: String, baud_rate: u32, profile: Arc<MavProfile>) -> io::Result<Self> {
        Ok(Self(ConnectionInner::Serial(SerialConnection::new(
            address, baud_rate, profile,
        )?)))
    }
}

impl MavConnection for Connection {
    fn recv_frame(&self) -> Result<MavFrame, MessageReadError> {
        match &self.0 {
            ConnectionInner::Udp(conn) => conn.recv_frame(),
            ConnectionInner::Serial(conn) => conn.recv_frame(),
        }
    }

    fn send_frame(&self, frame: MavFrame) -> Result<usize, MessageWriteError> {
        match &self.0 {
            ConnectionInner::Udp(conn) => conn.send_frame(frame),
            ConnectionInner::Serial(conn) => conn.send_frame(frame),
        }
    }

    fn set_protocol_version(&mut self, version: MavlinkVersion) {
        match &mut self.0 {
            ConnectionInner::Udp(conn) => conn.set_protocol_version(version),
            ConnectionInner::Serial(conn) => conn.set_protocol_version(version),
        }
    }

    fn protocol_version(&self) -> MavlinkVersion {
        match &self.0 {
            ConnectionInner::Udp(conn) => conn.protocol_version(),
            ConnectionInner::Serial(conn) => conn.protocol_version(),
        }
    }

    fn set_allow_recv_any_version(&mut self, allow: bool) {
        match &mut self.0 {
            ConnectionInner::Udp(conn) => conn.set_allow_recv_any_version(allow),
            ConnectionInner::Serial(conn) => conn.set_allow_recv_any_version(allow),
        }
    }

    fn allow_recv_any_version(&self) -> bool {
        match &self.0 {
            ConnectionInner::Udp(conn) => conn.allow_recv_any_version(),
            ConnectionInner::Serial(conn) => conn.allow_recv_any_version(),
        }
    }
}
