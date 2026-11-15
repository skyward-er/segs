//! Conversion of P&ID files exported by SEGS v1 into the v2 diagram model.
//!
//! Only the geometry is imported: symbols, positions, rotations, sizes, label texts and pipes
//! with their bend points. v1 bound symbols to MAVLink message fields, which have no automatic
//! v2 equivalent, so motor valves and value displays arrive unbound.

use std::{f32::consts::FRAC_PI_2, path::Path};

use ahash::{HashMap, HashMapExt};
use egui::{Color32, Pos2, pos2, vec2};
use serde::{Deserialize, de::IgnoredAny};

use super::{AnchorRef, IconKind, MotorValveKind, PidWidget, Rotation, Symbol};

/// v1 value displays always showed this many decimals.
const V1_DECIMALS: u8 = 5;

/// Summary of an import, shown to the user.
pub(super) struct ImportReport {
    pub(super) elements: usize,
    pub(super) connections: usize,
    /// Pipes dropped because an end referred to a missing element or anchor.
    pub(super) skipped_connections: usize,
    /// Motor valves and value displays that need a stream bound again.
    pub(super) unbound: usize,
}

/// Reads a v1 P&ID JSON file and appends its contents to `diagram`, to the right of anything
/// already there.
///
/// Accepts both the file written by the v1 P&ID export and a P&ID pane copied out of a v1
/// layout, since both carry `elements` and `connections`. Returns a summary of what was added,
/// or a readable error when the file cannot be read or is not a v1 P&ID; on error `diagram`
/// is unchanged.
pub(super) fn import(path: &Path, diagram: &mut PidWidget) -> Result<ImportReport, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("Cannot read {}: {error}", path.display()))?;
    let v1: V1Diagram = serde_json::from_str(&text).map_err(|error| format!("Not a v1 P&ID file: {error}"))?;

    // Convert every element first, in id order so repeated imports give the same result
    let mut ids: Vec<_> = v1.elements.keys().copied().collect();
    ids.sort_unstable();
    let converted: Vec<_> = ids.iter().map(|id| (*id, convert_element(&v1.elements[id]))).collect();

    // Build the import on its own, remembering which new id each v1 id became
    let mut imported = PidWidget::default();
    let mut unbound = 0;
    let mut new_ids = HashMap::with_capacity(converted.len());
    for (v1_id, (center, rotation, symbol)) in converted {
        if matches!(symbol, Symbol::MotorValve { .. } | Symbol::ValueDisplay { .. }) {
            unbound += 1;
        }
        let id = imported.add_rotated_element(center, rotation, symbol);
        new_ids.insert(v1_id, id);
    }

    // Recreate the pipes, dropping those whose ends do not exist in the imported elements
    let mut connections = 0;
    let mut skipped_connections = 0;
    for connection in &v1.connections {
        let ends = new_ids.get(&connection.start).zip(new_ids.get(&connection.end));
        let connected = ends.is_some_and(|(&start, &end)| {
            imported.connect(
                AnchorRef {
                    element: start,
                    anchor: connection.start_anchor,
                },
                AnchorRef {
                    element: end,
                    anchor: connection.end_anchor,
                },
            )
        });
        if !connected {
            skipped_connections += 1;
            continue;
        }
        if let Some(pipe) = imported.connections.last_mut() {
            pipe.waypoints = connection.points_g.iter().map(|&[x, y]| pos2(x, y)).collect();
        }
        connections += 1;
    }

    // Place it next to whatever the diagram already holds
    let elements = new_ids.len();
    diagram.append(imported);

    Ok(ImportReport {
        elements,
        connections,
        skipped_connections,
        unbound,
    })
}

/// Returns the v2 center, rotation and symbol of a v1 element.
///
/// v1 stores the rotated top-left corner and a rotation in radians that only ever changed in
/// quarter turns, so the rotation is rounded to the nearest quarter turn.
fn convert_element(element: &V1Element) -> (Pos2, Rotation, Symbol) {
    let quarter_turns = (element.rotation / FRAC_PI_2).round() as i32;
    let rotation = match quarter_turns.rem_euclid(4) {
        0 => Rotation::Deg0,
        1 => Rotation::Deg90,
        2 => Rotation::Deg180,
        _ => Rotation::Deg270,
    };
    let symbol = convert_symbol(&element.symbol);
    let [x, y] = element.position;
    let center = pos2(x, y) + rotation.apply(symbol.size() / 2.0);
    (center, rotation, symbol)
}

/// Returns the v2 symbol for a v1 symbol, without its stream binding.
fn convert_symbol(symbol: &V1Symbol) -> Symbol {
    let icon = |kind| Symbol::Icon(kind);
    match symbol {
        V1Symbol::Icon(v1_icon) => match v1_icon {
            V1Icon::Arrow => icon(IconKind::Arrow),
            V1Icon::BurstDisk => icon(IconKind::BurstDisk),
            V1Icon::CheckValve => icon(IconKind::CheckValve),
            V1Icon::FlexibleConnection => icon(IconKind::FlexibleConnection),
            V1Icon::ManualValve => icon(IconKind::ManualValve),
            V1Icon::PressureGauge => icon(IconKind::PressureGauge),
            V1Icon::PressureRegulator => icon(IconKind::PressureRegulator),
            V1Icon::PressureTransducer => icon(IconKind::PressureTransducer),
            V1Icon::QuickConnector => icon(IconKind::QuickConnector),
            V1Icon::ReliefValve => icon(IconKind::ReliefValve),
            V1Icon::Vessel => icon(IconKind::Vessel),
            V1Icon::MotorValve(valve) => Symbol::MotorValve {
                kind: match valve.variant {
                    V1MotorValveVariant::TwoWay(_) => MotorValveKind::TwoWay,
                    V1MotorValveVariant::ThreeWay(V1ThreeWay { invert }) => MotorValveKind::ThreeWay { invert },
                },
                stream: None,
            },
        },
        V1Symbol::ValueDisplay(display) => Symbol::ValueDisplay {
            stream: None,
            unit: String::new(),
            size: vec2(display.size[0], display.size[1]),
            color: match display.color {
                V1DisplayColor::Default => None,
                V1DisplayColor::Custom([r, g, b, a]) => Some(Color32::from_rgba_premultiplied(r, g, b, a)),
            },
            decimals: Some(V1_DECIMALS),
        },
        V1Symbol::Label(label) => Symbol::Label {
            text: label.text.clone(),
            size: vec2(label.size[0], label.size[1]),
        },
    }
}

/// The parts of a v1 P&ID needed for the import; other fields are ignored.
#[derive(Deserialize)]
struct V1Diagram {
    /// Elements keyed by their v1 id, which connections refer to.
    elements: HashMap<u32, V1Element>,
    connections: Vec<V1Connection>,
}

#[derive(Deserialize)]
struct V1Element {
    /// Rotated top-left corner in grid units.
    position: [f32; 2],
    symbol: V1Symbol,
    /// Rotation in radians.
    rotation: f32,
}

#[derive(Deserialize)]
enum V1Symbol {
    Icon(V1Icon),
    ValueDisplay(V1ValueDisplay),
    Label(V1Label),
}

#[derive(Deserialize)]
enum V1Icon {
    Arrow,
    BurstDisk,
    CheckValve,
    FlexibleConnection,
    ManualValve,
    MotorValve(V1MotorValve),
    PressureGauge,
    PressureRegulator,
    PressureTransducer,
    QuickConnector,
    ReliefValve,
    Vessel,
}

#[derive(Deserialize)]
struct V1MotorValve {
    variant: V1MotorValveVariant,
}

#[derive(Deserialize)]
enum V1MotorValveVariant {
    TwoWay(IgnoredAny),
    ThreeWay(V1ThreeWay),
}

#[derive(Deserialize)]
struct V1ThreeWay {
    #[serde(default)]
    invert: bool,
}

#[derive(Deserialize)]
struct V1ValueDisplay {
    size: [f32; 2],
    #[serde(default)]
    color: V1DisplayColor,
}

#[derive(Deserialize, Default)]
enum V1DisplayColor {
    /// v1 picked a color from the field's unit, which v2 does not know.
    #[default]
    Default,
    Custom([u8; 4]),
}

#[derive(Deserialize)]
struct V1Label {
    text: String,
    size: [f32; 2],
}

#[derive(Deserialize)]
struct V1Connection {
    start: u32,
    start_anchor: usize,
    end: u32,
    end_anchor: usize,
    /// Bend points in grid units.
    #[serde(default)]
    points_g: Vec<[f32; 2]>,
}
