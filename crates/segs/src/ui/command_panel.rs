use crate::ui::components::command_editor::{
    FieldDraft, ProtocolChoices, fields_are_valid, initialize_drafts, message_selector, parse_fields, protocol_choices,
    show_field_editors, target_selector,
};
use std::{sync::Arc, time::SystemTime};

use ahash::HashMap;
use chrono::{DateTime, Local};
use egui::{
    Align, Button, Frame, Grid, Id, Key, KeyboardShortcut, Label, Layout, Margin, Modifiers, Panel, RichText,
    ScrollArea, Sense, Ui, vec2,
};
use segs_ui::{
    components::{Tooltip, panel_header::PanelHeader},
    containers::Card,
    style::CtxStyleExt,
    widgets::{
        Separator,
        labels::{Badge, SectionHeader},
    },
};

use crate::{
    app::AppContext,
    dataflow::{
        Command, CommandId, CommandStatus, DataKey, DataValue, MessageKey, SourceKey,
        adapter::{DataAdapterInstance, DataAdapterInstanceToken},
        protocol::{FieldDescriptor, ProtocolDescriptor},
        store::DataStore,
    },
};

const COMPOSER_HEIGHT_FRACTION: f32 = 0.58;
const COMPOSER_ITEM_SPACING: f32 = 4.;
const COMPOSER_VERTICAL_MARGIN: i8 = 8;
const SELECTION_ROW_SPACING: f32 = 8.;
const SELECTION_SEPARATOR_SPACING: f32 = 8.;
const SEND_BUTTON_TOP_SPACING: f32 = 4.;
const COMMAND_PANEL_ID: &str = "command_panel";
const COMMAND_PANEL_OPEN_ID: &str = "command_panel_open";
const COMMAND_PANEL_FOCUS_REQUEST_ID: &str = "command_panel_focus_request";
const COMMAND_PANEL_STATE_ID: &str = "command_panel_state";
const SEND_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Enter);

/// Keyboard shortcut that toggles the global command panel.
pub const TOGGLE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::B);

/// Holds the global command composer and latest panel-issued sequence.
#[derive(Clone, Default)]
struct CommandPanelState {
    adapter_token: Option<DataAdapterInstanceToken>,
    target: Option<SourceKey>,
    message: Option<MessageKey>,
    /// Protocol catalogs shared with transition settings, independent of this panel's selections.
    choices: Option<Arc<ProtocolChoices>>,
    drafts: HashMap<DataKey, FieldDraft>,
    latest_sequence: Option<CommandId>,
}

/// Shows the global command panel when it is open.
pub fn show(ui: &mut Ui, appctx: &mut AppContext) {
    let state_id = Id::new(COMMAND_PANEL_STATE_ID);
    let mut state = ui
        .data_mut(|data| data.remove_temp::<CommandPanelState>(state_id))
        .unwrap_or_default();
    state.sync_adapter(ui.ctx(), appctx.data_adapter.as_ref());
    let request_initial_focus = ui
        .data(|data| data.get_temp::<bool>(Id::new(COMMAND_PANEL_FOCUS_REQUEST_ID)))
        .unwrap_or(false);

    // Render the panel and forward one-shot focus requests to its first control
    let app_style = ui.app_style();
    let panel_frame = Frame::new().fill(app_style.main_panels_fill);
    if is_open(ui) {
        Panel::left(COMMAND_PANEL_ID)
            .default_size(300.)
            .min_size(260.)
            .max_size(400.)
            .frame(panel_frame)
            .show(ui, |ui| {
                show_contents(ui, &mut state, appctx, request_initial_focus);
            });
    } else {
        // Keep following widget identities stable while this conditional panel is absent
        ui.skip_ahead_auto_ids(1);
    }

    ui.data_mut(|data| data.insert_temp(state_id, state));
}

/// Reports whether the global command panel is currently open.
///
/// The returned value is `true` while the panel should be rendered.
pub fn is_open(ui: &Ui) -> bool {
    ui.data(|data| data.get_temp(Id::new(COMMAND_PANEL_OPEN_ID)))
        .unwrap_or(false)
}

/// Toggles the global command panel and requests initial focus when opening it.
pub fn toggle(ui: &mut Ui) {
    let open = is_open(ui);
    ui.data_mut(|data| {
        data.insert_temp(Id::new(COMMAND_PANEL_OPEN_ID), !open);
        if open {
            data.remove_temp::<bool>(Id::new(COMMAND_PANEL_FOCUS_REQUEST_ID));
        } else {
            data.insert_temp(Id::new(COMMAND_PANEL_FOCUS_REQUEST_ID), true);
        }
    });
}

impl CommandPanelState {
    fn sync_adapter(&mut self, ctx: &egui::Context, adapter: Option<&DataAdapterInstance>) {
        let token = adapter.map(DataAdapterInstance::token);
        let unchanged = match (&self.adapter_token, token) {
            (Some(current), Some(token)) => current == token,
            (None, None) => true,
            _ => false,
        };
        if unchanged {
            return;
        }

        // Reset panel-owned inputs while reusing catalogs already built by another editor
        *self = Self {
            adapter_token: token.cloned(),
            choices: adapter.map(|adapter| protocol_choices(ctx, adapter)),
            ..Self::default()
        };
    }
}

fn show_contents(ui: &mut Ui, state: &mut CommandPanelState, appctx: &mut AppContext, request_initial_focus: bool) {
    ui.add(PanelHeader::new("COMMANDS").subtitle("Send commands to targets"));

    let Some(adapter) = appctx.data_adapter.as_ref() else {
        Frame::new().inner_margin(ui.spacing().window_margin).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.weak("Connect a data source to compose commands.");
        });
        return;
    };
    let protocol = adapter.describe_protocol();
    let composer_height = (ui.available_height() * COMPOSER_HEIGHT_FRACTION).max(160.);
    let panel_item_spacing = ui.spacing().item_spacing.y;
    let composer_margin = Margin {
        top: COMPOSER_VERTICAL_MARGIN,
        bottom: COMPOSER_VERTICAL_MARGIN,
        ..ui.spacing().window_margin
    };

    // Join the header, composer, and latest-sequence sections without implicit gaps
    ui.spacing_mut().item_spacing.y = 0.;
    ScrollArea::vertical()
        .id_salt("command_composer")
        .max_height(composer_height)
        .auto_shrink([false, true])
        .content_margin(composer_margin)
        .show(ui, |ui| {
            show_composer(ui, state, protocol, &mut appctx.data_store, request_initial_focus);
        });

    ui.spacing_mut().item_spacing.y = panel_item_spacing;
    ui.add(Separator::default().spacing(0.));
    show_latest_sequence(ui, state.latest_sequence, protocol, &appctx.data_store);
}

fn show_composer(
    ui: &mut Ui,
    state: &mut CommandPanelState,
    protocol: &ProtocolDescriptor,
    store: &mut DataStore,
    request_initial_focus: bool,
) {
    // Reserve the panel-wide shortcut before focused controls can interpret Enter
    let shortcut_pressed = ui.input_mut(|input| input.consume_shortcut(&SEND_SHORTCUT));
    ui.spacing_mut().item_spacing.y = COMPOSER_ITEM_SPACING;

    // Align command selection controls to one shared label column
    let message_changed = Grid::new("command_selection_grid")
        .num_columns(2)
        .spacing([8., SELECTION_ROW_SPACING])
        .show(ui, |ui| {
            show_target_selector(ui, state, request_initial_focus);
            ui.end_row();
            // Leave the final row open to avoid reserving trailing row spacing
            show_message_selector(ui, state, protocol)
        })
        .inner;
    if protocol.sources.is_empty() {
        ui.weak("This protocol exposes no command targets.");
    }

    // Separate command selection from the editable payload and send action
    let horizontal_margin = ui.spacing().window_margin.leftf();
    ui.add(
        Separator::default()
            .spacing(SELECTION_SEPARATOR_SPACING)
            .grow(horizontal_margin),
    );

    let has_fields = state
        .message
        .is_some_and(|message_key| !protocol.message_schemas[&message_key].fields.is_empty());
    if has_fields {
        // Render the selected command's editable payload
        let message_key = state.message.expect("field rendering requires a selected command");
        let message = &protocol.message_schemas[&message_key];
        ui.push_id("command_message_fields", |ui| {
            show_field_editors(ui, &message.fields, &mut state.drafts, !message_changed);
        });
    }

    let ready = state.target.is_some()
        && state.message.is_some()
        && state
            .message
            .is_some_and(|key| fields_are_valid(&protocol.message_schemas[&key].fields, &state.drafts));
    if has_fields {
        ui.add_space(SEND_BUTTON_TOP_SPACING);
    }

    // Mirror egui's active button visuals while the valid send shortcut is held
    let shortcut_down = ready
        && ui.input(|input| {
            input.key_down(SEND_SHORTCUT.logical_key) && input.modifiers.matches_logically(SEND_SHORTCUT.modifiers)
        });

    // Accept pointer clicks or the panel-wide shortcut without adding Send to focus traversal
    let send_row_size = vec2(ui.available_width(), ui.spacing().interact_size.y);
    let response = ui
        .allocate_ui_with_layout(send_row_size, Layout::right_to_left(Align::Center), |ui| {
            if shortcut_down {
                let active = ui.visuals().widgets.active;
                ui.visuals_mut().widgets.inactive = active;
                ui.visuals_mut().widgets.hovered = active;
            }
            let send_button = Button::new("Send").sense(Sense::click() - Sense::focusable_noninteractive());
            ui.add_enabled(ready, send_button)
        })
        .inner;
    Tooltip::new(&response, "Send Command").shortcut(SEND_SHORTCUT).show();

    if ready && (response.clicked() || shortcut_pressed) {
        let message_key = state.message.expect("send requires a selected message");
        let target = state.target.expect("send requires a selected target");
        let fields = parse_fields(&protocol.message_schemas[&message_key].fields, &state.drafts)
            .expect("send requires valid field drafts");
        state.latest_sequence = Some(store.enqueue_command(Command {
            key: message_key,
            target,
            timestamp: SystemTime::now(),
            fields,
        }));
    }
}

fn show_target_selector(ui: &mut Ui, state: &mut CommandPanelState, request_initial_focus: bool) {
    let Some(choices) = state.choices.as_ref() else {
        return;
    };
    let Some(adapter_token) = state.adapter_token.as_ref() else {
        return;
    };
    let selector_id = ui.make_persistent_id(("command_target_selector", adapter_token));

    ui.label("Target");
    let response = target_selector(ui, selector_id, &choices.targets, &mut state.target);
    if request_initial_focus && response.enabled() {
        // Consume initial focus only after an enabled first control accepts it
        response.request_focus();
        ui.data_mut(|data| data.remove_temp::<bool>(Id::new(COMMAND_PANEL_FOCUS_REQUEST_ID)));
    }
}

fn show_message_selector(ui: &mut Ui, state: &mut CommandPanelState, protocol: &ProtocolDescriptor) -> bool {
    let Some(choices) = state.choices.as_ref() else {
        return false;
    };
    let Some(adapter_token) = state.adapter_token.as_ref() else {
        return false;
    };
    let selector_id = ui.make_persistent_id(("command_message_selector", adapter_token));

    ui.label("Command");
    let response = message_selector(ui, selector_id, &choices.commands, &mut state.message);
    if response.changed() {
        let key = state.message.expect("changed command selection must contain a value");
        state.drafts.clear();
        initialize_drafts(&protocol.message_schemas[&key].fields, &mut state.drafts);
    }
    response.changed()
}

fn show_latest_sequence(ui: &mut Ui, command_id: Option<CommandId>, protocol: &ProtocolDescriptor, store: &DataStore) {
    ScrollArea::vertical()
        .id_salt("latest_command_sequence")
        .auto_shrink([false, false])
        .content_margin(ui.spacing().window_margin)
        .show(ui, |ui| {
            ui.label(RichText::new("LATEST SEQUENCE").strong());
            ui.add_space(6.);

            let Some(command_id) = command_id else {
                ui.weak("No command has been sent from this panel.");
                return;
            };
            let sequence = store.command_sequence(command_id);

            ui.add(SectionHeader::new("Request"));
            ui.add_space(2.);
            show_command_card(ui, protocol, &sequence.request, Some(&sequence.status), "To");

            ui.add_space(10.);
            ui.add(SectionHeader::new("Responses"));
            ui.add_space(2.);
            if sequence.responses.is_empty() {
                ui.weak("No responses received.");
            } else {
                for (index, response) in sequence.responses.iter().enumerate() {
                    if index > 0 {
                        ui.add_space(8.);
                    }
                    show_command_card(ui, protocol, response, None, "From");
                }
            }
        });
}

fn show_command_card(
    ui: &mut Ui,
    protocol: &ProtocolDescriptor,
    command: &Command,
    status: Option<&CommandStatus>,
    direction: &str,
) {
    let descriptor = &protocol.message_schemas[&command.key];
    Card::new().show(ui, |ui| {
        // Keep cards aligned regardless of whether their contents naturally fill the row
        ui.take_available_width();

        ui.horizontal(|ui| {
            ui.label(RichText::new(&descriptor.name).strong());
            if let Some(status) = status {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| show_status_badge(ui, status));
            }
        });
        let timestamp: DateTime<Local> = command.timestamp.into();
        ui.weak(format!(
            "{direction} {} · {}",
            source_name(protocol, command.target),
            timestamp.format("%H:%M:%S")
        ));
        if !descriptor.fields.is_empty() {
            ui.add_space(6.);
            show_field_values(ui, &descriptor.fields, &command.fields);
        }
    });
}

fn show_status_badge(ui: &mut Ui, status: &CommandStatus) {
    let app_style = ui.app_style();
    let fill = match status {
        CommandStatus::Pending => app_style.neutral_fill,
        CommandStatus::TimedOut => app_style.timeout_fill,
        CommandStatus::Success => app_style.success_fill,
        CommandStatus::Rejected => app_style.error_fill,
        CommandStatus::LocalError => app_style.local_error_fill,
    };
    ui.add(Badge::new(status.to_string()).fill(fill));
}

fn show_field_values(ui: &mut Ui, descriptors: &[FieldDescriptor], values: &HashMap<DataKey, DataValue>) {
    for descriptor in descriptors {
        // Resolve the shared leaf data while preserving nested structures
        let (name, data_key, enum_descriptor) = match descriptor {
            FieldDescriptor::Structure { name, fields } => {
                ui.label(RichText::new(name).strong());
                ui.indent(name, |ui| show_field_values(ui, fields, values));
                continue;
            }
            FieldDescriptor::Field { name, data_key, .. } => (name, data_key, None),
            FieldDescriptor::EnumField {
                name,
                descriptor,
                data_key,
            } => (name, data_key, Some(descriptor)),
        };
        let value = &values[data_key];

        // Prefer a matching enum variant while retaining unknown numeric values
        let displayed_value = enum_descriptor
            .and_then(|descriptor| {
                descriptor
                    .variants
                    .iter()
                    .find(|(_, variant_value)| variant_value == value)
            })
            .map_or_else(|| value.to_string(), |(variant_name, _)| variant_name.clone());

        // Show the resolved field value on the right side of the row
        ui.horizontal(|ui| {
            ui.label(name);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(Label::new(displayed_value).truncate());
            });
        });
    }
}

fn source_name(protocol: &ProtocolDescriptor, key: SourceKey) -> &str {
    protocol
        .sources
        .iter()
        .find(|source| source.key == key)
        .expect("Adapter command target must be a described source")
        .name
        .as_str()
}
