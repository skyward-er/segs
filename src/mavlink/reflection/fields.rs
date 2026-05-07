use serde::ser::SerializeStruct;

use crate::{
    ccsds::{FieldValue, TelemetryPacket},
    cosmos::TlmFieldDef,
    mavlink::MAVLINK_PROFILE,
};

/// A reference to a single field in a telemetry packet, identified by APID and index.
#[derive(Debug, Clone)]
pub struct IndexedField {
    pub apid: u16,
    pub index: usize,
    pub field: &'static TlmFieldDef,
    display_name: String,
}

impl IndexedField {
    pub fn new(apid: u16, index: usize, field: &'static TlmFieldDef, packet_name: &str) -> Self {
        let display_name = format!("{}/{}", packet_name, field.name);
        Self { apid, index, field, display_name }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn field(&self) -> &TlmFieldDef {
        self.field
    }

    pub fn name(&self) -> &str {
        &self.display_name
    }

    pub fn extract_as_f64(&self, packet: &TelemetryPacket) -> Result<f64, String> {
        if packet.apid != self.apid {
            return Err(format!("packet APID {} != field APID {}", packet.apid, self.apid));
        }
        match packet.fields.get(self.index) {
            Some(FieldValue::Uint(v)) => Ok(*v as f64),
            Some(FieldValue::Int(v)) => Ok(*v as f64),
            Some(FieldValue::Float(v)) => Ok(*v),
            None => Err(format!("field index {} out of range", self.index)),
        }
    }

    pub fn extract_as_u64(&self, packet: &TelemetryPacket) -> Result<u64, String> {
        if packet.apid != self.apid {
            return Err(format!("packet APID {} != field APID {}", packet.apid, self.apid));
        }
        match packet.fields.get(self.index) {
            Some(FieldValue::Uint(v)) => Ok(*v),
            Some(FieldValue::Int(v)) => Ok(*v as u64),
            Some(FieldValue::Float(v)) => Ok(*v as u64),
            None => Err(format!("field index {} out of range", self.index)),
        }
    }

    pub fn extract_as_string(&self, packet: &TelemetryPacket) -> String {
        if packet.apid != self.apid {
            return "N/A".to_string();
        }
        match packet.fields.get(self.index) {
            Some(FieldValue::Uint(v)) => self
                .field
                .state_name(*v)
                .map(|s| s.to_string())
                .unwrap_or_else(|| v.to_string()),
            Some(FieldValue::Int(v)) => self
                .field
                .state_name(*v as u64)
                .map(|s| s.to_string())
                .unwrap_or_else(|| v.to_string()),
            Some(FieldValue::Float(v)) => format!("{v:.5}"),
            None => "N/A".to_string(),
        }
    }
}

impl std::hash::Hash for IndexedField {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.apid.hash(state);
        self.index.hash(state);
    }
}

impl PartialEq for IndexedField {
    fn eq(&self, other: &Self) -> bool {
        self.apid == other.apid && self.index == other.index
    }
}

impl serde::Serialize for IndexedField {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("IndexedField", 2)?;
        state.serialize_field("apid", &self.apid)?;
        state.serialize_field("index", &self.index)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for IndexedField {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct IndexedFieldDe {
            apid: u16,
            index: usize,
        }

        let de = IndexedFieldDe::deserialize(deserializer)?;
        let registry = MAVLINK_PROFILE
            .get()
            .ok_or_else(|| serde::de::Error::custom("PacketRegistry not initialized"))?;
        let pkt = registry
            .get_telemetry_by_apid(de.apid)
            .ok_or_else(|| serde::de::Error::custom(format!("no packet for APID {}", de.apid)))?;
        let field = pkt.fields.get(de.index).ok_or_else(|| {
            serde::de::Error::custom(format!("field index {} out of range", de.index))
        })?;
        Ok(IndexedField::new(de.apid, de.index, field, &pkt.name))
    }
}
