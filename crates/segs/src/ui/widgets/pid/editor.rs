//! Modal editor that changes a copy of a P&ID and writes it back to the layout on save.

use egui::{
    Button, Context, CornerRadius, Id, Key, Modifiers, Pos2, Rect, ScrollArea, Sense, Ui, UiBuilder, Vec2, pos2, vec2,
};
use segs_ui::containers::Modal;

use super::{
    Element, IconKind, MotorValveKind, PidWidget, Symbol,
    canvas::Canvas,
    file,
    history::History,
    import_v1,
    properties::{self, PanelRequest},
};
use crate::{
    dataflow::{adapter::DataAdapterInstance, store::DataStore},
    ui::widgets::{WidgetData, WidgetVariant},
};

const EDITOR_ID: &str = "pid_editor";
/// Share of the window covered by the editor.
const SCREEN_FRACTION: f32 = 0.9;
const PALETTE_WIDTH: f32 = 140.;
const PROPERTIES_WIDTH: f32 = 220.;
/// Gap between the canvas and the side panels.
const PANEL_GAP: f32 = 8.;
const TOOLBAR_HEIGHT: f32 = 28.;
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
    /// View, selection and drag state of the diagram area.
    canvas: Canvas,
    /// Undo and redo steps of the draft.
    history: History,
    /// Path typed into the file field.
    file_path: String,
    /// Outcome of the last export or import, and whether it failed.
    file_status: Option<(String, bool)>,
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
            canvas: Canvas::default(),
            history: History::new(widget),
            file_path: String::new(),
            file_status: None,
        };
        ctx.data_mut(|data| data.insert_temp(Id::new(EDITOR_ID), editor));
    }

    /// Shows the open editor, if any, and applies a save to the matching widget in `widgets`.
    ///
    /// The session ends on save, on discard, or when its widget is no longer in `widgets`.
    pub fn show(
        ui: &mut Ui,
        widgets: &mut [WidgetData],
        data_store: &DataStore,
        adapter: Option<&DataAdapterInstance>,
    ) {
        let id = Id::new(EDITOR_ID);
        let Some(mut editor) = ui.ctx().data_mut(|data| data.get_temp::<Self>(id)) else {
            return;
        };
        let Some(target) = widgets.iter_mut().find(|widget| widget.id == editor.widget_id) else {
            ui.ctx().data_mut(|data| data.remove::<Self>(id));
            return;
        };

        // Treat the close button and Escape like Cancel, and a second Cancel like Discard
        let response =
            Modal::new(id.with("modal"), "Edit diagram").show(ui.ctx(), |ui| editor.contents(ui, data_store, adapter));
        let closed = response.should_close();
        let action = response.inner.or(closed.then_some(EditorAction::Cancel));

        // Apply the action, keeping the session open unless it ends here
        let close = match action {
            Some(EditorAction::Save) => {
                target.variant = WidgetVariant::Pid(editor.draft.clone());
                true
            }
            Some(EditorAction::Cancel) if editor.draft != editor.original && !editor.confirm_discard => {
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
    fn contents(
        &mut self,
        ui: &mut Ui,
        data_store: &DataStore,
        adapter: Option<&DataAdapterInstance>,
    ) -> Option<EditorAction> {
        let size = ui.ctx().content_rect().size() * SCREEN_FRACTION;

        // Lay the toolbar out in a fixed rect that cannot grow the modal, whatever its contents
        let (toolbar_rect, _) = ui.allocate_exact_size(vec2(size.x, TOOLBAR_HEIGHT), Sense::hover());
        let mut toolbar = ui.new_child(
            UiBuilder::new()
                .max_rect(toolbar_rect)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );
        toolbar.set_clip_rect(toolbar_rect);
        let action = self.toolbar(&mut toolbar);
        self.handle_history_keys(ui);

        // Split the remaining area into the palette, the canvas and the properties panel
        let (body, _) = ui.allocate_exact_size(vec2(size.x, size.y - TOOLBAR_HEIGHT), Sense::hover());
        let (palette, rest) = body.split_left_right_at_x(body.left() + PALETTE_WIDTH);
        let (canvas, properties) = rest.split_left_right_at_x(rest.right() - PROPERTIES_WIDTH);
        let canvas = canvas.shrink2(vec2(PANEL_GAP / 2., 0.));
        ui.scope_builder(UiBuilder::new().max_rect(palette.shrink2(vec2(0., 4.))), |ui| {
            self.palette(ui)
        });
        ui.painter()
            .rect_filled(canvas, CornerRadius::same(4), ui.visuals().extreme_bg_color);
        self.canvas.show(ui, canvas, &mut self.draft, data_store);

        // Show the selected symbol's settings or pipe actions, applying the edits they ask for
        let request = ui
            .scope_builder(
                UiBuilder::new().max_rect(properties.shrink2(vec2(PANEL_GAP, 4.))),
                |ui| {
                    ScrollArea::vertical()
                        .id_salt("pid_properties")
                        .show(ui, |ui| {
                            properties::show(
                                ui,
                                &mut self.draft,
                                self.canvas.selected_element(),
                                self.canvas.selected_connection().is_some(),
                                adapter,
                            )
                        })
                        .inner
                },
            )
            .inner;
        match request {
            Some(PanelRequest::DeleteElement) => {
                if let Some(id) = self.canvas.selected_element() {
                    self.draft.remove_element(id);
                    self.canvas.clear_selection();
                }
            }
            Some(PanelRequest::SplitPipe) => self.canvas.split_selected_connection(&mut self.draft),
            None => {}
        }

        // Record finished edits only, so a whole drag or a typed value is one undo step
        if !edit_in_progress(ui.ctx()) {
            self.history.record(&self.draft);
        }

        action
    }

    /// Handles Ctrl+Z for undo, and Ctrl+Y or Ctrl+Shift+Z for redo, unless a text field
    /// has focus and needs those keys for its own editing.
    fn handle_history_keys(&mut self, ui: &Ui) {
        if ui.ctx().text_edit_focused() {
            return;
        }
        let (undo, redo) = ui.input_mut(|input| {
            let redo = input.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z)
                || input.consume_key(Modifiers::COMMAND, Key::Y);
            let undo = input.consume_key(Modifiers::COMMAND, Key::Z);
            (undo, redo)
        });
        if undo {
            self.undo();
        } else if redo {
            self.redo();
        }
    }

    fn undo(&mut self) {
        if self.history.undo(&mut self.draft) {
            self.canvas.clear_selection();
        }
    }

    fn redo(&mut self) {
        if self.history.redo(&mut self.draft) {
            self.canvas.clear_selection();
        }
    }

    /// Shows Save and Cancel, or the discard confirmation while it is pending.
    /// Expects a right-to-left `ui`, so the first button added is the rightmost.
    fn toolbar(&mut self, ui: &mut Ui) -> Option<EditorAction> {
        let mut action = None;
        if self.confirm_discard {
            if ui.button("Discard").clicked() {
                action = Some(EditorAction::Discard);
            }
            if ui.button("Keep editing").clicked() {
                self.confirm_discard = false;
            }
            ui.label("Discard changes?");
        } else {
            if ui.button("Save").clicked() {
                action = Some(EditorAction::Save);
            }
            if ui.button("Cancel").clicked() {
                action = Some(EditorAction::Cancel);
            }
        }
        ui.add_space(16.);
        if ui.button("Fit view").clicked() {
            self.canvas.fit();
        }
        let can_redo = self.history.can_redo(&self.draft);
        if ui
            .add_enabled(can_redo, Button::new("Redo"))
            .on_hover_text("Ctrl+Y")
            .clicked()
        {
            self.redo();
        }
        let can_undo = self.history.can_undo(&self.draft);
        if ui
            .add_enabled(can_undo, Button::new("Undo"))
            .on_hover_text("Ctrl+Z")
            .clicked()
        {
            self.undo();
        }
        action
    }

    /// Lists the symbols that can be added, adding one to the draft when clicked.
    fn palette(&mut self, ui: &mut Ui) {
        self.file_section(ui);
        ui.add_space(4.);
        ui.weak("Click to add");
        ScrollArea::vertical().show(ui, |ui| {
            for (name, symbol) in palette_entries() {
                let button = Button::new(name).min_size(vec2(ui.available_width(), 0.));
                if ui.add(button).clicked() {
                    // Place at the view center so the new element is visible, then select it
                    let pos = self
                        .canvas
                        .center()
                        .unwrap_or_else(|| placement(&self.draft, symbol.size()));
                    let id = self.draft.add_element(pos, symbol);
                    self.canvas.select_element(id);
                }
            }
        });
    }
}

impl PidEditor {
    /// Shows the file controls: a path, Export, Import and Import v1 buttons, and the last outcome.
    fn file_section(&mut self, ui: &mut Ui) {
        egui::CollapsingHeader::new("File").show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.file_path)
                    .hint_text("Path to a .json file")
                    .desired_width(f32::INFINITY),
            );
            let has_path = !self.file_path.trim().is_empty();
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(has_path, Button::new("Export"))
                    .on_hover_text("Save this diagram to the file")
                    .clicked()
                {
                    self.export();
                }
                if ui
                    .add_enabled(has_path, Button::new("Import"))
                    .on_hover_text("Add a diagram saved with Export")
                    .clicked()
                {
                    self.import();
                }
                if ui
                    .add_enabled(has_path, Button::new("Import v1"))
                    .on_hover_text("Add a schematic saved by SEGS v1")
                    .clicked()
                {
                    self.import_v1();
                }
            });
            if let Some((message, failed)) = &self.file_status {
                if *failed {
                    ui.colored_label(ui.visuals().error_fg_color, message);
                } else {
                    ui.weak(message);
                }
            }
        });
    }

    /// Writes the draft to `file_path`, adding a `.json` extension when there is none.
    fn export(&mut self) {
        let mut path = expand_home(self.file_path.trim());
        if path.extension().is_none() {
            path.set_extension("json");
        }
        self.file_status = Some(match file::export(&path, &self.draft) {
            Ok(()) => (format!("Saved to {}.", path.display()), false),
            Err(error) => (error, true),
        });
    }

    /// Appends the diagram exported at `file_path` to the draft and reports the outcome.
    fn import(&mut self) {
        let path = expand_home(self.file_path.trim());
        self.file_status = Some(match file::import(&path, &mut self.draft) {
            Ok(elements) => {
                self.after_import();
                (format!("Imported {elements} symbols."), false)
            }
            Err(error) => (error, true),
        });
    }

    /// Appends the v1 schematic at `file_path` to the draft and reports the outcome.
    fn import_v1(&mut self) {
        let path = expand_home(self.file_path.trim());
        self.file_status = Some(match import_v1::import(&path, &mut self.draft) {
            Ok(report) => {
                self.after_import();
                let mut message = format!("Imported {} symbols and {} pipes.", report.elements, report.connections);
                if report.skipped_connections > 0 {
                    message += &format!(" Skipped {} broken pipes.", report.skipped_connections);
                }
                if report.unbound > 0 {
                    message += &format!(" Bind streams again on {} valves and displays.", report.unbound);
                }
                (message, false)
            }
            Err(error) => (error, true),
        });
    }

    /// Shows the whole diagram after something was added to it from a file.
    fn after_import(&mut self) {
        self.canvas.clear_selection();
        self.canvas.fit();
    }
}

/// Expands a leading `~/`, or `~\` on Windows, to the home directory, as a shell would.
///
/// Returns the path unchanged when it has no such prefix or the home directory is unknown.
fn expand_home(path: &str) -> std::path::PathBuf {
    let rest = path
        .strip_prefix("~/")
        .or_else(|| path.strip_prefix("~\\").filter(|_| cfg!(windows)));
    match (rest, std::env::home_dir()) {
        (Some(rest), Some(home)) => home.join(rest),
        _ => path.into(),
    }
}

/// Returns whether the user is in the middle of an edit: dragging anything or typing in a field.
fn edit_in_progress(ctx: &Context) -> bool {
    ctx.dragged_id().is_some() || ctx.text_edit_focused()
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
                color: None,
                decimals: None,
            },
        ),
        (
            "Label",
            Symbol::Label {
                text: "Label".to_owned(),
                size: vec2(8.0, 3.0),
            },
        ),
        ("Junction", Symbol::Junction),
    ]
}

/// Returns a center for a new element of `size`, to the right of everything already placed.
fn placement(diagram: &PidWidget, size: Vec2) -> Pos2 {
    match diagram.elements.iter().map(Element::bounds).reduce(Rect::union) {
        Some(bounds) => pos2(bounds.max.x + NEW_ELEMENT_GAP + size.x / 2.0, bounds.center().y),
        None => Pos2::ZERO,
    }
}
