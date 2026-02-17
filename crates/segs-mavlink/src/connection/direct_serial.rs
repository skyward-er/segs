//! Serial MAVLINK connection

use core::ops::DerefMut;
use std::{
    io::{self, BufReader},
    sync::{Arc, Mutex},
};

use mavlink_core::{
    error::{MessageReadError, MessageWriteError},
    peek_reader::PeekReader,
};
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};

use crate::{
    MavFrame, MavProfile, MavlinkVersion,
    connection::MavConnection,
    core::{ReadVersion, read_versioned_raw_message, write_versioned_msg},
};

pub struct SerialConnection {
    // Separate ports for reading and writing as it's safe to use concurrently.
    // See the official ref: https://github.com/serialport/serialport-rs/blob/321f85e1886eaa1302aef8a600a631bc1c88703a/examples/duplex.rs
    read_port: Mutex<PeekReader<BufReader<Box<dyn SerialPort>>>>,
    write_port: Mutex<Box<dyn SerialPort>>,
    profile: Arc<MavProfile>,
    protocol_version: MavlinkVersion,
    recv_any_version: bool,
}

impl SerialConnection {
    pub(crate) fn new(address: String, baud_rate: u32, profile: Arc<MavProfile>) -> io::Result<Self> {
        let read_port = serialport::new(&address, baud_rate)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .open()?;
        let write_port = read_port.try_clone()?;

        // Calculate a sane default buffer capacity based on the baud rate.
        let read_buffer_capacity = (baud_rate / 100).clamp(1024, 1024 * 8) as usize;
        let buf_reader = BufReader::with_capacity(read_buffer_capacity, read_port);
        Ok(Self {
            read_port: Mutex::new(PeekReader::new(buf_reader)),
            write_port: Mutex::new(write_port),
            profile,
            protocol_version: MavlinkVersion::V2,
            recv_any_version: false,
        })
    }
}

impl MavConnection for SerialConnection {
    fn recv_frame(&self) -> Result<MavFrame, MessageReadError> {
        let mut port = self.read_port.lock().unwrap();
        let version = ReadVersion::from_conn_cfg(self);
        let (raw_msg, profile_msg) = read_versioned_raw_message(&mut port, version, &self.profile.messages)?;
        MavFrame::parse(&raw_msg, profile_msg).map_err(MessageReadError::Parse)
    }

    fn send_frame(&self, frame: MavFrame) -> Result<usize, MessageWriteError> {
        let mut port = self.write_port.lock().unwrap();
        let msg_info = self
            .profile
            .messages
            .get(&frame.message.id)
            .ok_or(MessageWriteError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Message ID {} not found in profile", frame.message.id),
            )))?;
        write_versioned_msg(
            port.deref_mut(),
            self.protocol_version,
            frame.header,
            frame.message,
            msg_info,
        )
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
