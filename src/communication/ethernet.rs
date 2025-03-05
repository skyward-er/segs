use std::{
    collections::VecDeque,
    io::Read,
    net::UdpSocket,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};

use anyhow::Context;
use ring_channel::{RingReceiver, RingSender};
use skyward_mavlink::mavlink::{
    MavFrame,
    error::{MessageReadError, MessageWriteError},
    read_v1_msg, write_v1_msg,
};
use tracing::{debug, error, trace};

use crate::{
    error::ErrInstrument,
    mavlink::{
        MavHeader, MavMessage, MavlinkVersion, TimedMessage, peek_reader::PeekReader,
        read_versioned_msg,
    },
};

use super::{Connectable, ConnectionError, MessageTransceiver, Transceivers};

#[derive(Debug, Clone)]
pub struct EthernetConfiguration {
    pub port: u16,
}

impl Connectable for EthernetConfiguration {
    type Connected = EthernetTransceiver;

    fn connect(self) -> Result<Self::Connected, ConnectionError> {
        let socket = std::net::UdpSocket::bind(format!("0.0.0.0:{}", self.port))?;
        debug!("Connected to Ethernet port on port {}", self.port);
        let reader = Mutex::new(PeekReader::new(VecDeque::new()));
        Ok(EthernetTransceiver { socket, reader })
    }
}

/// Manages a connection to a Ethernet port.
pub struct EthernetTransceiver {
    socket: UdpSocket,
    reader: Mutex<PeekReader<VecDeque<u8>>>,
}

impl MessageTransceiver for EthernetTransceiver {
    fn wait_for_message(&self) -> Result<TimedMessage, MessageReadError> {
        let mut reader = self.reader.lock().log_unwrap();
        let read = self.socket.recv(reader.reader_mut().make_contiguous())?;
        trace!("Received {} bytes", read);
        let (_, res) = read_v1_msg(&mut reader)?;
        debug!("Received message: {:?}", res);
        Ok(TimedMessage::just_received(res))
    }

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
