//! Side panel of the P&ID editor showing the selected element's settings.

use egui::{DragValue, Grid, Ui};

use super::{ElementId, IconKind, MotorValveKind, PidWidget, Symbol};
use crate::{
    dataflow::adapter::DataAdapterInstance,
    ui::{views::configuration::settings::stream_selector, widget_settings::StreamValueFilter},
};

/// Size limits for labels and value displays, in diagram units.
const MIN_BOX_SIZE: f32 = 1.0;
const MAX_BOX_SIZE: f32 = 200.0;

/// Shows the settings of the element `selected`, applying edits to `diagram`.
///
/// Returns `true` when the user asked to delete the element; the caller removes it
/// so the canvas selection can be cleared at the same time.
pub(super) fn show(
    ui: &mut Ui,
    diagram: &mut PidWidget,
    selected: Option<ElementId>,
    adapter: Option<&DataAdapterInstance>,
) -> bool {
    let Some(element) = selected.and_then(|id| diagram.elements.iter_mut().find(|element| element.id == id)) else {
        ui.weak("Select a symbol to edit its settings.");
        return false;
    };
    let id = element.id;

    // Scope widget state per element so selectors do not carry over between selections
    ui.push_id(id, |ui| show_element(ui, diagram, id, adapter)).inner
}

/// Shows the settings of the element `id`, which must be in `diagram`.
///
/// Returns `true` when the user asked to delete the element.
fn show_element(ui: &mut Ui, diagram: &mut PidWidget, id: ElementId, adapter: Option<&DataAdapterInstance>) -> bool {
    let Some(element) = diagram.elements.iter_mut().find(|element| element.id == id) else {
        return false;
    };
    ui.strong(symbol_name(&element.symbol));
    ui.add_space(4.);

    // Rotation applies to every symbol
    ui.horizontal(|ui| {
        ui.label("Rotation");
        if ui.button("-90°").on_hover_text("Counter-clockwise (Shift+R)").clicked() {
            element.rotation = element.rotation.counter_clockwise();
        }
        if ui.button("+90°").on_hover_text("Clockwise (R)").clicked() {
            element.rotation = element.rotation.clockwise();
        }
    });
    ui.add_space(8.);

    // Edit the symbol's own fields, collecting a kind change that alters its anchors
    let mut replacement = None;
    match &mut element.symbol {
        Symbol::Icon(_) => {
            ui.weak("This symbol has no settings.");
        }
        Symbol::MotorValve { kind, stream } => {
            let mut three_way = matches!(kind, MotorValveKind::ThreeWay { .. });
            ui.horizontal(|ui| {
                ui.selectable_value(&mut three_way, false, "Two-way");
                ui.selectable_value(&mut three_way, true, "Three-way");
            });
            if let MotorValveKind::ThreeWay { invert } = kind {
                ui.checkbox(invert, "Invert outlets");
            }
            ui.add_space(4.);
            stream_selector::show(ui, "State stream", stream, None, StreamValueFilter::Integer, adapter);
            if three_way != matches!(kind, MotorValveKind::ThreeWay { .. }) {
                replacement = Some(Symbol::MotorValve {
                    kind: if three_way {
                        MotorValveKind::ThreeWay { invert: false }
                    } else {
                        MotorValveKind::TwoWay
                    },
                    stream: *stream,
                });
            }
        }
        Symbol::ValueDisplay { stream, unit, size } => {
            stream_selector::show(ui, "Stream", stream, None, StreamValueFilter::Any, adapter);
            ui.add_space(4.);
            Grid::new("pid_value_display_settings").num_columns(2).show(ui, |ui| {
                ui.label("Unit");
                ui.text_edit_singleline(unit);
                ui.end_row();
                size_rows(ui, size);
            });
        }
        Symbol::Label { text, size } => {
            Grid::new("pid_label_settings").num_columns(2).show(ui, |ui| {
                ui.label("Text");
                ui.text_edit_singleline(text);
                ui.end_row();
                size_rows(ui, size);
            });
        }
    }

    // Swap the symbol through the diagram so pipes on removed anchors are dropped
    if let Some(symbol) = replacement {
        diagram.set_symbol(id, symbol);
    }

    ui.add_space(12.);
    ui.button("Delete symbol").clicked()
}

/// Adds width and height rows for a text box size, in diagram units.
fn size_rows(ui: &mut Ui, size: &mut egui::Vec2) {
    for (label, value) in [("Width", &mut size.x), ("Height", &mut size.y)] {
        ui.label(label);
        ui.add(DragValue::new(value).range(MIN_BOX_SIZE..=MAX_BOX_SIZE).speed(0.1));
        ui.end_row();
    }
}

/// Returns the name shown for a symbol, matching the editor palette.
fn symbol_name(symbol: &Symbol) -> &'static str {
    match symbol {
        Symbol::Icon(icon) => match icon {
            IconKind::Arrow => "Arrow",
            IconKind::BurstDisk => "Burst disk",
            IconKind::CheckValve => "Check valve",
            IconKind::FlexibleConnection => "Flexible connection",
            IconKind::ManualValve => "Manual valve",
            IconKind::PressureGauge => "Pressure gauge",
            IconKind::PressureRegulator => "Pressure regulator",
            IconKind::PressureTransducer => "Pressure transducer",
            IconKind::QuickConnector => "Quick connector",
            IconKind::ReliefValve => "Relief valve",
            IconKind::Vessel => "Vessel",
        },
        Symbol::MotorValve {
            kind: MotorValveKind::TwoWay,
            ..
        } => "Motor valve",
        Symbol::MotorValve {
            kind: MotorValveKind::ThreeWay { .. },
            ..
        } => "Three-way valve",
        Symbol::ValueDisplay { .. } => "Value display",
        Symbol::Label { .. } => "Label",
    }
}
