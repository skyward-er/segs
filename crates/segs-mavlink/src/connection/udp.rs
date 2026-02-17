//! UDP MAVLink connection

use core::ops::DerefMut;
use std::{
    collections::VecDeque,
    io::{self, Read},
    net::{SocketAddrV4, UdpSocket},
    sync::{Arc, Mutex},
};

use mavlink_core::{
    MavlinkVersion,
    error::{MessageReadError, MessageWriteError},
    peek_reader::PeekReader,
};

use crate::{
    MavFrame, MavProfile,
    connection::MavConnection,
    core::{ReadVersion, read_versioned_raw_message, write_versioned_msg},
};

struct UdpRead {
    socket: UdpSocket,
    buffer: VecDeque<u8>,
}

const MTU_SIZE: usize = 1500;
impl Read for UdpRead {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if !self.buffer.is_empty() {
            self.buffer.read(buf)
        } else {
            let mut read_buffer = [0u8; MTU_SIZE];
            let n_buffer = self.socket.recv(&mut read_buffer)?;
            let n = (&read_buffer[0..n_buffer]).read(buf)?;
            self.buffer.extend(&read_buffer[n..n_buffer]);
            Ok(n)
        }
    }
}

struct UdpWrite {
    socket: UdpSocket,
    dest: SocketAddrV4,
}

pub struct UdpConnection {
    reader: Mutex<PeekReader<UdpRead>>,
    writer: Mutex<UdpWrite>,
    profile: Arc<MavProfile>,
    protocol_version: MavlinkVersion,
    recv_any_version: bool,
}

impl UdpConnection {
    pub(crate) fn new(
        listen_address: SocketAddrV4,
        send_address: SocketAddrV4,
        profile: Arc<MavProfile>,
    ) -> io::Result<Self> {
        let socket = UdpSocket::bind(listen_address)?;
        socket.set_broadcast(true)?;
        Ok(Self {
            reader: Mutex::new(PeekReader::new(UdpRead {
                socket: socket.try_clone()?,
                buffer: VecDeque::new(),
            })),
            writer: Mutex::new(UdpWrite {
                socket,
                dest: send_address,
            }),
            profile,
            protocol_version: MavlinkVersion::V2,
            recv_any_version: false,
        })
    }
}

impl MavConnection for UdpConnection {
    fn recv_frame(&self) -> Result<MavFrame, MessageReadError> {
        let mut reader = self.reader.lock().unwrap();
        let version = ReadVersion::from_conn_cfg(self);
        let (raw_msg, profile_msg) = read_versioned_raw_message(reader.deref_mut(), version, &self.profile.messages)?;
        MavFrame::parse(&raw_msg, profile_msg).map_err(MessageReadError::Parse)
    }

    fn send_frame(&self, frame: MavFrame) -> Result<usize, MessageWriteError> {
        let mut guard = self.writer.lock().unwrap();
        let state = &mut *guard;
        let mut buf: Vec<u8> = Vec::new();

        let profile_msg = self
            .profile
            .messages
            .get(&frame.message.id)
            .ok_or(MessageWriteError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Message ID {} not found in profile", frame.message.id),
            )))?;
        write_versioned_msg(
            &mut buf,
            self.protocol_version,
            frame.header,
            frame.message,
            profile_msg,
        )?;
        Ok(state.socket.send_to(&buf, state.dest)?)
    }

    fn set_protocol_version(&mut self, version: MavlinkVersion) {
        self.protocol_version = version;
    }

    fn protocol_version(&self) -> MavlinkVersion {
        self.protocol_version
    }

    fn set_allow_recv_any_version(&mut self, allow: bool) {
        self.recv_any_version = allow;
    }

    fn allow_recv_any_version(&self) -> bool {
        self.recv_any_version
    }
}
