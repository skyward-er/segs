mod fields;

pub use fields::IndexedField;

use crate::mavlink::MAVLINK_PROFILE;

/// Returns all telemetry fields across all packets as `IndexedField` references.
pub fn all_fields() -> Vec<IndexedField> {
    let Some(registry) = MAVLINK_PROFILE.get() else {
        return Vec::new();
    };
    registry
        .telemetry
        .iter()
        .flat_map(|pkt| {
            pkt.fields
                .iter()
                .enumerate()
                .map(|(index, field)| IndexedField::new(pkt.apid, index, field, &pkt.name))
        })
        .collect()
}

/// Returns only fields that can be plotted (numeric types), across all packets.
pub fn plottable_fields() -> Vec<IndexedField> {
    use crate::cosmos::FieldType;
    let Some(registry) = MAVLINK_PROFILE.get() else {
        return Vec::new();
    };
    registry
        .telemetry
        .iter()
        .flat_map(|pkt| {
            pkt.fields
                .iter()
                .enumerate()
                .filter(|(_, f)| matches!(f.ty, FieldType::Uint | FieldType::Int | FieldType::Float))
                .map(|(index, field)| IndexedField::new(pkt.apid, index, field, &pkt.name))
        })
        .collect()
}
