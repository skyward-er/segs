//! CCSDS Space Packet Protocol framing and payload codec.
//!
//! Covers CCSDS 133.0-B primary header encoding/decoding, big-endian payload
//! decoding for telemetry, and command packet encoding.

use thiserror::Error;

use crate::cosmos::{CommandDef, FieldType, TelemetryPacketDef};

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum CcsdsError {
    #[error("buffer too short for CCSDS header: need 6 bytes, got {0}")]
    HeaderTooShort(usize),
    #[error(
        "payload too short: need {expected} bytes for field '{field}', only {actual} remaining"
    )]
    PayloadTooShort {
        expected: usize,
        actual: usize,
        field: String,
    },
    #[error("unsupported field encoding: '{field}' ({ty} {bits}-bit)")]
    UnsupportedField {
        field: String,
        ty: String,
        bits: u32,
    },
}

// ── CCSDS Primary Header ─────────────────────────────────────────────────────

/// Decoded CCSDS Space Packet primary header (6 bytes).
///
/// Bit layout (big-endian):
/// ```text
/// Word 0: [VER 3][TYPE 1][SHF 1][APID 11]
/// Word 1: [SEQFLAGS 2][SEQCNT 14]
/// Word 2: [DATA_LENGTH 16]
/// ```
/// `data_length` = (number of octets in the packet data field) − 1.
#[derive(Debug, Clone, PartialEq)]
pub struct CcsdsHeader {
    pub version: u8,
    /// 0 = telemetry, 1 = command.
    pub packet_type: u8,
    pub secondary_header_flag: bool,
    /// Application Process Identifier (11 bits).
    pub apid: u16,
    /// Sequence flags (2 bits): 3 = standalone packet.
    pub sequence_flags: u8,
    /// Packet sequence count (14 bits).
    pub sequence_count: u16,
    /// Byte count of the data field minus 1.
    pub data_length: u16,
}

impl CcsdsHeader {
    /// Parse a CCSDS primary header from the first 6 bytes of `buf`.
    pub fn decode(buf: &[u8]) -> Result<Self, CcsdsError> {
        if buf.len() < 6 {
            return Err(CcsdsError::HeaderTooShort(buf.len()));
        }
        let w0 = u16::from_be_bytes([buf[0], buf[1]]);
        let w1 = u16::from_be_bytes([buf[2], buf[3]]);
        let w2 = u16::from_be_bytes([buf[4], buf[5]]);
        Ok(Self {
            version: ((w0 >> 13) & 0x7) as u8,
            packet_type: ((w0 >> 12) & 0x1) as u8,
            secondary_header_flag: ((w0 >> 11) & 0x1) != 0,
            apid: w0 & 0x07FF,
            sequence_flags: ((w1 >> 14) & 0x3) as u8,
            sequence_count: w1 & 0x3FFF,
            data_length: w2,
        })
    }

    /// Encode the header into 6 bytes.
    pub fn encode(&self) -> [u8; 6] {
        let w0: u16 = (u16::from(self.version & 0x7) << 13)
            | (u16::from(self.packet_type & 0x1) << 12)
            | (u16::from(self.secondary_header_flag) << 11)
            | (self.apid & 0x07FF);
        let w1: u16 = (u16::from(self.sequence_flags & 0x3) << 14) | (self.sequence_count & 0x3FFF);
        let [b0, b1] = w0.to_be_bytes();
        let [b2, b3] = w1.to_be_bytes();
        let [b4, b5] = self.data_length.to_be_bytes();
        [b0, b1, b2, b3, b4, b5]
    }

    /// Total packet length in bytes (6-byte header + data field).
    pub fn total_len(&self) -> usize {
        6 + self.data_length as usize + 1
    }

    /// Byte count of the data field (data_length + 1).
    pub fn data_field_len(&self) -> usize {
        self.data_length as usize + 1
    }
}

// ── Telemetry ─────────────────────────────────────────────────────────────────

/// Runtime value of a single telemetry field.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Uint(u64),
    Int(i64),
    Float(f64),
}

/// A decoded telemetry packet payload.
#[derive(Debug, Clone)]
pub struct TelemetryPacket {
    pub apid: u16,
    pub sequence_count: u16,
    /// One entry per field in the matching [`TelemetryPacketDef`], in order.
    pub fields: Vec<FieldValue>,
}

/// Decode the payload bytes of a telemetry packet (the bytes *after* the 6-byte
/// CCSDS primary header) according to `def`.
///
/// Fields are decoded from a big-endian bit stream, supporting sub-byte fields
/// that are packed contiguously (standard CCSDS secondary header layout).
pub fn decode_telemetry(
    payload: &[u8],
    seq: u16,
    apid: u16,
    def: &TelemetryPacketDef,
) -> Result<TelemetryPacket, CcsdsError> {
    let mut fields = Vec::with_capacity(def.fields.len());
    let mut bit_offset = 0usize;

    for field_def in &def.fields {
        let bit_size = field_def.bit_size as usize;
        let end_byte = (bit_offset + bit_size + 7) / 8;

        if end_byte > payload.len() {
            return Err(CcsdsError::PayloadTooShort {
                expected: end_byte,
                actual: payload.len(),
                field: field_def.name.clone(),
            });
        }

        let raw_bits = extract_bits(payload, bit_offset, bit_size);
        let value = interpret_bits(raw_bits, &field_def.ty, bit_size);
        fields.push(value);
        bit_offset += bit_size;
    }

    Ok(TelemetryPacket {
        apid,
        sequence_count: seq,
        fields,
    })
}

/// Extract `bit_size` bits starting at `bit_offset` from a big-endian byte slice.
/// Supports up to 64-bit values.
fn extract_bits(payload: &[u8], bit_offset: usize, bit_size: usize) -> u64 {
    let start_byte = bit_offset / 8;
    let end_byte = (bit_offset + bit_size + 7) / 8;
    let bytes_needed = end_byte - start_byte;

    // Load up to 8 bytes into a u64 accumulator (big-endian).
    let mut acc: u64 = 0;
    for i in 0..bytes_needed.min(8) {
        acc = (acc << 8) | payload[start_byte + i] as u64;
    }

    // Remove trailing bits that belong to the next field.
    let trailing = bytes_needed * 8 - (bit_offset % 8) - bit_size;
    acc >>= trailing;

    let mask = if bit_size >= 64 { u64::MAX } else { (1u64 << bit_size) - 1 };
    acc & mask
}

fn interpret_bits(raw: u64, ty: &FieldType, bits: usize) -> FieldValue {
    match ty {
        FieldType::Uint => FieldValue::Uint(raw),
        FieldType::Int => {
            // Sign-extend from `bits` to 64 bits.
            let shift = 64 - bits;
            FieldValue::Int(((raw as i64) << shift) >> shift)
        }
        FieldType::Float => match bits {
            32 => FieldValue::Float(f32::from_bits(raw as u32) as f64),
            64 => FieldValue::Float(f64::from_bits(raw)),
            // Non-standard float width — store raw bits as float zero.
            _ => FieldValue::Float(0.0),
        },
    }
}

// ── Command encoding ──────────────────────────────────────────────────────────

/// A command ready to be sent: carries the target command definition and the
/// operator-supplied parameter values (one `u64` per non-hidden parameter).
#[derive(Debug, Clone)]
pub struct CommandPacket {
    pub command_id: u32,
    /// One value per entry in `CommandDef::params`.
    pub param_values: Vec<u64>,
}

impl CommandPacket {
    pub fn new(def: &CommandDef) -> Self {
        Self {
            command_id: def.command_id,
            param_values: def.default_param_values(),
        }
    }
}

/// Encode a command into a CCSDS-framed byte vector ready for transmission.
///
/// # Wire layout
/// ```text
/// [6-byte primary header]
/// [4-byte CommandId, big-endian]   ← secondary header
/// [user params, big-endian]
/// ```
pub fn encode_command(cmd: &CommandPacket, def: &CommandDef, apid: u16, seq: u16) -> Vec<u8> {
    // Build data field first so we can compute data_length
    let mut data: Vec<u8> = Vec::new();

    // Secondary header: 4-byte CommandId
    data.extend_from_slice(&cmd.command_id.to_be_bytes());

    // User parameters
    for (param_def, &value) in def.params.iter().zip(cmd.param_values.iter()) {
        match param_def.bit_size {
            8 => data.push(value as u8),
            16 => data.extend_from_slice(&(value as u16).to_be_bytes()),
            32 => data.extend_from_slice(&(value as u32).to_be_bytes()),
            64 => data.extend_from_slice(&value.to_be_bytes()),
            _ => data.push(value as u8), // fallback: treat as u8
        }
    }

    let data_length = (data.len() - 1) as u16; // data_length = payload_bytes - 1
    let header = CcsdsHeader {
        version: 0,
        packet_type: 1, // command
        secondary_header_flag: true,
        apid,
        sequence_flags: 3, // standalone
        sequence_count: seq,
        data_length,
    };

    let mut out = header.encode().to_vec();
    out.extend_from_slice(&data);
    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cosmos::types::{
        CmdParamDef, CommandDef, FieldType, TelemetryPacketDef, TlmFieldDef,
    };

    // ── CcsdsHeader ──────────────────────────────────────────────────

    #[test]
    fn header_roundtrip() {
        let h = CcsdsHeader {
            version: 0,
            packet_type: 0, // telemetry
            secondary_header_flag: true,
            apid: 3,
            sequence_flags: 3,
            sequence_count: 42,
            data_length: 99,
        };
        let encoded = h.encode();
        let decoded = CcsdsHeader::decode(&encoded).unwrap();
        assert_eq!(h, decoded);
    }

    #[test]
    fn header_decode_known_bytes() {
        // Build a header for apid=3, tlm, seq=0, data_length=4
        // word0 = 0b000_0_1_00000000011 = 0x0003 (ver=0, type=0, shf=1, apid=3)
        //   actually SHF=1 → bit 11 → 0x0803
        // word0: ver=0(3b), type=0(1b), shf=1(1b), apid=3(11b)
        //   = 0b000_0_1_00000000011 = 0x0803
        // word1: seqflags=3(2b), seqcnt=0(14b) = 0b11_00000000000000 = 0xC000
        // word2: 0x0004
        let bytes = [0x08u8, 0x03, 0xC0, 0x00, 0x00, 0x04];
        let h = CcsdsHeader::decode(&bytes).unwrap();
        assert_eq!(h.apid, 3);
        assert_eq!(h.packet_type, 0);
        assert!(h.secondary_header_flag);
        assert_eq!(h.sequence_flags, 3);
        assert_eq!(h.sequence_count, 0);
        assert_eq!(h.data_length, 4);
        assert_eq!(h.total_len(), 11); // 6 + 4 + 1
    }

    #[test]
    fn header_too_short() {
        assert!(matches!(
            CcsdsHeader::decode(&[0u8; 5]),
            Err(CcsdsError::HeaderTooShort(5))
        ));
    }

    // ── decode_telemetry ─────────────────────────────────────────────

    fn make_tlm_def() -> TelemetryPacketDef {
        TelemetryPacketDef {
            target: "T".into(),
            name: "PKT".into(),
            apid: 3,
            fields: vec![
                TlmFieldDef {
                    name: "counter".into(),
                    bit_size: 32,
                    ty: FieldType::Uint,
                    states: vec![],
                    units: None,
                    description: String::new(),
                },
                TlmFieldDef {
                    name: "temperature".into(),
                    bit_size: 32,
                    ty: FieldType::Float,
                    states: vec![],
                    units: Some("degC".into()),
                    description: String::new(),
                },
                TlmFieldDef {
                    name: "delta".into(),
                    bit_size: 16,
                    ty: FieldType::Int,
                    states: vec![],
                    units: None,
                    description: String::new(),
                },
            ],
        }
    }

    #[test]
    fn decode_telemetry_values() {
        let def = make_tlm_def();
        // counter = 1000 (u32 BE), temperature = 25.5 f32 BE, delta = -10 (i16 BE)
        let counter: u32 = 1000;
        let temp: f32 = 25.5;
        let delta: i16 = -10;
        let mut payload = Vec::new();
        payload.extend_from_slice(&counter.to_be_bytes());
        payload.extend_from_slice(&temp.to_be_bytes());
        payload.extend_from_slice(&delta.to_be_bytes());

        let pkt = decode_telemetry(&payload, 7, 3, &def).unwrap();
        assert_eq!(pkt.sequence_count, 7);
        assert_eq!(pkt.fields[0], FieldValue::Uint(1000));
        assert_eq!(pkt.fields[1], FieldValue::Float(25.5));
        assert_eq!(pkt.fields[2], FieldValue::Int(-10));
    }

    #[test]
    fn decode_telemetry_truncated_returns_error() {
        let def = make_tlm_def();
        let result = decode_telemetry(&[0u8; 4], 0, 3, &def); // too short
        assert!(result.is_err());
    }

    // ── encode_command ───────────────────────────────────────────────

    fn make_cmd_def() -> CommandDef {
        CommandDef {
            target: "FSW".into(),
            name: "set_valve".into(),
            description: String::new(),
            command_id: 0xDEADBEEF,
            params: vec![CmdParamDef {
                name: "state".into(),
                bit_size: 8,
                ty: FieldType::Uint,
                default: 0,
                states: vec![],
                description: String::new(),
                required: true,
            }],
        }
    }

    #[test]
    fn encode_command_length() {
        let def = make_cmd_def();
        let pkt = CommandPacket {
            command_id: def.command_id,
            param_values: vec![1],
        };
        let bytes = encode_command(&pkt, &def, 0, 0);
        // 6 header + 4 cmd_id + 1 param = 11
        assert_eq!(bytes.len(), 11);
    }

    #[test]
    fn encode_command_header_fields() {
        let def = make_cmd_def();
        let pkt = CommandPacket {
            command_id: def.command_id,
            param_values: vec![0],
        };
        let bytes = encode_command(&pkt, &def, 5, 3);
        let h = CcsdsHeader::decode(&bytes).unwrap();
        assert_eq!(h.packet_type, 1); // command
        assert_eq!(h.apid, 5);
        assert_eq!(h.sequence_count, 3);
        assert!(h.secondary_header_flag);
    }

    #[test]
    fn encode_command_payload() {
        let def = make_cmd_def();
        let pkt = CommandPacket {
            command_id: 0xDEADBEEF,
            param_values: vec![0xAB],
        };
        let bytes = encode_command(&pkt, &def, 0, 0);
        // bytes[6..10] = CommandId big-endian
        assert_eq!(&bytes[6..10], &0xDEADBEEFu32.to_be_bytes());
        // bytes[10] = param value
        assert_eq!(bytes[10], 0xAB);
    }

    #[test]
    fn encode_command_data_length_field() {
        let def = make_cmd_def();
        let pkt = CommandPacket {
            command_id: def.command_id,
            param_values: vec![0],
        };
        let bytes = encode_command(&pkt, &def, 0, 0);
        let h = CcsdsHeader::decode(&bytes).unwrap();
        // data field = 4 (cmd_id) + 1 (param) = 5 bytes → data_length = 5 - 1 = 4
        assert_eq!(h.data_length, 4);
        assert_eq!(h.total_len(), bytes.len());
    }
}
