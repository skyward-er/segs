use std::{
    collections::BTreeMap,
    io::{Read, Write},
};

use mavlink_bindgen::parser::{MavMessage, extra_crc};
use mavlink_core::{
    MAV_STX, MAV_STX_V2, MavHeader, MavlinkVersion, calculate_crc,
    error::{MessageReadError, MessageWriteError, ParserError},
    peek_reader::PeekReader,
};

use crate::{MavMessage as MsgData, connection::MavConnection};

/// Raw byte representation of a MAVLink message of either version
#[derive(Debug)]
pub enum MAVLinkMessageRaw {
    V1(MAVLinkV1MessageRaw),
    V2(MAVLinkV2MessageRaw),
}

impl MAVLinkMessageRaw {
    pub fn payload(&self) -> &[u8] {
        match self {
            Self::V1(msg) => msg.payload(),
            Self::V2(msg) => msg.payload(),
        }
    }
    pub fn sequence(&self) -> u8 {
        match self {
            Self::V1(msg) => msg.sequence(),
            Self::V2(msg) => msg.sequence(),
        }
    }
    pub fn system_id(&self) -> u8 {
        match self {
            Self::V1(msg) => msg.system_id(),
            Self::V2(msg) => msg.system_id(),
        }
    }
    pub fn component_id(&self) -> u8 {
        match self {
            Self::V1(msg) => msg.component_id(),
            Self::V2(msg) => msg.component_id(),
        }
    }
    pub fn message_id(&self) -> u32 {
        match self {
            Self::V1(msg) => u32::from(msg.message_id()),
            Self::V2(msg) => msg.message_id(),
        }
    }
    pub fn version(&self) -> MavlinkVersion {
        match self {
            Self::V1(_) => MavlinkVersion::V1,
            Self::V2(_) => MavlinkVersion::V2,
        }
    }
}

/// Read and parse a MAVLink message of the specified version from a
/// [`PeekReader`].
///
/// # Errors
///
/// See [`read_` function error documentation](crate#read-errors)
pub fn read_versioned_raw_message<'a, R: Read>(
    r: &mut PeekReader<R>,
    version: ReadVersion,
    messages: &'a BTreeMap<u32, MavMessage>,
) -> Result<(MAVLinkMessageRaw, &'a MavMessage), MessageReadError> {
    match version {
        ReadVersion::Single(MavlinkVersion::V2) => {
            let (msg, profile_msg) = read_v2_raw_message(r, messages)?;
            Ok((MAVLinkMessageRaw::V2(msg), profile_msg))
        }
        ReadVersion::Single(MavlinkVersion::V1) => {
            let (msg, profile_msg) = read_v1_raw_message(r, messages)?;
            Ok((MAVLinkMessageRaw::V1(msg), profile_msg))
        }
        ReadVersion::Any => read_any_raw_message(r, messages),
    }
}

/// Write a MAVLink message using the given mavlink version to a [`Write`]r.
///
/// # Errors
///
/// See [`write_` function error documentation](crate#write-errors).
pub fn write_versioned_msg<W: Write>(
    w: &mut W,
    version: MavlinkVersion,
    header: MavHeader,
    data: MsgData,
    msg_info: &MavMessage,
) -> Result<usize, MessageWriteError> {
    match version {
        MavlinkVersion::V2 => write_v2_msg(w, header, data, msg_info),
        MavlinkVersion::V1 => write_v1_msg(w, header, data, msg_info),
    }
}

/// Read a raw MAVLink 1 message from a [`PeekReader`].
///
/// # Errors
///
/// See [`read_` function error documentation](crate#read-errors)
pub fn read_v1_raw_message<'a, R: Read>(
    reader: &mut PeekReader<R>,
    messages: &'a BTreeMap<u32, MavMessage>,
) -> Result<(MAVLinkV1MessageRaw, &'a MavMessage), MessageReadError> {
    loop {
        // search for the magic framing value indicating start of mavlink message
        while reader.peek_exact(1)?[0] != MAV_STX {
            reader.consume(1);
        }

        if let Some((message, profile_msg)) = try_decode_v1(reader, messages)? {
            return Ok((message, profile_msg));
        }

        reader.consume(1);
    }
}

/// Read a raw MAVLink 2 message from a [`PeekReader`].
///
/// # Errors
///
/// See [`read_` function error documentation](crate#read-errors)
fn read_v2_raw_message<'a, R: Read>(
    reader: &mut PeekReader<R>,
    messages: &'a BTreeMap<u32, MavMessage>,
) -> Result<(MAVLinkV2MessageRaw, &'a MavMessage), MessageReadError> {
    loop {
        // search for the magic framing value indicating start of mavlink message
        while reader.peek_exact(1)?[0] != MAV_STX_V2 {
            reader.consume(1);
        }

        if let Some((message, profile_msg)) = try_decode_v2(reader, messages)? {
            return Ok((message, profile_msg));
        }
    }
}

/// Read a raw MAVLink 1 or 2 message from a [`PeekReader`].
///
/// # Errors
///
/// See [`read_` function error documentation](crate#read-errors)
fn read_any_raw_message<'a, R: Read>(
    reader: &mut PeekReader<R>,
    messages: &'a BTreeMap<u32, MavMessage>,
) -> Result<(MAVLinkMessageRaw, &'a MavMessage), MessageReadError> {
    loop {
        // search for the magic framing value indicating start of MAVLink message
        let version = loop {
            let byte = reader.peek_exact(1)?[0];
            if byte == MAV_STX {
                break MavlinkVersion::V1;
            }
            if byte == MAV_STX_V2 {
                break MavlinkVersion::V2;
            }
            reader.consume(1);
        };
        match version {
            MavlinkVersion::V1 => {
                if let Some((message, profile_msg)) = try_decode_v1(reader, messages)? {
                    return Ok((MAVLinkMessageRaw::V1(message), profile_msg));
                }

                reader.consume(1);
            }
            MavlinkVersion::V2 => {
                if let Some((message, profile_msg)) = try_decode_v2(reader, messages)? {
                    return Ok((MAVLinkMessageRaw::V2(message), profile_msg));
                }
            }
        }
    }
}

/// Write a MAVLink 2 message to a [`Write`]r.
///
/// # Errors
///
/// See [`write_` function error documentation](crate#write-errors).
pub fn write_v2_msg<W: Write>(
    w: &mut W,
    header: MavHeader,
    data: MsgData,
    msg_info: &MavMessage,
) -> Result<usize, MessageWriteError> {
    let mut message_raw = MAVLinkV2MessageRaw::new();
    message_raw.serialize_message(header, data, msg_info);

    let payload_length: usize = message_raw.payload_length().into();
    let len = 1 + MAVLinkV2MessageRaw::HEADER_SIZE + payload_length + 2;

    w.write_all(&message_raw.0[..len])?;

    Ok(len)
}

/// Write a MAVLink 1 message to a [`Write`]r.
///
/// # Errors
///
/// See [`write_` function error documentation](crate#write-errors).
pub fn write_v1_msg<W: Write>(
    w: &mut W,
    header: MavHeader,
    data: MsgData,
    msg_info: &MavMessage,
) -> Result<usize, MessageWriteError> {
    if data.id > u8::MAX.into() {
        return Err(MessageWriteError::MAVLink2Only);
    }
    let mut message_raw = MAVLinkV1MessageRaw::new();
    message_raw.serialize_message(header, data, msg_info);

    let payload_length: usize = message_raw.payload_length().into();
    let len = 1 + MAVLinkV1MessageRaw::HEADER_SIZE + payload_length + 2;

    w.write_all(&message_raw.0[..len])?;

    Ok(len)
}

fn try_decode_v1<'a, R: Read>(
    reader: &mut PeekReader<R>,
    messages: &'a BTreeMap<u32, MavMessage>,
) -> Result<Option<(MAVLinkV1MessageRaw, &'a MavMessage)>, MessageReadError> {
    let mut message = MAVLinkV1MessageRaw::new();
    let whole_header_size = MAVLinkV1MessageRaw::HEADER_SIZE + 1;

    message.0[0] = MAV_STX;
    let header = &reader.peek_exact(whole_header_size)?[1..whole_header_size];
    message.mut_header().copy_from_slice(header);

    // Get the message definition from the profile
    let msg_id = message.message_id() as u32;
    let profile_msg = messages
        .get(&msg_id)
        .ok_or(MessageReadError::Parse(ParserError::UnknownMessage { id: msg_id }))?;

    let packet_length = message.raw_bytes().len();
    let payload_and_checksum = &reader.peek_exact(packet_length)?[whole_header_size..packet_length];
    message.mut_payload_and_checksum().copy_from_slice(payload_and_checksum);

    // retry if CRC failed after previous STX
    // (an STX byte may appear in the middle of a message)
    if message.has_valid_crc(profile_msg) {
        reader.consume(message.raw_bytes().len());
        Ok(Some((message, profile_msg)))
    } else {
        Ok(None)
    }
}

fn try_decode_v2<'a, R: Read>(
    reader: &mut PeekReader<R>,
    messages: &'a BTreeMap<u32, MavMessage>,
) -> Result<Option<(MAVLinkV2MessageRaw, &'a MavMessage)>, MessageReadError> {
    let mut message = MAVLinkV2MessageRaw::new();
    let whole_header_size = MAVLinkV2MessageRaw::HEADER_SIZE + 1;

    message.0[0] = MAV_STX_V2;
    let header = &reader.peek_exact(whole_header_size)?[1..whole_header_size];
    message.mut_header().copy_from_slice(header);

    // Get the message definition from the profile
    let msg_id = message.message_id();
    let profile_msg = messages
        .get(&msg_id)
        .ok_or(MessageReadError::Parse(ParserError::UnknownMessage { id: msg_id }))?;

    if message.incompatibility_flags() & !MAVLINK_SUPPORTED_IFLAGS > 0 {
        // if there are incompatibility flags set that we do not know discard the
        // message
        reader.consume(1);
        return Ok(None);
    }

    let packet_length = message.raw_bytes().len();
    let payload_and_checksum_and_sign = &reader.peek_exact(packet_length)?[whole_header_size..packet_length];
    message
        .mut_payload_and_checksum_and_sign()
        .copy_from_slice(payload_and_checksum_and_sign);

    if message.has_valid_crc(profile_msg) {
        // even if the signature turn out to be invalid the valid crc shows that the
        // received data presents a valid message as opposed to random bytes
        reader.consume(message.raw_bytes().len());
    } else {
        reader.consume(1);
        return Ok(None);
    }

    Ok(Some((message, profile_msg)))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Byte buffer containing the raw representation of a MAVLink 1 message
/// beginning with the STX marker.
///
/// Follow protocol definition: <https://mavlink.io/en/guide/serialization.html#v1_packet_format>.
/// Maximum size is 263 bytes.
pub struct MAVLinkV1MessageRaw([u8; 1 + Self::HEADER_SIZE + 255 + 2]);

impl Default for MAVLinkV1MessageRaw {
    fn default() -> Self {
        Self::new()
    }
}

impl MAVLinkV1MessageRaw {
    const HEADER_SIZE: usize = 5;

    /// Create a new raw MAVLink 1 message filled with zeros.
    pub const fn new() -> Self {
        Self([0; 1 + Self::HEADER_SIZE + 255 + 2])
    }

    /// Create a new raw MAVLink 1 message from a given buffer.
    ///
    /// Note: This method does not guarantee that the constructed MAVLink
    /// message is valid.
    pub const fn from_bytes_unparsed(bytes: [u8; 1 + Self::HEADER_SIZE + 255 + 2]) -> Self {
        Self(bytes)
    }

    /// Read access to its internal buffer.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Mutable reference to its internal buffer.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }

    /// Deconstruct the MAVLink message into its owned internal buffer.
    #[inline]
    pub fn into_inner(self) -> [u8; 1 + Self::HEADER_SIZE + 255 + 2] {
        self.0
    }

    /// Reference to the 5 byte header slice of the message
    #[inline]
    pub fn header(&self) -> &[u8] {
        &self.0[1..=Self::HEADER_SIZE]
    }

    /// Mutable reference to the 5 byte header slice of the message
    #[inline]
    fn mut_header(&mut self) -> &mut [u8] {
        &mut self.0[1..=Self::HEADER_SIZE]
    }

    /// Size of the payload of the message
    #[inline]
    pub fn payload_length(&self) -> u8 {
        self.0[1]
    }

    /// Packet sequence number
    #[inline]
    pub fn sequence(&self) -> u8 {
        self.0[2]
    }

    /// Message sender System ID
    #[inline]
    pub fn system_id(&self) -> u8 {
        self.0[3]
    }

    /// Message sender Component ID
    #[inline]
    pub fn component_id(&self) -> u8 {
        self.0[4]
    }

    /// Message ID
    #[inline]
    pub fn message_id(&self) -> u8 {
        self.0[5]
    }

    /// Reference to the payload byte slice of the message
    #[inline]
    pub fn payload(&self) -> &[u8] {
        let payload_length: usize = self.payload_length().into();
        &self.0[(1 + Self::HEADER_SIZE)..(1 + Self::HEADER_SIZE + payload_length)]
    }

    /// [CRC-16 checksum](https://mavlink.io/en/guide/serialization.html#checksum) field of the message
    #[inline]
    pub fn checksum(&self) -> u16 {
        let payload_length: usize = self.payload_length().into();
        u16::from_le_bytes([
            self.0[1 + Self::HEADER_SIZE + payload_length],
            self.0[1 + Self::HEADER_SIZE + payload_length + 1],
        ])
    }

    #[inline]
    fn mut_payload_and_checksum(&mut self) -> &mut [u8] {
        let payload_length: usize = self.payload_length().into();
        &mut self.0[(1 + Self::HEADER_SIZE)..(1 + Self::HEADER_SIZE + payload_length + 2)]
    }

    /// Checks wether the message’s [CRC-16 checksum](https://mavlink.io/en/guide/serialization.html#checksum) calculation matches its checksum field.
    #[inline]
    pub fn has_valid_crc(&self, message: &MavMessage) -> bool {
        let payload_length: usize = self.payload_length().into();
        self.checksum() == calculate_crc(&self.0[1..(1 + Self::HEADER_SIZE + payload_length)], extra_crc(message))
    }

    /// Raw byte slice of the message
    pub fn raw_bytes(&self) -> &[u8] {
        let payload_length = self.payload_length() as usize;
        &self.0[..(1 + Self::HEADER_SIZE + payload_length + 2)]
    }

    /// # Panics
    ///
    /// If the `msgid` parameter exceeds 255 and is therefore not supported for
    /// MAVLink 1
    fn serialize_stx_and_header_and_crc(
        &mut self,
        header: MavHeader,
        msgid: u32,
        payload_length: usize,
        extra_crc: u8,
    ) {
        self.0[0] = MAV_STX;

        let header_buf = self.mut_header();
        header_buf.copy_from_slice(&[
            payload_length as u8,
            header.sequence,
            header.system_id,
            header.component_id,
            msgid.try_into().unwrap(),
        ]);

        let crc = calculate_crc(&self.0[1..(1 + Self::HEADER_SIZE + payload_length)], extra_crc);
        self.0[(1 + Self::HEADER_SIZE + payload_length)..(1 + Self::HEADER_SIZE + payload_length + 2)]
            .copy_from_slice(&crc.to_le_bytes());
    }

    /// Serialize a Message with a given header into this raw message buffer.
    pub fn serialize_message(&mut self, header: MavHeader, data: MsgData, msg_info: &MavMessage) {
        let payload_buf = &mut self.0[(1 + Self::HEADER_SIZE)..(1 + Self::HEADER_SIZE + 255)];
        let msgid = data.id;
        let payload_length = data.ser(MavlinkVersion::V1, payload_buf);

        self.serialize_stx_and_header_and_crc(header, msgid, payload_length, extra_crc(msg_info));
    }

    // /// # Panics
    // ///
    // /// If the `MessageData`'s `ID` exceeds 255 and is therefore not supported
    // for MAVLink 1 pub fn serialize_message_data<D: MessageData>(&mut self,
    // header: MavHeader, message_data: &D) {     let payload_buf = &mut
    // self.0[(1 + Self::HEADER_SIZE)..(1 + Self::HEADER_SIZE + 255)];
    //     let payload_length = message_data.ser(MavlinkVersion::V1, payload_buf);

    //     self.serialize_stx_and_header_and_crc(header, D::ID, payload_length,
    // D::EXTRA_CRC); }
}

const MAVLINK_IFLAG_SIGNED: u8 = 0x01;
const MAVLINK_SUPPORTED_IFLAGS: u8 = MAVLINK_IFLAG_SIGNED;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Byte buffer containing the raw representation of a MAVLink 2 message
/// beginning with the STX marker.
///
/// Follow protocol definition: <https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format>.
/// Maximum size is [280 bytes](MAX_FRAME_SIZE).
pub struct MAVLinkV2MessageRaw([u8; 1 + Self::HEADER_SIZE + 255 + 2 + Self::SIGNATURE_SIZE]);

impl Default for MAVLinkV2MessageRaw {
    fn default() -> Self {
        Self::new()
    }
}

impl MAVLinkV2MessageRaw {
    const HEADER_SIZE: usize = 9;
    const SIGNATURE_SIZE: usize = 13;

    /// Create a new raw MAVLink 2 message filled with zeros.
    pub const fn new() -> Self {
        Self([0; 1 + Self::HEADER_SIZE + 255 + 2 + Self::SIGNATURE_SIZE])
    }

    /// Create a new raw MAVLink 1 message from a given buffer.
    ///
    /// Note: This method does not guarantee that the constructed MAVLink
    /// message is valid.
    pub const fn from_bytes_unparsed(bytes: [u8; 1 + Self::HEADER_SIZE + 255 + 2 + Self::SIGNATURE_SIZE]) -> Self {
        Self(bytes)
    }

    /// Read access to its internal buffer.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Mutable reference to its internal buffer.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }

    /// Deconstruct the MAVLink message into its owned internal buffer.
    #[inline]
    pub fn into_inner(self) -> [u8; 1 + Self::HEADER_SIZE + 255 + 2 + Self::SIGNATURE_SIZE] {
        self.0
    }

    /// Reference to the 9 byte header slice of the message
    #[inline]
    pub fn header(&self) -> &[u8] {
        &self.0[1..=Self::HEADER_SIZE]
    }

    /// Mutable reference to the header byte slice of the message
    #[inline]
    fn mut_header(&mut self) -> &mut [u8] {
        &mut self.0[1..=Self::HEADER_SIZE]
    }

    /// Size of the payload of the message
    #[inline]
    pub fn payload_length(&self) -> u8 {
        self.0[1]
    }

    /// [Incompatiblity flags](https://mavlink.io/en/guide/serialization.html#incompat_flags) of the message
    ///
    /// Currently the only supported incompatebility flag is
    /// `MAVLINK_IFLAG_SIGNED`.
    #[inline]
    pub fn incompatibility_flags(&self) -> u8 {
        self.0[2]
    }

    /// Mutable reference to the [incompatiblity flags](https://mavlink.io/en/guide/serialization.html#incompat_flags) of the message
    ///
    /// Currently the only supported incompatebility flag is
    /// `MAVLINK_IFLAG_SIGNED`.
    #[inline]
    pub fn incompatibility_flags_mut(&mut self) -> &mut u8 {
        &mut self.0[2]
    }

    /// [Compatibility Flags](https://mavlink.io/en/guide/serialization.html#compat_flags) of the message
    #[inline]
    pub fn compatibility_flags(&self) -> u8 {
        self.0[3]
    }

    /// Packet sequence number
    #[inline]
    pub fn sequence(&self) -> u8 {
        self.0[4]
    }

    /// Message sender System ID
    #[inline]
    pub fn system_id(&self) -> u8 {
        self.0[5]
    }

    /// Message sender Component ID
    #[inline]
    pub fn component_id(&self) -> u8 {
        self.0[6]
    }

    /// Message ID
    #[inline]
    pub fn message_id(&self) -> u32 {
        u32::from_le_bytes([self.0[7], self.0[8], self.0[9], 0])
    }

    /// Reference to the payload byte slice of the message
    #[inline]
    pub fn payload(&self) -> &[u8] {
        let payload_length: usize = self.payload_length().into();
        &self.0[(1 + Self::HEADER_SIZE)..(1 + Self::HEADER_SIZE + payload_length)]
    }

    /// [CRC-16 checksum](https://mavlink.io/en/guide/serialization.html#checksum) field of the message
    #[inline]
    pub fn checksum(&self) -> u16 {
        let payload_length: usize = self.payload_length().into();
        u16::from_le_bytes([
            self.0[1 + Self::HEADER_SIZE + payload_length],
            self.0[1 + Self::HEADER_SIZE + payload_length + 1],
        ])
    }

    fn mut_payload_and_checksum_and_sign(&mut self) -> &mut [u8] {
        let payload_length: usize = self.payload_length().into();

        // Signature to ensure the link is tamper-proof.
        let signature_size = if (self.incompatibility_flags() & MAVLINK_IFLAG_SIGNED) == 0 {
            0
        } else {
            Self::SIGNATURE_SIZE
        };

        &mut self.0[(1 + Self::HEADER_SIZE)..(1 + Self::HEADER_SIZE + payload_length + signature_size + 2)]
    }

    /// Checks wether the message's [CRC-16 checksum](https://mavlink.io/en/guide/serialization.html#checksum) calculation matches its checksum field.
    #[inline]
    pub fn has_valid_crc(&self, message: &MavMessage) -> bool {
        let payload_length: usize = self.payload_length().into();
        self.checksum() == calculate_crc(&self.0[1..(1 + Self::HEADER_SIZE + payload_length)], extra_crc(message))
    }

    /// Raw byte slice of the message
    pub fn raw_bytes(&self) -> &[u8] {
        let payload_length = self.payload_length() as usize;

        let signature_size = if (self.incompatibility_flags() & MAVLINK_IFLAG_SIGNED) == 0 {
            0
        } else {
            Self::SIGNATURE_SIZE
        };

        &self.0[..(1 + Self::HEADER_SIZE + payload_length + signature_size + 2)]
    }

    fn serialize_stx_and_header_and_crc(
        &mut self,
        header: MavHeader,
        msgid: u32,
        payload_length: usize,
        extra_crc: u8,
        incompat_flags: u8,
    ) {
        self.0[0] = MAV_STX_V2;
        let msgid_bytes = msgid.to_le_bytes();

        let header_buf = self.mut_header();
        header_buf.copy_from_slice(&[
            payload_length as u8,
            incompat_flags,
            0, //compat_flags
            header.sequence,
            header.system_id,
            header.component_id,
            msgid_bytes[0],
            msgid_bytes[1],
            msgid_bytes[2],
        ]);

        let crc = calculate_crc(&self.0[1..(1 + Self::HEADER_SIZE + payload_length)], extra_crc);
        self.0[(1 + Self::HEADER_SIZE + payload_length)..(1 + Self::HEADER_SIZE + payload_length + 2)]
            .copy_from_slice(&crc.to_le_bytes());
    }

    /// Serialize a [Message] with a given header into this raw message buffer.
    ///
    /// This does not set any compatiblity or incompatiblity flags.
    pub fn serialize_message(&mut self, header: MavHeader, data: MsgData, msg_info: &MavMessage) {
        let payload_buf = &mut self.0[(1 + Self::HEADER_SIZE)..(1 + Self::HEADER_SIZE + 255)];
        let msgid = data.id;
        let payload_length = data.ser(MavlinkVersion::V2, payload_buf);

        self.serialize_stx_and_header_and_crc(header, msgid, payload_length, extra_crc(msg_info), 0);
    }

    // FIXME: remove this if not used
    // pub fn serialize_message_data<D: MessageData>(&mut self, header: MavHeader,
    // message_data: &D) {     let payload_buf = &mut self.0[(1 +
    // Self::HEADER_SIZE)..(1 + Self::HEADER_SIZE + 255)];
    //     let payload_length = message_data.ser(MavlinkVersion::V2, payload_buf);

    //     self.serialize_stx_and_header_and_crc(header, D::ID, payload_length,
    // D::EXTRA_CRC, 0); }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// MAVLink Version selection when attempting to read
pub enum ReadVersion {
    /// Only attempt to read using a single MAVLink version
    Single(MavlinkVersion),
    /// Attempt to read messages from both MAVLink versions
    Any,
}

impl ReadVersion {
    pub fn from_conn_cfg<C: MavConnection>(conn: &C) -> Self {
        if conn.allow_recv_any_version() {
            Self::Any
        } else {
            Self::Single(conn.protocol_version())
        }
    }
}
