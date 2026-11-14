mod editor;
mod paint;

pub use editor::PidEditor;

use egui::{Pos2, Rect, Ui, Vec2, pos2, vec2};
use serde::{Deserialize, Serialize};

use crate::{
    dataflow::{DataValue, StreamKey, preview::PreviewContext, store::DataStore},
    ui::{widget_settings::WidgetDataSetting, widgets::WidgetTrait},
};

/// Piping and instrumentation diagram showing live valve states and values.
///
/// Stores only the diagram definition. Live state is derived from the [`DataStore`] on every frame.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PidWidget {
    /// Diagram elements in insertion order, which is also the paint order.
    elements: Vec<Element>,
    connections: Vec<Connection>,
    next_element_id: u32,
}

impl WidgetTrait for PidWidget {
    fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        paint::show(self, ui, data_store);
    }

    /// Replaces the empty gallery diagram with a small example bound to the sample streams.
    fn configure_preview(&mut self, preview: &PreviewContext) {
        if self.elements.is_empty() {
            *self = Self::example(preview.integer_stream, preview.numeric_stream);
        }
    }

    /// Streams are bound per symbol inside the diagram editor.
    fn data_settings(&mut self) -> Vec<WidgetDataSetting<'_>> {
        Vec::new()
    }

    fn display_name(&self) -> &'static str {
        "P&ID"
    }

    fn min_size(&self) -> Vec2 {
        Vec2::new(2.0, 2.0)
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(6.0, 4.0)
    }
}

impl PidWidget {
    /// Adds a new element to the diagram.
    ///
    /// Returns the identifier of the new element, unique within this diagram.
    pub fn add_element(&mut self, pos: Pos2, symbol: Symbol) -> ElementId {
        let id = ElementId(self.next_element_id);
        self.next_element_id += 1;
        self.elements.push(Element {
            id,
            pos,
            rotation: Rotation::default(),
            symbol,
        });
        id
    }

    /// Removes an element and every connection attached to it.
    pub fn remove_element(&mut self, id: ElementId) {
        self.elements.retain(|element| element.id != id);
        self.connections
            .retain(|connection| connection.start.element != id && connection.end.element != id);
    }

    /// Replaces an element's symbol and removes connections to anchors the new symbol lacks.
    ///
    /// Always use this instead of assigning `symbol` directly, so no connection is
    /// left pointing past the end of the anchor list. Unknown ids are ignored.
    pub fn set_symbol(&mut self, id: ElementId, symbol: Symbol) {
        let Some(element) = self.elements.iter_mut().find(|element| element.id == id) else {
            return;
        };
        let anchor_count = symbol.anchors().len();
        element.symbol = symbol;

        // Keep a connection only if its ends on this element are still valid anchors
        self.connections.retain(|connection| {
            [connection.start, connection.end]
                .iter()
                .all(|end| end.element != id || end.anchor < anchor_count)
        });
    }

    /// Connects two element anchors with a straight pipe.
    ///
    /// Returns `false`, leaving the diagram unchanged, when either end names a
    /// missing element or an anchor index the element's symbol does not have.
    pub fn connect(&mut self, start: AnchorRef, end: AnchorRef) -> bool {
        let valid = |anchor: AnchorRef| {
            self.element(anchor.element)
                .is_some_and(|element| anchor.anchor < element.symbol.anchors().len())
        };
        if !valid(start) || !valid(end) {
            return false;
        }
        self.connections.push(Connection {
            start,
            end,
            waypoints: Vec::new(),
        });
        true
    }

    /// Returns the element with the given id, or `None` if it is not in the diagram.
    fn element(&self, id: ElementId) -> Option<&Element> {
        self.elements.iter().find(|element| element.id == id)
    }

    /// Returns the diagram-space position of a connection end.
    ///
    /// `None` means the element or its anchor no longer exists.
    fn anchor_pos(&self, anchor: AnchorRef) -> Option<Pos2> {
        let element = self.element(anchor.element)?;
        let local = *element.symbol.anchors().get(anchor.anchor)?;
        Some(element.to_diagram(local))
    }

    /// Builds the gallery example: a vessel feeding a motor valve and a manual valve,
    /// with a pressure transducer and its reading on top.
    fn example(valve_stream: StreamKey, pressure_stream: StreamKey) -> Self {
        let mut diagram = Self::default();

        // Place the components along a horizontal line at y = 10
        let vessel = diagram.add_element(pos2(5.0, 10.0), Symbol::Icon(IconKind::Vessel));
        let motor_valve = diagram.add_element(
            pos2(20.0, 9.0),
            Symbol::MotorValve {
                kind: MotorValveKind::TwoWay,
                stream: Some(valve_stream),
            },
        );
        let manual_valve = diagram.add_element(pos2(35.0, 10.0), Symbol::Icon(IconKind::ManualValve));
        let transducer = diagram.add_element(pos2(5.0, -6.0), Symbol::Icon(IconKind::PressureTransducer));
        diagram.add_element(
            pos2(16.0, -6.0),
            Symbol::ValueDisplay {
                stream: Some(pressure_stream),
                unit: "bar".to_owned(),
                size: vec2(10.0, 4.0),
            },
        );
        diagram.add_element(
            pos2(5.0, 21.0),
            Symbol::Label {
                text: "Tank".to_owned(),
                size: vec2(8.0, 3.0),
            },
        );

        // Pipe the vessel's side outlet through both valves and its top into the transducer
        let at = |element, anchor| AnchorRef { element, anchor };
        diagram.connect(at(vessel, 1), at(motor_valve, 0));
        diagram.connect(at(motor_valve, 1), at(manual_valve, 0));
        diagram.connect(at(transducer, 0), at(vessel, 2));
        diagram
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementId(u32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    id: ElementId,
    /// Symbol center in diagram space, independent of the dashboard grid.
    pos: Pos2,
    rotation: Rotation,
    symbol: Symbol,
}

impl Element {
    /// Converts a point in symbol space (origin at the unrotated top-left corner)
    /// to diagram space, applying the element's position and rotation.
    fn to_diagram(&self, local: Pos2) -> Pos2 {
        let from_center = local.to_vec2() - self.symbol.size() / 2.0;
        self.pos + self.rotation.apply(from_center)
    }

    /// Returns the rotated area covered by the element in diagram space.
    fn bounds(&self) -> Rect {
        Rect::from_center_size(self.pos, self.rotation.apply_to_size(self.symbol.size()))
    }
}

/// Element orientation in clockwise quarter turns, so rotated anchors stay exactly on the grid.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Rotation {
    /// Returns the orientation after one more quarter turn clockwise.
    pub fn clockwise(self) -> Self {
        match self {
            Self::Deg0 => Self::Deg90,
            Self::Deg90 => Self::Deg180,
            Self::Deg180 => Self::Deg270,
            Self::Deg270 => Self::Deg0,
        }
    }

    /// Returns the orientation after one more quarter turn counter-clockwise.
    pub fn counter_clockwise(self) -> Self {
        match self {
            Self::Deg0 => Self::Deg270,
            Self::Deg90 => Self::Deg0,
            Self::Deg180 => Self::Deg90,
            Self::Deg270 => Self::Deg180,
        }
    }

    /// Rotates a vector clockwise on screen, where y points down.
    fn apply(self, v: Vec2) -> Vec2 {
        match self {
            Self::Deg0 => v,
            Self::Deg90 => vec2(-v.y, v.x),
            Self::Deg180 => -v,
            Self::Deg270 => vec2(v.y, -v.x),
        }
    }

    /// Returns the width and height after rotation, swapped on quarter turns.
    fn apply_to_size(self, size: Vec2) -> Vec2 {
        match self {
            Self::Deg0 | Self::Deg180 => size,
            Self::Deg90 | Self::Deg270 => vec2(size.y, size.x),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Symbol {
    Icon(IconKind),
    MotorValve {
        kind: MotorValveKind,
        stream: Option<StreamKey>,
    },
    ValueDisplay {
        stream: Option<StreamKey>,
        /// Unit appended after the value, empty for none.
        unit: String,
        /// Box the text is fitted into, in diagram units.
        size: Vec2,
    },
    Label {
        text: String,
        /// Box the text is fitted into, in diagram units.
        size: Vec2,
    },
}

impl Symbol {
    /// Returns the unrotated size in diagram units, matching the v1 symbol artwork.
    pub fn size(&self) -> Vec2 {
        match self {
            Self::Icon(icon) => match icon {
                IconKind::Arrow => vec2(4.0, 4.0),
                IconKind::BurstDisk => vec2(4.0, 6.0),
                IconKind::CheckValve | IconKind::ManualValve => vec2(10.0, 5.0),
                IconKind::FlexibleConnection => vec2(10.0, 6.0),
                IconKind::PressureGauge | IconKind::PressureTransducer => vec2(7.0, 7.0),
                IconKind::PressureRegulator => vec2(10.0, 10.0),
                IconKind::QuickConnector => vec2(6.0, 5.0),
                IconKind::ReliefValve => vec2(6.0, 10.0),
                IconKind::Vessel => vec2(8.2, 15.2),
            },
            Self::MotorValve { .. } => vec2(10.0, 8.0),
            Self::ValueDisplay { size, .. } | Self::Label { size, .. } => *size,
        }
    }

    /// Returns the points where pipes attach, in symbol space (origin at the unrotated
    /// top-left corner). Connections refer to them by index.
    pub fn anchors(&self) -> Vec<Pos2> {
        match self {
            Self::Icon(icon) => match icon {
                IconKind::Arrow => vec![pos2(0.0, 2.0), pos2(4.0, 2.0)],
                IconKind::BurstDisk => vec![pos2(0.0, 3.0), pos2(4.0, 3.0)],
                IconKind::CheckValve | IconKind::ManualValve => vec![pos2(0.0, 2.5), pos2(10.0, 2.5)],
                IconKind::FlexibleConnection => vec![pos2(0.0, 3.0), pos2(10.0, 3.0)],
                IconKind::PressureGauge | IconKind::PressureTransducer => vec![pos2(3.5, 7.0)],
                IconKind::PressureRegulator => vec![pos2(0.0, 7.0), pos2(10.0, 7.0)],
                IconKind::QuickConnector => vec![pos2(0.0, 2.5), pos2(6.0, 2.5)],
                IconKind::ReliefValve => vec![pos2(3.0, 10.0)],
                IconKind::Vessel => vec![pos2(0.0, 7.6), pos2(8.2, 7.6), pos2(4.1, 0.0), pos2(4.1, 15.1)],
            },
            Self::MotorValve {
                kind: MotorValveKind::TwoWay,
                ..
            } => vec![pos2(0.0, 5.0), pos2(10.0, 5.0)],
            Self::MotorValve {
                kind: MotorValveKind::ThreeWay { .. },
                ..
            } => vec![pos2(0.0, 3.0), pos2(10.0, 3.0), pos2(5.0, 8.0)],
            Self::ValueDisplay { .. } | Self::Label { .. } => vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IconKind {
    Arrow,
    BurstDisk,
    CheckValve,
    FlexibleConnection,
    ManualValve,
    PressureGauge,
    PressureRegulator,
    PressureTransducer,
    QuickConnector,
    ReliefValve,
    Vessel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotorValveKind {
    TwoWay,
    ThreeWay {
        /// Swaps which outlet is active for each stream value.
        invert: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValveState {
    Open,
    Closed,
    ActiveLeft,
    ActiveRight,
    /// No stream, no data yet, or a value outside the expected range.
    Unknown,
}

impl MotorValveKind {
    /// Reads the latest valve position from the data store.
    ///
    /// Returns the position mapped from the stream value:
    /// - two-way: 0 is closed, 1 is open;
    /// - three-way: 0 is left, 1 is right, swapped when `invert` is set;
    /// - anything else, or no data: [`ValveState::Unknown`].
    pub fn state(&self, stream: Option<StreamKey>, data_store: &DataStore) -> ValveState {
        let Some((_, DataValue::I64(value))) = stream.and_then(|stream| data_store.latest(stream)) else {
            return ValveState::Unknown;
        };
        match (self, value) {
            (Self::TwoWay, 0) => ValveState::Closed,
            (Self::TwoWay, 1) => ValveState::Open,
            (Self::ThreeWay { invert }, 0 | 1) if (value == 1) != *invert => ValveState::ActiveRight,
            (Self::ThreeWay { .. }, 0 | 1) => ValveState::ActiveLeft,
            _ => ValveState::Unknown,
        }
    }
}

/// Returns the text shown by a value display: the latest value followed by `unit`,
/// or "No stream" / "No data" when there is nothing to show.
fn value_text(stream: Option<StreamKey>, unit: &str, data_store: &DataStore) -> String {
    let Some(stream) = stream else {
        return "No stream".to_owned();
    };
    let Some((_, value)) = data_store.latest(stream) else {
        return "No data".to_owned();
    };
    if unit.is_empty() {
        value.to_string()
    } else {
        format!("{value} {unit}")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    start: AnchorRef,
    end: AnchorRef,
    /// Bend points in diagram space, from start to end.
    waypoints: Vec<Pos2>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorRef {
    element: ElementId,
    /// Index into the symbol's anchor list.
    anchor: usize,
}
