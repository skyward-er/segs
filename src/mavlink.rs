pub mod reflection;

use std::{path::Path, sync::OnceLock, time::Instant};

use crate::{
    ccsds::TelemetryPacket,
    cosmos::{CommandDef, ParseError, TelemetryPacketDef, parse_commands, parse_telemetry_packets},
};

pub use crate::ccsds::CommandPacket;

/// Global packet registry — initialised once at startup from the COSMOS definition files.
pub static MAVLINK_PROFILE: OnceLock<PacketRegistry> = OnceLock::new();

pub struct PacketRegistry {
    pub telemetry: Vec<TelemetryPacketDef>,
    pub commands: Vec<CommandDef>,
}

impl PacketRegistry {
    pub fn get_command_by_name(&self, name: &str) -> Option<&CommandDef> {
        self.commands.iter().find(|c| c.name == name)
    }

    pub fn get_telemetry_by_apid(&self, apid: u16) -> Option<&TelemetryPacketDef> {
        self.telemetry.iter().find(|p| p.apid == apid)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("IO error reading '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("COSMOS parse error: {0}")]
    Parse(#[from] ParseError),
}

/// Initialise the global packet registry from COSMOS definition files.
/// Must be called once before any reflection or communication code runs.
pub fn init_packet_registry(tlm_path: &Path, cmd_path: &Path) -> Result<(), RegistryError> {
    let tlm_text = std::fs::read_to_string(tlm_path).map_err(|source| RegistryError::Io {
        path: tlm_path.display().to_string(),
        source,
    })?;
    let cmd_text = std::fs::read_to_string(cmd_path).map_err(|source| RegistryError::Io {
        path: cmd_path.display().to_string(),
        source,
    })?;
    let telemetry = parse_telemetry_packets(&tlm_text)?;
    let commands = parse_commands(&cmd_text)?;
    MAVLINK_PROFILE
        .set(PacketRegistry { telemetry, commands })
        .ok();
    Ok(())
}

/// A decoded telemetry packet tagged with its reception time.
#[derive(Debug, Clone)]
pub struct TimedMessage {
    pub packet: TelemetryPacket,
    pub time: Instant,
}

impl TimedMessage {
    pub fn just_received(packet: TelemetryPacket) -> Self {
        Self {
            packet,
            time: Instant::now(),
        }
    }

    /// Decode a raw CCSDS byte slice into a `TimedMessage`, stamped now.
    pub fn from_ccsds_bytes(buf: &[u8]) -> Result<Self, String> {
        use crate::ccsds::{CcsdsHeader, decode_telemetry};

        let registry = MAVLINK_PROFILE
            .get()
            .ok_or("PacketRegistry not initialized")?;
        let header = CcsdsHeader::decode(buf).map_err(|e| e.to_string())?;
        let pkt_def = registry
            .get_telemetry_by_apid(header.apid)
            .ok_or_else(|| format!("no telemetry definition for APID {}", header.apid))?;
        let payload = buf.get(6..).ok_or("buffer too short for payload")?;
        let packet = decode_telemetry(payload, header.sequence_count, header.apid, pkt_def)
            .map_err(|e| e.to_string())?;
        Ok(Self::just_received(packet))
    }
}
