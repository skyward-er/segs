use std::{collections::HashMap, fmt, time::SystemTime};

use chrono::{DateTime, Local};
use egui::{Align, Button, ComboBox, Frame, Id, Layout, Margin, Panel, RichText, ScrollArea, Ui};
use segs_ui::{
    components::panel_header::PanelHeader,
    containers::Card,
    style::CtxStyleExt,
    widgets::{
        ExpandableSelector, Separator,
        labels::{Badge, SectionHeader, SelectableRow},
        text::{TextEdit, ValidationTextEdit},
    },
};

use crate::{
    app::AppContext,
    dataflow::{
        Command, CommandId, CommandStatus, DataKey, DataType, DataValue, MessageKey, SourceKey,
        adapter::DataAdapterInstanceToken,
        protocol::{FieldDescriptor, MessageDescriptor, ProtocolDescriptor},
        store::DataStore,
    },
};

const COMPOSER_HEIGHT_FRACTION: f32 = 0.58;
const COMPOSER_ITEM_SPACING: f32 = 4.;
const SEND_BUTTON_TOP_SPACING: f32 = 4.;
const MESSAGE_LIST_HEIGHT: f32 = 180.;
const COMMAND_PANEL_ID: &str = "command_panel";
const COMMAND_PANEL_OPEN_ID: &str = "command_panel_open";
const COMMAND_PANEL_STATE_ID: &str = "command_panel_state";

/// Holds the global command composer and latest panel-issued sequence.
#[derive(Clone, Default)]
struct CommandPanelState {
    adapter_token: Option<DataAdapterInstanceToken>,
    target: Option<SourceKey>,
    message: Option<MessageKey>,
    message_picker_open: bool,
    query: String,
    drafts: HashMap<DataKey, FieldDraft>,
    latest_sequence: Option<CommandId>,
}

/// Shows the global command panel when it is open.
pub fn show(ui: &mut Ui, appctx: &mut AppContext) {
    let state_id = Id::new(COMMAND_PANEL_STATE_ID);
    let mut state = ui
        .data_mut(|data| data.remove_temp::<CommandPanelState>(state_id))
        .unwrap_or_default();
    state.sync_adapter(appctx.data_adapter.as_ref().map(|adapter| adapter.token()));

    let app_style = ui.app_style();
    let panel_frame = Frame::new().fill(app_style.main_panels_fill);
    if is_open(ui) {
        Panel::left(COMMAND_PANEL_ID)
            .default_size(300.)
            .min_size(260.)
            .max_size(400.)
            .frame(panel_frame)
            .show_inside(ui, |ui| show_contents(ui, &mut state, appctx));
    } else {
        // Keep following widget identities stable while this conditional panel is absent
        ui.skip_ahead_auto_ids(1);
    }

    ui.data_mut(|data| data.insert_temp(state_id, state));
}

pub fn is_open(ui: &Ui) -> bool {
    ui.data(|data| data.get_temp(Id::new(COMMAND_PANEL_OPEN_ID)))
        .unwrap_or(false)
}

pub fn toggle(ui: &mut Ui) {
    let open = is_open(ui);
    ui.data_mut(|data| data.insert_temp(Id::new(COMMAND_PANEL_OPEN_ID), !open));
}

#[derive(Clone, Default)]
struct FieldDraft {
    text: String,
    touched: bool,
}

impl CommandPanelState {
    fn sync_adapter(&mut self, token: Option<&DataAdapterInstanceToken>) {
        let unchanged = match (&self.adapter_token, token) {
            (Some(current), Some(token)) => current == token,
            (None, None) => true,
            _ => false,
        };
        if unchanged {
            return;
        }

        *self = Self {
            adapter_token: token.cloned(),
            ..Self::default()
        };
    }

    fn select_message(&mut self, key: MessageKey, descriptor: &MessageDescriptor) {
        if self.message == Some(key) {
            self.message_picker_open = false;
            return;
        }

        self.message = Some(key);
        self.message_picker_open = false;
        self.drafts.clear();
        initialize_drafts(&descriptor.fields, &mut self.drafts);
    }
}

fn show_contents(ui: &mut Ui, state: &mut CommandPanelState, appctx: &mut AppContext) {
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

    ScrollArea::vertical()
        .id_salt("command_composer")
        .max_height(composer_height)
        .auto_shrink([false, true])
        .content_margin(ui.spacing().window_margin)
        .show(ui, |ui| show_composer(ui, state, protocol, &mut appctx.data_store));

    ui.add(Separator::default().spacing(0.));
    show_latest_sequence(ui, state.latest_sequence, protocol, &appctx.data_store);
}

fn show_composer(ui: &mut Ui, state: &mut CommandPanelState, protocol: &ProtocolDescriptor, store: &mut DataStore) {
    ui.spacing_mut().item_spacing.y = COMPOSER_ITEM_SPACING;

    show_target_selector(ui, state, protocol);
    show_message_selector(ui, state, protocol);

    if let Some(message_key) = state.message {
        let message = &protocol.message_schemas[&message_key];
        show_field_editors(ui, &message.fields, &mut state.drafts);
    }

    let ready = state.target.is_some()
        && state.message.is_some()
        && state
            .message
            .is_some_and(|key| fields_are_valid(&protocol.message_schemas[&key].fields, &state.drafts));
    ui.add_space(SEND_BUTTON_TOP_SPACING);
    if ui.add_enabled(ready, Button::new("Send")).clicked() {
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

fn show_target_selector(ui: &mut Ui, state: &mut CommandPanelState, protocol: &ProtocolDescriptor) {
    ui.horizontal(|ui| {
        ui.label("Target");
        let selected = state
            .target
            .map(|key| source_name(protocol, key))
            .unwrap_or("Select target");
        ui.add_enabled_ui(!protocol.sources.is_empty(), |ui| {
            ComboBox::from_id_salt("command_target")
                .width(ui.available_width())
                .truncate()
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for source in &protocol.sources {
                        ui.selectable_value(&mut state.target, Some(source.key), &source.name);
                    }
                });
        });
    });
    if protocol.sources.is_empty() {
        ui.weak("This protocol exposes no command targets.");
    }
}

fn show_message_selector(ui: &mut Ui, state: &mut CommandPanelState, protocol: &ProtocolDescriptor) {
    let content_margin = ui.spacing().window_margin;
    let composer_item_spacing = ui.spacing().item_spacing.y;
    let selected = state
        .message
        .map(|key| protocol.message_schemas[&key].name.as_str())
        .unwrap_or("Select message");

    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 0.;
        ui.add(
            ExpandableSelector::new("Message", selected, &mut state.message_picker_open)
                .preview_weak(state.message.is_none())
                .horizontal_bleed(content_margin),
        );

        if !state.message_picker_open {
            return;
        }

        let inner_margin = Margin {
            left: content_margin.left,
            right: content_margin.right,
            top: 6,
            bottom: 0,
        };
        let outer_margin = Margin {
            left: -content_margin.left,
            right: -content_margin.right,
            top: 0,
            bottom: 0,
        };
        Frame::new()
            .fill(ui.visuals().panel_fill)
            .inner_margin(inner_margin)
            .outer_margin(outer_margin)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = composer_item_spacing;
                ui.add(
                    TextEdit::singleline(&mut state.query)
                        .hint_text("Search command messages…")
                        .desired_width(ui.available_width()),
                );
                let messages = filtered_command_messages(protocol, &state.query);
                if messages.is_empty() {
                    ui.weak("No matching command messages.");
                } else {
                    ui.visuals_mut().clip_rect_margin = 0.;
                    ScrollArea::vertical()
                        .id_salt("command_message_list")
                        .max_height(MESSAGE_LIST_HEIGHT)
                        .min_scrolled_height(0.)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for key in messages {
                                let descriptor = &protocol.message_schemas[&key];
                                if ui
                                    .add(SelectableRow::new(state.message == Some(key), &descriptor.name))
                                    .clicked()
                                {
                                    state.select_message(key, descriptor);
                                }
                            }
                        });
                }
            });
    });
}

fn show_field_editors(ui: &mut Ui, descriptors: &[FieldDescriptor], drafts: &mut HashMap<DataKey, FieldDraft>) {
    for descriptor in descriptors {
        match descriptor {
            FieldDescriptor::Structure { name, fields } => {
                ui.label(RichText::new(name).strong());
                ui.indent(name, |ui| show_field_editors(ui, fields, drafts));
            }
            FieldDescriptor::Field {
                name,
                field_type,
                data_key,
            } => {
                let draft = drafts
                    .get_mut(data_key)
                    .expect("Selected message fields must have initialized drafts");
                let error = draft
                    .touched
                    .then(|| parse_field(draft, field_type).err())
                    .flatten()
                    .map(|error| error.to_string());

                ui.label(RichText::new(format!("{name} · {field_type}")).size(11.));
                let mut editor = ValidationTextEdit::new(&mut draft.text)
                    .id_salt(*data_key)
                    .desired_width(ui.available_width());
                if let Some(error) = error {
                    editor = editor.error(error);
                }
                let response = ui.add(editor);
                if response.changed() || response.lost_focus() {
                    draft.touched = true;
                }
            }
        }
    }
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
        ui.add_space(6.);
        show_field_values(ui, &descriptor.fields, &command.fields);
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
        match descriptor {
            FieldDescriptor::Structure { name, fields } => {
                ui.label(RichText::new(name).strong());
                ui.indent(name, |ui| show_field_values(ui, fields, values));
            }
            FieldDescriptor::Field { name, data_key, .. } => {
                ui.horizontal(|ui| {
                    ui.label(name);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(values[data_key].to_string());
                    });
                });
            }
        }
    }
}

fn initialize_drafts(descriptors: &[FieldDescriptor], drafts: &mut HashMap<DataKey, FieldDraft>) {
    for descriptor in descriptors {
        match descriptor {
            FieldDescriptor::Structure { fields, .. } => initialize_drafts(fields, drafts),
            FieldDescriptor::Field { data_key, .. } => {
                drafts.insert(*data_key, FieldDraft::default());
            }
        }
    }
}

fn filtered_command_messages(protocol: &ProtocolDescriptor, query: &str) -> Vec<MessageKey> {
    let query = query.trim().to_lowercase();
    protocol
        .command_messages
        .iter()
        .copied()
        .filter(|key| protocol.message_schemas[key].name.to_lowercase().contains(&query))
        .collect()
}

fn fields_are_valid(descriptors: &[FieldDescriptor], drafts: &HashMap<DataKey, FieldDraft>) -> bool {
    descriptors.iter().all(|descriptor| match descriptor {
        FieldDescriptor::Structure { fields, .. } => fields_are_valid(fields, drafts),
        FieldDescriptor::Field {
            field_type, data_key, ..
        } => parse_field(&drafts[data_key], field_type).is_ok(),
    })
}

fn parse_fields(
    descriptors: &[FieldDescriptor],
    drafts: &HashMap<DataKey, FieldDraft>,
) -> Result<HashMap<DataKey, DataValue>, FieldParseError> {
    let mut values = HashMap::new();
    collect_parsed_fields(descriptors, drafts, &mut values)?;
    Ok(values)
}

fn collect_parsed_fields(
    descriptors: &[FieldDescriptor],
    drafts: &HashMap<DataKey, FieldDraft>,
    values: &mut HashMap<DataKey, DataValue>,
) -> Result<(), FieldParseError> {
    for descriptor in descriptors {
        match descriptor {
            FieldDescriptor::Structure { fields, .. } => collect_parsed_fields(fields, drafts, values)?,
            FieldDescriptor::Field {
                field_type, data_key, ..
            } => {
                values.insert(*data_key, parse_field(&drafts[data_key], field_type)?);
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldParseError {
    Missing,
    Invalid(&'static str),
}

impl fmt::Display for FieldParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => formatter.write_str("Value is required"),
            Self::Invalid(err) => write!(formatter, "{err}"),
        }
    }
}

fn parse_field(draft: &FieldDraft, field_type: &DataType) -> Result<DataValue, FieldParseError> {
    if !draft.touched {
        return Err(FieldParseError::Missing);
    }
    if matches!(field_type, DataType::String) {
        return Ok(DataValue::String(draft.text.clone()));
    }

    let text = draft.text.trim();
    if text.is_empty() {
        return Err(FieldParseError::Missing);
    }

    macro_rules! parse {
        ($value_type:ty, $variant:ident, $expected:literal) => {
            text.parse::<$value_type>()
                .map(DataValue::$variant)
                .map_err(|_| FieldParseError::Invalid($expected))
        };
    }

    match field_type {
        DataType::U8 => parse!(u8, U8, "Value is out of range for unsigned 8-bit"),
        DataType::U16 => parse!(u16, U16, "Value is out of range for unsigned 16-bit"),
        DataType::U32 => parse!(u32, U32, "Value is out of range for unsigned 32-bit"),
        DataType::U64 => parse!(u64, U64, "Value is out of range for unsigned 64-bit"),
        DataType::I8 => parse!(i8, I8, "Value is out of range for signed 8-bit"),
        DataType::I16 => parse!(i16, I16, "Value is out of range for signed 16-bit"),
        DataType::I32 => parse!(i32, I32, "Value is out of range for signed 32-bit"),
        DataType::I64 => parse!(i64, I64, "Value is out of range for signed 64-bit"),
        DataType::F32 => parse!(f32, F32, "Value is out of range for single float"),
        DataType::F64 => parse!(f64, F64, "Value is out of range for double float"),
        DataType::Bool => parse!(bool, Bool, "Value must be true or false"),
        DataType::String => unreachable!("DataType::String returns before scalar parsing"),
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::dataflow::{StreamKey, protocol::SourceDescriptor, testing::message_key};

    fn draft(text: &str) -> FieldDraft {
        FieldDraft {
            text: text.into(),
            touched: true,
        }
    }

    fn command_protocol() -> ProtocolDescriptor {
        let alpha = message_key(1);
        let beta = message_key(2);
        let telemetry = message_key(3);
        ProtocolDescriptor {
            message_schemas: HashMap::from([
                (
                    alpha,
                    MessageDescriptor {
                        name: "ALPHA_TC".into(),
                        fields: Vec::new(),
                    },
                ),
                (
                    beta,
                    MessageDescriptor {
                        name: "BETA_TC".into(),
                        fields: Vec::new(),
                    },
                ),
                (
                    telemetry,
                    MessageDescriptor {
                        name: "ALPHA_TM".into(),
                        fields: Vec::new(),
                    },
                ),
            ]),
            stream_messages: vec![telemetry],
            command_messages: vec![beta, alpha],
            sources: vec![SourceDescriptor {
                name: "Target".into(),
                key: StreamKey::mock().source_key,
            }],
        }
    }

    #[test]
    fn command_filtering_is_case_insensitive_and_preserves_role_order() {
        let protocol = command_protocol();

        assert_eq!(
            filtered_command_messages(&protocol, "  tc "),
            vec![message_key(2), message_key(1)]
        );
        assert_eq!(filtered_command_messages(&protocol, "alpha"), vec![message_key(1)]);
        assert!(filtered_command_messages(&protocol, "missing").is_empty());
        assert_eq!(
            filtered_command_messages(&protocol, ""),
            vec![message_key(2), message_key(1)]
        );
    }

    #[test]
    fn untouched_and_empty_scalar_drafts_are_missing() {
        assert!(matches!(
            parse_field(&FieldDraft::default(), &DataType::U8),
            Err(FieldParseError::Missing)
        ));
        assert!(matches!(
            parse_field(&draft("  "), &DataType::Bool),
            Err(FieldParseError::Missing)
        ));
    }

    #[test]
    fn parses_every_exact_data_type() {
        assert!(matches!(
            parse_field(&draft("255"), &DataType::U8),
            Ok(DataValue::U8(255))
        ));
        assert!(matches!(
            parse_field(&draft("65535"), &DataType::U16),
            Ok(DataValue::U16(65535))
        ));
        assert!(matches!(
            parse_field(&draft(" 42 "), &DataType::U32),
            Ok(DataValue::U32(42))
        ));
        assert!(matches!(
            parse_field(&draft("42"), &DataType::U64),
            Ok(DataValue::U64(42))
        ));
        assert!(matches!(
            parse_field(&draft("-128"), &DataType::I8),
            Ok(DataValue::I8(-128))
        ));
        assert!(matches!(
            parse_field(&draft("-42"), &DataType::I16),
            Ok(DataValue::I16(-42))
        ));
        assert!(matches!(
            parse_field(&draft("-42"), &DataType::I32),
            Ok(DataValue::I32(-42))
        ));
        assert!(matches!(
            parse_field(&draft("-42"), &DataType::I64),
            Ok(DataValue::I64(-42))
        ));
        assert!(matches!(
            parse_field(&draft("1.5"), &DataType::F32),
            Ok(DataValue::F32(1.5))
        ));
        assert!(matches!(
            parse_field(&draft("2.5"), &DataType::F64),
            Ok(DataValue::F64(2.5))
        ));
        assert!(matches!(
            parse_field(&draft("true"), &DataType::Bool),
            Ok(DataValue::Bool(true))
        ));
        assert!(matches!(
            parse_field(&draft("  exact text  "), &DataType::String),
            Ok(DataValue::String(value)) if value == "  exact text  "
        ));
        assert!(matches!(
            parse_field(&draft(""), &DataType::String),
            Ok(DataValue::String(value)) if value.is_empty()
        ));
    }

    #[test]
    fn rejects_malformed_and_out_of_range_scalars() {
        assert!(matches!(
            parse_field(&draft("256"), &DataType::U8),
            Err(FieldParseError::Invalid(_))
        ));
        assert!(matches!(
            parse_field(&draft("-1"), &DataType::U16),
            Err(FieldParseError::Invalid(_))
        ));
        assert!(matches!(
            parse_field(&draft("128"), &DataType::I8),
            Err(FieldParseError::Invalid(_))
        ));
        assert!(matches!(
            parse_field(&draft("-129"), &DataType::I8),
            Err(FieldParseError::Invalid(_))
        ));
        assert!(matches!(
            parse_field(&draft("yes"), &DataType::Bool),
            Err(FieldParseError::Invalid(_))
        ));
        assert!(matches!(
            parse_field(&draft("not-a-number"), &DataType::F64),
            Err(FieldParseError::Invalid(_))
        ));
    }
}
