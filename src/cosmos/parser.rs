use thiserror::Error;

use super::types::{
    CmdParamDef, CommandDef, FieldType, StateEntry, StateValue, TelemetryPacketDef, TlmFieldDef,
};

// ── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("line {line}: invalid bit size '{token}': {source}")]
    InvalidBitSize {
        line: usize,
        token: String,
        source: std::num::ParseIntError,
    },
    #[error("line {line}: unknown field type '{token}'")]
    InvalidFieldType { line: usize, token: String },
    #[error("line {line}: too few tokens for '{keyword}' (got {got}, need at least {need})")]
    TooFewTokens {
        line: usize,
        keyword: String,
        got: usize,
        need: usize,
    },
    #[error("no TELEMETRY header found")]
    NoTelemetryHeader,
    #[error("line {line}: invalid state value '{token}'")]
    InvalidStateValue { line: usize, token: String },
    #[error("line {line}: invalid default value '{token}': {source}")]
    InvalidDefaultValue {
        line: usize,
        token: String,
        source: std::num::ParseIntError,
    },
}

// ── Tokeniser ────────────────────────────────────────────────────────────────

/// Split one logical line into tokens.
/// Respects single- and double-quoted strings.
/// Everything from an unquoted `#` onwards is a comment.
fn tokenize(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();

    loop {
        // Skip leading whitespace
        while chars.peek().is_some_and(|&c| c == ' ' || c == '\t') {
            chars.next();
        }

        match chars.peek() {
            None | Some('#') => break,
            Some(&q @ '\'' | &q @ '"') => {
                chars.next();
                let mut s = String::new();
                for c in chars.by_ref() {
                    if c == q {
                        break;
                    }
                    s.push(c);
                }
                tokens.push(s);
            }
            _ => {
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ' ' || c == '\t' || c == '#' {
                        break;
                    }
                    s.push(c);
                    chars.next();
                }
                if !s.is_empty() {
                    tokens.push(s);
                }
            }
        }
    }

    tokens
}

// ── Field type helper ────────────────────────────────────────────────────────

fn parse_field_type(token: &str, line: usize) -> Result<FieldType, ParseError> {
    match token {
        "UINT" => Ok(FieldType::Uint),
        "INT" => Ok(FieldType::Int),
        "FLOAT" => Ok(FieldType::Float),
        // STRING / BLOCK / DERIVED — treat as Uint for now (unlikely in our files)
        other => Err(ParseError::InvalidFieldType {
            line,
            token: other.to_string(),
        }),
    }
}

fn parse_bit_size(token: &str, line: usize) -> Result<u32, ParseError> {
    token
        .parse::<u32>()
        .map_err(|source| ParseError::InvalidBitSize {
            line,
            token: token.to_string(),
            source,
        })
}

fn parse_state_value(token: &str, line: usize) -> Result<StateValue, ParseError> {
    if token.eq_ignore_ascii_case("ANY") {
        return Ok(StateValue::Any);
    }
    // Might be a signed integer in disguise (COSMOS allows negative state values)
    if let Ok(v) = token.parse::<i64>() {
        return Ok(StateValue::Exact(v as u64));
    }
    token
        .parse::<u64>()
        .map(StateValue::Exact)
        .map_err(|_| ParseError::InvalidStateValue {
            line,
            token: token.to_string(),
        })
}

fn parse_default(token: &str, line: usize) -> Result<u64, ParseError> {
    // Defaults for UINT/FLOAT fields are stored as u64 raw bits
    if let Ok(v) = token.parse::<u64>() {
        return Ok(v);
    }
    if let Ok(v) = token.parse::<i64>() {
        return Ok(v as u64);
    }
    // float default — convert to bits
    if let Ok(v) = token.parse::<f64>() {
        return Ok(v.to_bits());
    }
    Err(ParseError::InvalidDefaultValue {
        line,
        token: token.to_string(),
        source: token.parse::<u64>().unwrap_err(),
    })
}

// ── Telemetry parser ─────────────────────────────────────────────────────────

/// Parse all TELEMETRY blocks in a COSMOS definition file.
///
/// Each `TELEMETRY` keyword starts a new packet. The
/// `<%= render "_ccsds_tlm.txt", locals: {apid: N} %>` template tag is
/// handled specially to extract the APID; all other `<%= ... %>` lines are skipped.
pub fn parse_telemetry_packets(input: &str) -> Result<Vec<TelemetryPacketDef>, ParseError> {
    let mut packets: Vec<TelemetryPacketDef> = Vec::new();
    let mut target = String::new();
    let mut name = String::new();
    let mut apid: u16 = 0;
    let mut fields: Vec<TlmFieldDef> = Vec::new();
    let mut current: Option<TlmFieldDef> = None;
    let mut in_packet = false;

    for (zero_idx, raw_line) in input.lines().enumerate() {
        let line_no = zero_idx + 1;
        let trimmed = raw_line.trim_start();

        // Skip blanks and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Handle COSMOS template tags
        if trimmed.starts_with("<%=") {
            // Extract `apid: N` if present
            if let Some(apid_val) = extract_apid_from_template(trimmed) {
                apid = apid_val;
            }
            continue;
        }

        let tokens = tokenize(trimmed);
        if tokens.is_empty() {
            continue;
        }

        match tokens[0].as_str() {
            "TELEMETRY" => {
                // Flush any in-progress packet
                if in_packet && !name.is_empty() {
                    push_current(&mut fields, &mut current);
                    packets.push(TelemetryPacketDef {
                        target: target.clone(),
                        name: name.clone(),
                        apid,
                        fields: std::mem::take(&mut fields),
                    });
                }
                if tokens.len() < 3 {
                    return Err(ParseError::TooFewTokens {
                        line: line_no,
                        keyword: "TELEMETRY".into(),
                        got: tokens.len(),
                        need: 3,
                    });
                }
                target = tokens[1].clone();
                name = tokens[2].clone();
                apid = 0;
                current = None;
                in_packet = true;
            }

            "APPEND_ITEM" => {
                // APPEND_ITEM <Name> <BitSize> <DataType> ["<Description>"]
                push_current(&mut fields, &mut current);
                if tokens.len() < 4 {
                    return Err(ParseError::TooFewTokens {
                        line: line_no,
                        keyword: "APPEND_ITEM".into(),
                        got: tokens.len(),
                        need: 4,
                    });
                }
                current = Some(TlmFieldDef {
                    name: tokens[1].clone(),
                    bit_size: parse_bit_size(&tokens[2], line_no)?,
                    ty: parse_field_type(&tokens[3], line_no)?,
                    states: Vec::new(),
                    units: None,
                    description: tokens.get(4).cloned().unwrap_or_default(),
                });
            }

            "APPEND_ID_ITEM" => {
                // APPEND_ID_ITEM <Name> <BitSize> <DataType> <IDValue> ["<Description>"]
                // (Used for the CCSDSAPID field in the template — skip it, just read the APID.)
                push_current(&mut fields, &mut current);
                if tokens.len() < 5 {
                    return Err(ParseError::TooFewTokens {
                        line: line_no,
                        keyword: "APPEND_ID_ITEM".into(),
                        got: tokens.len(),
                        need: 5,
                    });
                }
                let field_name = &tokens[1];
                if field_name.starts_with("CCSDS") {
                    if field_name == "CCSDSAPID" {
                        if let Ok(id) = tokens[4].parse::<u16>() {
                            apid = id;
                        }
                    }
                    // Do not add CCSDS header fields to the payload field list
                } else {
                    current = Some(TlmFieldDef {
                        name: tokens[1].clone(),
                        bit_size: parse_bit_size(&tokens[2], line_no)?,
                        ty: parse_field_type(&tokens[3], line_no)?,
                        states: Vec::new(),
                        units: None,
                        // description is at index 5 (after the IDValue at index 4)
                        description: tokens.get(5).cloned().unwrap_or_default(),
                    });
                }
            }

            "STATE" => {
                // STATE <Name> <Value> [<Color>]
                if tokens.len() < 3 {
                    return Err(ParseError::TooFewTokens {
                        line: line_no,
                        keyword: "STATE".into(),
                        got: tokens.len(),
                        need: 3,
                    });
                }
                if let Some(field) = current.as_mut() {
                    field.states.push(StateEntry {
                        name: tokens[1].clone(),
                        value: parse_state_value(&tokens[2], line_no)?,
                    });
                }
            }

            "UNITS" => {
                // UNITS <FullName> <Abbreviation>
                if let Some(field) = current.as_mut() {
                    // Use the abbreviation (second token) as the display unit
                    field.units = tokens.get(2).cloned().or_else(|| tokens.get(1).cloned());
                }
            }

            // Sub-keywords we intentionally ignore for telemetry
            "HIDDEN"
            | "OVERFLOW"
            | "FORMAT_STRING"
            | "DESCRIPTION"
            | "POLY_READ_CONVERSION"
            | "READ_CONVERSION"
            | "LIMITS"
            | "GENERIC_READ_CONVERSION_START"
            | "GENERIC_READ_CONVERSION_END"
            | "CONVERTED_DATA"
            | "META"
            | "REQUIRED" => {}

            _ => {
                // Unknown keyword — skip silently
            }
        }
    }

    // Flush the last in-progress packet
    if in_packet && !name.is_empty() {
        push_current(&mut fields, &mut current);
        packets.push(TelemetryPacketDef { target, name, apid, fields });
    }

    if packets.is_empty() {
        return Err(ParseError::NoTelemetryHeader);
    }

    Ok(packets)
}

/// Parse a single-packet COSMOS telemetry file (legacy wrapper).
pub fn parse_telemetry(input: &str) -> Result<TelemetryPacketDef, ParseError> {
    parse_telemetry_packets(input)?
        .into_iter()
        .next()
        .ok_or(ParseError::NoTelemetryHeader)
}

// ── Command parser ────────────────────────────────────────────────────────────

/// Parse a COSMOS command definition file, returning all commands found.
pub fn parse_commands(input: &str) -> Result<Vec<CommandDef>, ParseError> {
    let mut commands: Vec<CommandDef> = Vec::new();

    // Builder state for the command being assembled
    let mut cur_target = String::new();
    let mut cur_name = String::new();
    let mut cur_desc = String::new();
    let mut cur_cmd_id: u32 = 0;
    let mut cur_params: Vec<CmdParamDefBuilder> = Vec::new();
    let mut in_command = false;

    // Builder state for the parameter being assembled
    let mut cur_param: Option<CmdParamDefBuilder> = None;

    for (zero_idx, raw_line) in input.lines().enumerate() {
        let line_no = zero_idx + 1;
        let trimmed = raw_line.trim_start();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("<%=") {
            continue;
        }

        let tokens = tokenize(trimmed);
        if tokens.is_empty() {
            continue;
        }

        match tokens[0].as_str() {
            "COMMAND" => {
                // Flush the previous command (if any)
                flush_command(
                    &mut commands,
                    &mut cur_param,
                    &mut cur_params,
                    in_command,
                    &cur_target,
                    &cur_name,
                    &cur_desc,
                    cur_cmd_id,
                );

                if tokens.len() < 3 {
                    return Err(ParseError::TooFewTokens {
                        line: line_no,
                        keyword: "COMMAND".into(),
                        got: tokens.len(),
                        need: 3,
                    });
                }
                cur_target = tokens[1].clone();
                cur_name = tokens[2].clone();
                cur_desc = tokens.get(4).cloned().unwrap_or_default();
                cur_cmd_id = 0;
                cur_params = Vec::new();
                cur_param = None;
                in_command = true;
            }

            "APPEND_PARAMETER" | "APPEND_ID_PARAMETER" => {
                // APPEND_PARAMETER <Name> <BitSize> <DataType> <Min> <Max> <Default> ["<Desc>"]
                // (STRING/BLOCK have: <Name> <BitSize> <DataType> <Default> ["<Desc>"])
                if !in_command {
                    continue;
                }

                // Flush in-progress parameter
                if let Some(p) = cur_param.take() {
                    cur_params.push(p);
                }

                if tokens.len() < 4 {
                    return Err(ParseError::TooFewTokens {
                        line: line_no,
                        keyword: tokens[0].clone(),
                        got: tokens.len(),
                        need: 4,
                    });
                }

                let param_name = tokens[1].clone();
                let bit_size = parse_bit_size(&tokens[2], line_no)?;
                let ty_str = tokens[3].as_str();

                // For SecondaryHeader/CommandId, capture the command_id and mark hidden
                if param_name == "SecondaryHeader/CommandId" {
                    // Default value is at index 6 for numeric types
                    if let Some(default_tok) = tokens.get(6) {
                        if let Ok(id) = default_tok.parse::<u32>() {
                            cur_cmd_id = id;
                        }
                    }
                    // Mark as hidden by pushing with hidden=true
                    cur_param = Some(CmdParamDefBuilder {
                        name: param_name,
                        bit_size,
                        ty: FieldType::Uint,
                        default: 0,
                        states: Vec::new(),
                        description: String::new(),
                        required: false,
                        hidden: true,
                    });
                    continue;
                }

                // Parse type and default
                let (ty, default) = match ty_str {
                    "UINT" | "INT" | "FLOAT" | "DERIVED" => {
                        let ty = parse_field_type(ty_str, line_no).unwrap_or(FieldType::Uint);
                        // min=tokens[4], max=tokens[5], default=tokens[6]
                        let default = tokens
                            .get(6)
                            .map(|t| parse_default(t, line_no))
                            .transpose()?
                            .unwrap_or(0);
                        (ty, default)
                    }
                    "STRING" | "BLOCK" => {
                        // default=tokens[4]
                        let default = tokens
                            .get(4)
                            .map(|t| parse_default(t, line_no))
                            .transpose()?
                            .unwrap_or(0);
                        (FieldType::Uint, default)
                    }
                    _ => {
                        // Unknown type — treat as Uint
                        let default = tokens
                            .get(6)
                            .map(|t| parse_default(t, line_no))
                            .transpose()?
                            .unwrap_or(0);
                        (FieldType::Uint, default)
                    }
                };

                let description = match ty_str {
                    "STRING" | "BLOCK" => tokens.get(5).cloned().unwrap_or_default(),
                    _ => tokens.get(7).cloned().unwrap_or_default(),
                };

                // Detect APPEND_ID_PARAMETER: for those with "CCSDS" prefix treat as hidden
                let hidden = param_name.starts_with("CCSDS") || tokens[0] == "APPEND_ID_PARAMETER";

                cur_param = Some(CmdParamDefBuilder {
                    name: param_name,
                    bit_size,
                    ty,
                    default,
                    states: Vec::new(),
                    description,
                    required: false,
                    hidden,
                });
            }

            "HIDDEN" => {
                if let Some(p) = cur_param.as_mut() {
                    p.hidden = true;
                }
            }

            "REQUIRED" => {
                if let Some(p) = cur_param.as_mut() {
                    p.required = true;
                }
            }

            "STATE" => {
                if tokens.len() < 3 {
                    return Err(ParseError::TooFewTokens {
                        line: line_no,
                        keyword: "STATE".into(),
                        got: tokens.len(),
                        need: 3,
                    });
                }
                if let Some(p) = cur_param.as_mut() {
                    p.states.push(StateEntry {
                        name: tokens[1].clone(),
                        value: parse_state_value(&tokens[2], line_no)?,
                    });
                }
            }

            "UNITS" => {
                // No units on commands in our files, but handle gracefully
            }

            // Command-level keywords we don't need
            "HAZARDOUS"
            | "RESTRICTED"
            | "DISABLED"
            | "DISABLE_MESSAGES"
            | "HIDDEN_CMD"
            | "VIRTUAL"
            | "CATCHALL"
            | "META"
            | "VALIDATOR"
            | "RESPONSE"
            | "ERROR_RESPONSE"
            | "RELATED_ITEM"
            | "SCREEN"
            | "TEMPLATE"
            | "TEMPLATE_FILE"
            | "ACCESSOR"
            | "FORMAT_STRING"
            | "DESCRIPTION"
            | "OVERFLOW"
            | "POLY_WRITE_CONVERSION"
            | "WRITE_CONVERSION"
            | "GENERIC_WRITE_CONVERSION_START"
            | "GENERIC_WRITE_CONVERSION_END"
            | "MINIMUM_VALUE"
            | "MAXIMUM_VALUE"
            | "DEFAULT_VALUE"
            | "OBFUSCATE"
            | "OVERLAP"
            | "KEY"
            | "VARIABLE_BIT_SIZE" => {}

            _ => {}
        }
    }

    // Flush the last command
    flush_command(
        &mut commands,
        &mut cur_param,
        &mut cur_params,
        in_command,
        &cur_target,
        &cur_name,
        &cur_desc,
        cur_cmd_id,
    );

    Ok(commands)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

struct CmdParamDefBuilder {
    name: String,
    bit_size: u32,
    ty: FieldType,
    default: u64,
    states: Vec<StateEntry>,
    description: String,
    required: bool,
    hidden: bool,
}

impl CmdParamDefBuilder {
    fn build(self) -> Option<CmdParamDef> {
        if self.hidden {
            return None;
        }
        Some(CmdParamDef {
            name: self.name,
            bit_size: self.bit_size,
            ty: self.ty,
            default: self.default,
            states: self.states,
            description: self.description,
            required: self.required,
        })
    }
}

fn push_current(fields: &mut Vec<TlmFieldDef>, current: &mut Option<TlmFieldDef>) {
    if let Some(f) = current.take() {
        fields.push(f);
    }
}

#[allow(clippy::too_many_arguments)]
fn flush_command(
    commands: &mut Vec<CommandDef>,
    cur_param: &mut Option<CmdParamDefBuilder>,
    cur_params: &mut Vec<CmdParamDefBuilder>,
    in_command: bool,
    target: &str,
    name: &str,
    desc: &str,
    cmd_id: u32,
) {
    if !in_command || name.is_empty() {
        return;
    }
    if let Some(p) = cur_param.take() {
        cur_params.push(p);
    }
    let params = cur_params.drain(..).filter_map(|b| b.build()).collect();
    commands.push(CommandDef {
        target: target.to_string(),
        name: name.to_string(),
        description: desc.to_string(),
        command_id: cmd_id,
        params,
    });
}

/// Extract the APID from a COSMOS template tag such as:
/// `<%= render "_ccsds_tlm.txt", locals: {apid: 3} %>`
fn extract_apid_from_template(line: &str) -> Option<u16> {
    // Look for `apid:` followed by whitespace and a number
    let idx = line.find("apid:")?;
    let after = line[idx + 5..].trim_start();
    let end = after
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after.len());
    after[..end].parse().ok()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── tokenize ────────────────────────────────────────────────────

    #[test]
    fn tokenize_basic() {
        let t = tokenize("TELEMETRY FOO BAR BIG_ENDIAN");
        assert_eq!(t, ["TELEMETRY", "FOO", "BAR", "BIG_ENDIAN"]);
    }

    #[test]
    fn tokenize_single_quoted() {
        let t = tokenize("APPEND_ITEM foo 32 UINT 'a description here'");
        assert_eq!(
            t,
            ["APPEND_ITEM", "foo", "32", "UINT", "a description here"]
        );
    }

    #[test]
    fn tokenize_double_quoted() {
        let t = tokenize(r#"APPEND_PARAMETER bar 8 UINT 0 255 0 "CCSDS version""#);
        assert_eq!(
            t,
            [
                "APPEND_PARAMETER",
                "bar",
                "8",
                "UINT",
                "0",
                "255",
                "0",
                "CCSDS version"
            ]
        );
    }

    #[test]
    fn tokenize_strips_comment() {
        let t = tokenize("STATE FOO 1 # this is a comment");
        assert_eq!(t, ["STATE", "FOO", "1"]);
    }

    #[test]
    fn tokenize_empty_line() {
        assert!(tokenize("   ").is_empty());
        assert!(tokenize("").is_empty());
    }

    // ── extract_apid_from_template ───────────────────────────────────

    #[test]
    fn apid_extraction() {
        assert_eq!(
            extract_apid_from_template(r#"<%= render "_ccsds_tlm.txt", locals: {apid: 3} %>"#),
            Some(3)
        );
        assert_eq!(extract_apid_from_template(r#"<%= render "other" %>"#), None);
        assert_eq!(
            extract_apid_from_template(r#"<%= render "_ccsds_tlm.txt", locals: {apid: 42} %>"#),
            Some(42)
        );
    }

    // ── parse_telemetry ──────────────────────────────────────────────

    const MINIMAL_TLM: &str = r#"
# comment
TELEMETRY TARGET PKT BIG_ENDIAN "A packet"
<%= render "_ccsds_tlm.txt", locals: {apid: 7} %>
APPEND_ITEM pressure 32 FLOAT 'Pressure reading'
    UNITS 'pascals' 'Pa'
APPEND_ITEM valve_state 8 UINT 'Valve state'
    STATE CLOSED 0
    STATE OPEN 1
    STATE ERROR ANY
APPEND_ITEM timestamp 64 INT 'Unix timestamp'
    UNITS 'nanoseconds' 'ns'
"#;

    #[test]
    fn parse_telemetry_header() {
        let def = parse_telemetry(MINIMAL_TLM).unwrap();
        assert_eq!(def.target, "TARGET");
        assert_eq!(def.name, "PKT");
        assert_eq!(def.apid, 7);
    }

    #[test]
    fn parse_telemetry_field_count() {
        let def = parse_telemetry(MINIMAL_TLM).unwrap();
        assert_eq!(def.fields.len(), 3);
    }

    #[test]
    fn parse_telemetry_float_field() {
        let def = parse_telemetry(MINIMAL_TLM).unwrap();
        let f = &def.fields[0];
        assert_eq!(f.name, "pressure");
        assert_eq!(f.bit_size, 32);
        assert_eq!(f.ty, FieldType::Float);
        assert_eq!(f.units.as_deref(), Some("Pa"));
        assert!(f.states.is_empty());
    }

    #[test]
    fn parse_telemetry_uint_with_states() {
        let def = parse_telemetry(MINIMAL_TLM).unwrap();
        let f = &def.fields[1];
        assert_eq!(f.name, "valve_state");
        assert_eq!(f.ty, FieldType::Uint);
        assert_eq!(f.states.len(), 3);
        assert_eq!(
            f.states[0],
            StateEntry {
                name: "CLOSED".into(),
                value: StateValue::Exact(0)
            }
        );
        assert_eq!(
            f.states[1],
            StateEntry {
                name: "OPEN".into(),
                value: StateValue::Exact(1)
            }
        );
        assert_eq!(
            f.states[2],
            StateEntry {
                name: "ERROR".into(),
                value: StateValue::Any
            }
        );
    }

    #[test]
    fn parse_telemetry_int_field() {
        let def = parse_telemetry(MINIMAL_TLM).unwrap();
        let f = &def.fields[2];
        assert_eq!(f.ty, FieldType::Int);
        assert_eq!(f.bit_size, 64);
        assert_eq!(f.units.as_deref(), Some("ns"));
    }

    #[test]
    fn parse_telemetry_payload_len() {
        let def = parse_telemetry(MINIMAL_TLM).unwrap();
        // 4 (float) + 1 (uint8) + 8 (int64) = 13 bytes
        assert_eq!(def.payload_byte_len(), 13);
    }

    // ── parse_commands ───────────────────────────────────────────────

    const MINIMAL_CMD: &str = r#"
COMMAND FSW arm_rocket BIG_ENDIAN "Arms the rocket"
    APPEND_PARAMETER CCSDSVER                  3  UINT  0  10  0  "CCSDS version"
    HIDDEN
    APPEND_PARAMETER CCSDSTYPE                 1  UINT  0  1   1  "Type"
    HIDDEN
    APPEND_PARAMETER CCSDSSHF                  1  UINT  0  1   1  "SHF"
    HIDDEN
    APPEND_PARAMETER CCSDSAPID                11  UINT  0  2047 0 "APID"
    HIDDEN
    APPEND_PARAMETER CCSDSSEQFLAGS             2  UINT  0  3   0  "Seq flags"
    HIDDEN
    APPEND_PARAMETER CCSDSSEQCNT              14  UINT  0  0   0  "Seq count"
    HIDDEN
    APPEND_PARAMETER CCSDSLENGTH              16  UINT  0  1000 32 "Length"
    HIDDEN
    APPEND_PARAMETER SecondaryHeader/CommandId 32 UINT 0 4294967295 12345678 "Command ID"
    HIDDEN
    APPEND_PARAMETER arm_state                 8  UINT  0  1   0  "Arm state"
    REQUIRED
    STATE disarmed 0
    STATE armed    1

COMMAND FSW open_valve BIG_ENDIAN "Opens the main valve"
    APPEND_PARAMETER CCSDSVER                  3  UINT  0  10  0  "CCSDS version"
    HIDDEN
    APPEND_PARAMETER CCSDSTYPE                 1  UINT  0  1   1  "Type"
    HIDDEN
    APPEND_PARAMETER CCSDSSHF                  1  UINT  0  1   1  "SHF"
    HIDDEN
    APPEND_PARAMETER CCSDSAPID                11  UINT  0  2047 0 "APID"
    HIDDEN
    APPEND_PARAMETER CCSDSSEQFLAGS             2  UINT  0  3   0  "Seq flags"
    HIDDEN
    APPEND_PARAMETER CCSDSSEQCNT              14  UINT  0  0   0  "Seq count"
    HIDDEN
    APPEND_PARAMETER CCSDSLENGTH              16  UINT  0  1000 32 "Length"
    HIDDEN
    APPEND_PARAMETER SecondaryHeader/CommandId 32 UINT 0 4294967295 87654321 "Command ID"
    HIDDEN
    APPEND_PARAMETER valve_action              8  UINT  0  5   0  ""
    REQUIRED
    STATE open  0
    STATE close 1
"#;

    #[test]
    fn parse_commands_count() {
        let cmds = parse_commands(MINIMAL_CMD).unwrap();
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn parse_commands_ids() {
        let cmds = parse_commands(MINIMAL_CMD).unwrap();
        assert_eq!(cmds[0].command_id, 12345678);
        assert_eq!(cmds[1].command_id, 87654321);
    }

    #[test]
    fn parse_commands_names() {
        let cmds = parse_commands(MINIMAL_CMD).unwrap();
        assert_eq!(cmds[0].name, "arm_rocket");
        assert_eq!(cmds[1].name, "open_valve");
    }

    #[test]
    fn parse_commands_hidden_params_excluded() {
        let cmds = parse_commands(MINIMAL_CMD).unwrap();
        // Only the non-hidden user params should survive
        assert_eq!(cmds[0].params.len(), 1);
        assert_eq!(cmds[0].params[0].name, "arm_state");
        assert_eq!(cmds[1].params.len(), 1);
        assert_eq!(cmds[1].params[0].name, "valve_action");
    }

    #[test]
    fn parse_commands_param_states() {
        let cmds = parse_commands(MINIMAL_CMD).unwrap();
        let p = &cmds[0].params[0];
        assert_eq!(p.states.len(), 2);
        assert_eq!(
            p.states[0],
            StateEntry {
                name: "disarmed".into(),
                value: StateValue::Exact(0)
            }
        );
        assert_eq!(
            p.states[1],
            StateEntry {
                name: "armed".into(),
                value: StateValue::Exact(1)
            }
        );
        assert!(p.required);
    }

    #[test]
    fn parse_commands_default_values() {
        let cmds = parse_commands(MINIMAL_CMD).unwrap();
        assert_eq!(cmds[0].params[0].default, 0);
    }
}

// ── Integration tests against the real COSMOS files ──────────────

#[test]
fn parse_real_telemetry_file() {
    let input = std::fs::read_to_string("cosmos/fsw_tlm.txt")
        .expect("real telemetry file not found");

    let packets = parse_telemetry_packets(&input).expect("parse failed");
    assert!(!packets.is_empty(), "expected at least one packet");
    let total_fields: usize = packets.iter().map(|p| p.fields.len()).sum();
    assert!(total_fields > 0, "expected fields, got 0");
    // All APIDs should be non-zero and unique
    let mut apids: Vec<u16> = packets.iter().map(|p| p.apid).collect();
    apids.sort_unstable();
    apids.dedup();
    assert_eq!(apids.len(), packets.len(), "duplicate APIDs found");
}

#[test]
fn parse_real_command_file() {
    let input = std::fs::read_to_string("cosmos/fsw_cmd.txt")
        .expect("real command file not found");

    let cmds = parse_commands(&input).expect("parse failed");
    assert!(!cmds.is_empty(), "expected commands, got 0");
    // Every command must have a non-zero command_id
    for cmd in &cmds {
        assert_ne!(
            cmd.command_id, 0,
            "command '{}' has zero command_id",
            cmd.name
        );
    }
    // Every command has at most 3 user params (based on our earlier analysis: 1–3)
    for cmd in &cmds {
        assert!(
            cmd.params.len() <= 3,
            "command '{}' has {} params (unexpectedly many)",
            cmd.name,
            cmd.params.len()
        );
    }
}
