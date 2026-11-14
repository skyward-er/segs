//! Modal editor that changes a copy of a P&ID and writes it back to the layout on save.

use egui::{Button, Context, CornerRadius, Id, Pos2, Rect, ScrollArea, Sense, Ui, UiBuilder, Vec2, pos2, vec2};
use segs_ui::containers::Modal;

use super::{Element, IconKind, MotorValveKind, PidWidget, Symbol, paint};
use crate::{
    dataflow::store::DataStore,
    ui::widgets::{WidgetData, WidgetVariant},
};

const EDITOR_ID: &str = "pid_editor";
/// Share of the window covered by the editor.
const SCREEN_FRACTION: f32 = 0.85;
const PALETTE_WIDTH: f32 = 170.;
const TOOLBAR_HEIGHT: f32 = 28.;
const CANVAS_PADDING: f32 = 12.;
/// Horizontal gap between a newly added element and the rest of the diagram, in diagram units.
const NEW_ELEMENT_GAP: f32 = 4.0;

/// Editing session for one P&ID widget, kept in egui memory while the modal is open.
#[derive(Clone)]
pub struct PidEditor {
    /// Layout widget the draft is written back to.
    widget_id: Id,
    /// Diagram as it was when the editor opened, used to detect unsaved changes.
    original: PidWidget,
    /// Working copy changed by the editor.
    draft: PidWidget,
    /// Whether cancelling is waiting for the user to confirm losing changes.
    confirm_discard: bool,
}

/// What the user asked the editor to do this frame.
enum EditorAction {
    Save,
    Cancel,
    Discard,
}

impl PidEditor {
    /// Starts editing `widget`, replacing any session already open.
    pub fn open(ctx: &Context, widget_id: Id, widget: &PidWidget) {
        let editor = Self {
            widget_id,
            original: widget.clone(),
            draft: widget.clone(),
            confirm_discard: false,
        };
        ctx.data_mut(|data| data.insert_temp(Id::new(EDITOR_ID), editor));
    }

    /// Shows the open editor, if any, and applies a save to the matching widget in `widgets`.
    ///
    /// The session ends on save, on discard, or when its widget is no longer in `widgets`.
    pub fn show(ui: &mut Ui, widgets: &mut [WidgetData], data_store: &DataStore) {
        let id = Id::new(EDITOR_ID);
        let Some(mut editor) = ui.ctx().data_mut(|data| data.get_temp::<Self>(id)) else {
            return;
        };
        let Some(target) = widgets.iter_mut().find(|widget| widget.id == editor.widget_id) else {
            ui.ctx().data_mut(|data| data.remove::<Self>(id));
            return;
        };

        // Treat the close button and Escape like Cancel
        let response =
            Modal::new(id.with("modal"), "Edit diagram").show(ui.ctx(), |ui| editor.contents(ui, data_store));
        let closed = response.should_close();
        let action = response.inner.or(closed.then_some(EditorAction::Cancel));

        // Apply the action, keeping the session open unless it ends here
        let close = match action {
            Some(EditorAction::Save) => {
                target.variant = WidgetVariant::Pid(editor.draft.clone());
                true
            }
            Some(EditorAction::Cancel) if editor.draft != editor.original => {
                editor.confirm_discard = true;
                false
            }
            Some(EditorAction::Cancel | EditorAction::Discard) => true,
            None => false,
        };
        ui.ctx().data_mut(|data| {
            if close {
                data.remove::<Self>(id);
            } else {
                data.insert_temp(id, editor);
            }
        });
    }

    /// Lays out the toolbar, symbol palette and diagram, returning the action chosen this frame.
    fn contents(&mut self, ui: &mut Ui, data_store: &DataStore) -> Option<EditorAction> {
        let size = ui.ctx().content_rect().size() * SCREEN_FRACTION;
        let action = ui
            .allocate_ui(vec2(size.x, TOOLBAR_HEIGHT), |ui| self.toolbar(ui))
            .inner;

        // Split the remaining area into the palette and the canvas
        let (body, _) = ui.allocate_exact_size(vec2(size.x, size.y - TOOLBAR_HEIGHT), Sense::hover());
        let (palette, canvas) = body.split_left_right_at_x(body.left() + PALETTE_WIDTH);
        ui.scope_builder(UiBuilder::new().max_rect(palette.shrink2(vec2(0., 4.))), |ui| {
            self.palette(ui)
        });
        ui.painter()
            .rect_filled(canvas, CornerRadius::same(4), ui.visuals().extreme_bg_color);
        ui.scope_builder(UiBuilder::new().max_rect(canvas.shrink(CANVAS_PADDING)), |ui| {
            paint::show(&self.draft, ui, data_store);
        });

        action
    }

    /// Shows Save and Cancel, or the discard confirmation while it is pending.
    fn toolbar(&mut self, ui: &mut Ui) -> Option<EditorAction> {
        let mut action = None;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if self.confirm_discard {
                if ui.button("Discard").clicked() {
                    action = Some(EditorAction::Discard);
                }
                if ui.button("Keep editing").clicked() {
                    self.confirm_discard = false;
                }
                ui.label("Discard unsaved changes?");
            } else {
                if ui.button("Save").clicked() {
                    action = Some(EditorAction::Save);
                }
                if ui.button("Cancel").clicked() {
                    action = Some(EditorAction::Cancel);
                }
            }
        });
        action
    }

    /// Lists the symbols that can be added, adding one to the draft when clicked.
    fn palette(&mut self, ui: &mut Ui) {
        ui.weak("Click to add");
        ScrollArea::vertical().show(ui, |ui| {
            for (name, symbol) in palette_entries() {
                let button = Button::new(name).min_size(vec2(ui.available_width(), 0.));
                if ui.add(button).clicked() {
                    let pos = placement(&self.draft, symbol.size());
                    self.draft.add_element(pos, symbol);
                }
            }
        });
    }
}

/// Returns the palette in display order, each with the symbol it adds.
fn palette_entries() -> Vec<(&'static str, Symbol)> {
    let icon = |kind| Symbol::Icon(kind);
    vec![
        ("Vessel", icon(IconKind::Vessel)),
        ("Manual valve", icon(IconKind::ManualValve)),
        ("Check valve", icon(IconKind::CheckValve)),
        ("Relief valve", icon(IconKind::ReliefValve)),
        ("Burst disk", icon(IconKind::BurstDisk)),
        ("Pressure regulator", icon(IconKind::PressureRegulator)),
        ("Pressure gauge", icon(IconKind::PressureGauge)),
        ("Pressure transducer", icon(IconKind::PressureTransducer)),
        ("Quick connector", icon(IconKind::QuickConnector)),
        ("Flexible connection", icon(IconKind::FlexibleConnection)),
        ("Arrow", icon(IconKind::Arrow)),
        (
            "Motor valve",
            Symbol::MotorValve {
                kind: MotorValveKind::TwoWay,
                stream: None,
            },
        ),
        (
            "Three-way valve",
            Symbol::MotorValve {
                kind: MotorValveKind::ThreeWay { invert: false },
                stream: None,
            },
        ),
        (
            "Value display",
            Symbol::ValueDisplay {
                stream: None,
                unit: String::new(),
                size: vec2(10.0, 4.0),
            },
        ),
        (
            "Label",
            Symbol::Label {
                text: "Label".to_owned(),
                size: vec2(8.0, 3.0),
            },
        ),
    ]
}

/// Returns a center for a new element of `size`, to the right of everything already placed.
fn placement(diagram: &PidWidget, size: Vec2) -> Pos2 {
    match diagram.elements.iter().map(Element::bounds).reduce(Rect::union) {
        Some(bounds) => pos2(bounds.max.x + NEW_ELEMENT_GAP + size.x / 2.0, bounds.center().y),
        None => Pos2::ZERO,
    }
}
