/// Field data type as expressed in COSMOS definitions.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Uint,
    Int,
    Float,
}

/// Value of a STATE entry for a field or parameter.
#[derive(Debug, Clone, PartialEq)]
pub enum StateValue {
    /// Exact numeric value.
    Exact(u64),
    /// Catch-all — matches any value not covered by other states.
    Any,
}

/// A named state (enum variant) for a field or parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct StateEntry {
    pub name: String,
    pub value: StateValue,
}

/// Definition of one field inside a telemetry packet.
#[derive(Debug, Clone)]
pub struct TlmFieldDef {
    pub name: String,
    pub bit_size: u32,
    pub ty: FieldType,
    /// Named states (empty if the field has no enum mapping).
    pub states: Vec<StateEntry>,
    /// Short unit abbreviation (e.g. "bar", "ns"), if any.
    pub units: Option<String>,
    pub description: String,
}

impl TlmFieldDef {
    /// Returns true if the field carries a named state mapping.
    pub fn has_states(&self) -> bool {
        !self.states.is_empty()
    }

    /// Returns true if the field holds a numeric value suitable for plotting.
    pub fn is_plottable(&self) -> bool {
        matches!(self.ty, FieldType::Uint | FieldType::Int | FieldType::Float)
    }

    /// Byte count of the field in the wire payload.
    pub fn byte_size(&self) -> usize {
        self.bit_size as usize / 8
    }

    /// Look up the state name for a raw value.
    pub fn state_name(&self, raw: u64) -> Option<&str> {
        self.states
            .iter()
            .find(|s| s.value == StateValue::Exact(raw))
            .or_else(|| self.states.iter().find(|s| s.value == StateValue::Any))
            .map(|s| s.name.as_str())
    }
}

/// Definition of a complete telemetry packet (one COSMOS TELEMETRY block).
#[derive(Debug, Clone)]
pub struct TelemetryPacketDef {
    pub target: String,
    pub name: String,
    pub apid: u16,
    pub fields: Vec<TlmFieldDef>,
}

impl TelemetryPacketDef {
    /// Expected byte length of the payload (everything after the 6-byte CCSDS primary header).
    pub fn payload_byte_len(&self) -> usize {
        self.fields.iter().map(|f| f.byte_size()).sum()
    }
}

/// One non-hidden user parameter of a command.
#[derive(Debug, Clone)]
pub struct CmdParamDef {
    pub name: String,
    pub bit_size: u32,
    pub ty: FieldType,
    /// Default value from the COSMOS definition.
    pub default: u64,
    /// Named states (empty when the parameter is a raw number).
    pub states: Vec<StateEntry>,
    pub description: String,
    pub required: bool,
}

impl CmdParamDef {
    pub fn byte_size(&self) -> usize {
        self.bit_size as usize / 8
    }
}

/// Definition of one telecommand (one COSMOS COMMAND block).
#[derive(Debug, Clone)]
pub struct CommandDef {
    pub target: String,
    pub name: String,
    pub description: String,
    /// 32-bit command identifier placed in the CCSDS secondary header.
    pub command_id: u32,
    /// Non-hidden, non-CCSDS parameters visible to the operator.
    pub params: Vec<CmdParamDef>,
}

impl CommandDef {
    /// Returns default parameter values in definition order.
    pub fn default_param_values(&self) -> Vec<u64> {
        self.params.iter().map(|p| p.default).collect()
    }
}
