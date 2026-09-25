use crate::dataflow::{
    DataKey, DataType, DataValue, MessageKey, SourceKey,
    adapter::{DataAdapterInstance, DataAdapterInstanceToken},
    protocol::{EnumDescriptor, FieldDescriptor},
};
use ahash::{HashMap, HashMapExt};
use egui::{Id, Response, RichText, Ui};
use segs_ui::widgets::{SearchableComboBox, SearchableComboBoxList, SingleSelection, text::ValidationTextEdit};
use std::{fmt, sync::Arc};

/// Transient scalar or enum editor input initialized from a message descriptor.
#[derive(Clone, Default)]
pub struct FieldDraft {
    /// Raw scalar text, preserving whitespace for string payloads.
    pub text: String,
    /// Whether a scalar was explicitly entered, including an empty string.
    pub touched: bool,
    /// Selected index within the current descriptor, or no selection.
    pub selected_enum_variant: Option<usize>,
    /// Searchable enum choices derived from the current descriptor.
    pub enum_choices: Option<SearchableComboBoxList<usize>>,
}

/// Returns shared target and command choices for the installed adapter.
/// Keeps one catalog in the context, replacing it when the adapter lifecycle changes.
/// Protocol metadata must remain unchanged within an adapter lifecycle.
pub fn protocol_choices(ctx: &egui::Context, adapter: &DataAdapterInstance) -> Arc<ProtocolChoices> {
    // Share immutable catalogs across panels without sharing selector interaction state
    let id = Id::new("command_editor_protocol_choices");
    if let Some(choices) = ctx.data(|data| data.get_temp::<Arc<ProtocolChoices>>(id))
        && choices.adapter == *adapter.token()
    {
        return choices;
    }

    // Build searchable indexes only when no catalog belongs to the installed adapter
    let protocol = adapter.describe_protocol();
    let choices = Arc::new(ProtocolChoices {
        adapter: adapter.token().clone(),
        targets: SearchableComboBoxList::new(protocol.sources.iter().map(|source| (source.key, &source.name))),
        commands: SearchableComboBoxList::new(
            protocol
                .command_messages
                .iter()
                .filter_map(|key| protocol.message_schemas.get(key).map(|message| (*key, &message.name))),
        ),
    });
    ctx.data_mut(|data| data.insert_temp(id, choices.clone()));
    choices
}

/// Returns the response from editing a target selection using shared command UI text.
pub fn target_selector(
    ui: &mut Ui,
    id: Id,
    choices: &SearchableComboBoxList<SourceKey>,
    target: &mut Option<SourceKey>,
) -> Response {
    ui.add_enabled_ui(!choices.is_empty(), |ui| {
        ui.add(
            SearchableComboBox::new(id, choices, SingleSelection::new(target))
                .empty_selection_text("Select a target")
                .search_hint("Search targets…")
                .empty_results_text("No matching targets."),
        )
    })
    .inner
}

/// Returns the response from editing a command selection using shared command UI text.
pub fn message_selector(
    ui: &mut Ui,
    id: Id,
    choices: &SearchableComboBoxList<MessageKey>,
    message: &mut Option<MessageKey>,
) -> Response {
    ui.add(
        SearchableComboBox::new(id, choices, SingleSelection::new(message))
            .empty_selection_text("Select a command")
            .max_visible_rows(8)
            .search_hint("Search command…")
            .empty_results_text("No matching commands."),
    )
}

/// Edits initialized field drafts in protocol order, including nested structures.
/// Panics if drafts were not initialized for these descriptors.
pub fn show_field_editors(
    ui: &mut Ui,
    descriptors: &[FieldDescriptor],
    drafts: &mut HashMap<DataKey, FieldDraft>,
    validate_blur: bool,
) {
    // Keep widget identities tied to stable form positions rather than command-specific keys
    for (descriptor_index, descriptor) in descriptors.iter().enumerate() {
        ui.push_id(("command_field_descriptor", descriptor_index), |ui| match descriptor {
            FieldDescriptor::Structure { name, fields } => {
                ui.add(egui::Label::new(RichText::new(name).strong()).truncate());
                ui.indent("children", |ui| {
                    show_field_editors(ui, fields, drafts, validate_blur);
                });
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

                ui.add(egui::Label::new(RichText::new(format!("{name} · {field_type}")).size(11.)).truncate());
                let mut editor = ValidationTextEdit::new(&mut draft.text)
                    .id_salt("editor")
                    .desired_width(ui.available_width());
                if let Some(error) = error {
                    editor = editor.error(error);
                }
                let response = ui.add(editor);
                if response.changed() || (validate_blur && response.lost_focus()) {
                    draft.touched = true;
                }
            }
            FieldDescriptor::EnumField {
                name,
                descriptor,
                data_key,
            } => {
                let draft = drafts
                    .get_mut(data_key)
                    .expect("Selected message fields must have initialized drafts");
                let FieldDraft {
                    selected_enum_variant,
                    enum_choices,
                    ..
                } = draft;
                let choices = enum_choices
                    .as_ref()
                    .expect("Enum field drafts must contain searchable choices");

                ui.add(egui::Label::new(RichText::new(format!("{name} · {}", descriptor.name)).size(11.)).truncate());
                ui.add(
                    SearchableComboBox::new(
                        ui.make_persistent_id("enum_selector"),
                        choices,
                        SingleSelection::new(selected_enum_variant),
                    )
                    .empty_selection_text("Select a value")
                    .search_hint("Search enum values…")
                    .empty_results_text("No matching enum values."),
                );
            }
        });
    }
}

/// Inserts empty drafts and enum choices for every descriptor leaf, replacing matching entries.
pub fn initialize_drafts(descriptors: &[FieldDescriptor], drafts: &mut HashMap<DataKey, FieldDraft>) {
    for descriptor in descriptors {
        match descriptor {
            FieldDescriptor::Structure { fields, .. } => initialize_drafts(fields, drafts),
            FieldDescriptor::Field { data_key, .. } => {
                drafts.insert(*data_key, FieldDraft::default());
            }
            FieldDescriptor::EnumField {
                descriptor, data_key, ..
            } => {
                drafts.insert(
                    *data_key,
                    FieldDraft {
                        enum_choices: Some(SearchableComboBoxList::new(
                            descriptor
                                .variants
                                .iter()
                                .enumerate()
                                .map(|(index, (name, _))| (index, name)),
                        )),
                        ..FieldDraft::default()
                    },
                );
            }
        }
    }
}

/// Returns whether every initialized draft parses into its exact protocol type.
/// Panics if a descriptor has no initialized draft.
pub fn fields_are_valid(descriptors: &[FieldDescriptor], drafts: &HashMap<DataKey, FieldDraft>) -> bool {
    descriptors.iter().all(|descriptor| match descriptor {
        FieldDescriptor::Structure { fields, .. } => fields_are_valid(fields, drafts),
        FieldDescriptor::Field {
            field_type, data_key, ..
        } => parse_field(&drafts[data_key], field_type).is_ok(),
        FieldDescriptor::EnumField {
            descriptor, data_key, ..
        } => parse_enum_field(&drafts[data_key], descriptor).is_ok(),
    })
}

/// Returns exact typed payload values or the first missing, malformed, or out-of-range field error.
/// Panics if a descriptor has no initialized draft.
pub fn parse_fields(
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
            FieldDescriptor::EnumField {
                descriptor, data_key, ..
            } => {
                values.insert(*data_key, parse_enum_field(&drafts[data_key], descriptor)?);
            }
        }
    }
    Ok(())
}

fn parse_enum_field(draft: &FieldDraft, descriptor: &EnumDescriptor) -> Result<DataValue, FieldParseError> {
    let index = draft.selected_enum_variant.ok_or(FieldParseError::Missing)?;
    descriptor
        .variants
        .get(index)
        .map(|(_, value)| value.clone())
        .ok_or(FieldParseError::Invalid("Selected enum value is unavailable"))
}

/// Reason a command parameter cannot be transmitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldParseError {
    /// No scalar input or enum selection was provided.
    Missing,
    /// The input violates its protocol type, with a user-facing explanation.
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

/// Immutable protocol catalogs shared by every command editor in one egui context.
pub struct ProtocolChoices {
    /// Lifecycle identity used to invalidate catalogs even for an identical replacement protocol.
    adapter: DataAdapterInstanceToken,
    /// Selectable targets in protocol order.
    pub targets: SearchableComboBoxList<SourceKey>,
    /// Selectable commands with available schemas in protocol order.
    pub commands: SearchableComboBoxList<MessageKey>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(text: &str) -> FieldDraft {
        FieldDraft {
            text: text.into(),
            touched: true,
            selected_enum_variant: None,
            enum_choices: None,
        }
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
