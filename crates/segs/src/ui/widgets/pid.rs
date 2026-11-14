use egui::{Pos2, Ui, Vec2};
use serde::{Deserialize, Serialize};

use crate::{
    dataflow::{StreamKey, store::DataStore},
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
        // TODO(UI): Paint connections and symbols scaled to the widget rect
        let _ = data_store;
        ui.centered_and_justified(|ui| ui.label(format!("P&ID: {} elements", self.elements.len())));
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
        let anchor_count = symbol.anchor_count();
        element.symbol = symbol;

        // Keep a connection only if its ends on this element are still valid anchors
        self.connections.retain(|connection| {
            [connection.start, connection.end]
                .iter()
                .all(|end| end.element != id || end.anchor < anchor_count)
        });
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
    },
    Label {
        text: String,
    },
}

impl Symbol {
    /// Number of anchor points where pipes can attach, matching the v1 symbol artwork.
    pub fn anchor_count(&self) -> usize {
        match self {
            Self::Icon(IconKind::PressureGauge | IconKind::PressureTransducer | IconKind::ReliefValve) => 1,
            Self::Icon(
                IconKind::Arrow
                | IconKind::BurstDisk
                | IconKind::CheckValve
                | IconKind::FlexibleConnection
                | IconKind::ManualValve
                | IconKind::PressureRegulator
                | IconKind::QuickConnector,
            ) => 2,
            Self::Icon(IconKind::Vessel) => 4,
            Self::MotorValve {
                kind: MotorValveKind::TwoWay,
                ..
            } => 2,
            Self::MotorValve {
                kind: MotorValveKind::ThreeWay { .. },
                ..
            } => 3,
            Self::ValueDisplay { .. } | Self::Label { .. } => 0,
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
        // TODO: Port from v1 `MotorValve::update` using `data_store.latest(stream)`
        let _ = (stream, data_store);
        todo!()
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
