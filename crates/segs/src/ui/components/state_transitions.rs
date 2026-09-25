use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
    time::SystemTime,
};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use egui::{
    Align, Label, Layout, RichText, Sense, TextStyle, TextWrapMode, Ui, WidgetText,
    collapsing_header::{CollapsingState, paint_default_icon},
    pos2, vec2,
};
use segs_ui::{
    containers::Card,
    widgets::{SearchableComboBox, SearchableComboBoxList, SingleSelection},
};
use serde::{Deserialize, Deserializer, Serialize};

use crate::dataflow::{
    Command, DataKey, MessageKey, SourceKey,
    adapter::DataAdapterInstance,
    protocol::{FieldDescriptor, ProtocolDescriptor},
};

use super::{
    command_editor::{self, FieldDraft},
    mapping_table::IntegerTextMapping,
};

/// A command bound to an exact directed pair of integer states.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateTransition {
    /// Integer state from which the command may be sent.
    pub from: i64,
    /// Destination integer state the command requests.
    pub to: i64,
    /// Editable command configuration, including unfinished parameter drafts.
    pub command: CommandPreset,
}

/// Persisted command inputs resolved against the active protocol before sending.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommandPreset {
    /// Target source, or `None` until selected.
    target: Option<SourceKey>,
    /// Command message, or `None` until selected.
    message: Option<MessageKey>,
    /// Explicitly entered field values; absent entries are unfinished inputs.
    fields: HashMap<DataKey, ParameterInput>,
}

impl CommandPreset {
    /// Returns whether a target and command are selected, without validating their inputs or availability.
    pub fn is_configured(&self) -> bool {
        self.target.is_some() && self.message.is_some()
    }

    /// Builds a fresh command, or returns the reason its configuration cannot send.
    /// The returned command has the current wall-clock timestamp and is not enqueued.
    pub fn resolve(&self, protocol: &ProtocolDescriptor) -> Result<Command, String> {
        // Resolve only targets and messages advertised by the active adapter
        let target = self.target.ok_or("Select a target")?;
        if !protocol.sources.iter().any(|source| source.key == target) {
            return Err("Configured target is unavailable".into());
        }
        let key = self.message.ok_or("Select a command")?;
        if !protocol.command_messages.contains(&key) {
            return Err("Configured command is unavailable".into());
        }
        let descriptor = protocol
            .message_schemas
            .get(&key)
            .ok_or("Command schema is unavailable")?;
        let drafts = self.drafts(&descriptor.fields);
        let fields = command_editor::parse_fields(&descriptor.fields, &drafts).map_err(|error| error.to_string())?;
        if self.fields.keys().any(|key| !fields.contains_key(key)) {
            return Err("Command fields changed; remove and configure this command again".into());
        }
        Ok(Command {
            key,
            target,
            timestamp: SystemTime::now(),
            fields,
        })
    }

    /// Returns command, target, and parameter details for an actionable transition's tooltip.
    /// Call only when displaying the tooltip to avoid formatting unchanged commands each frame.
    pub fn summary(&self, protocol: &ProtocolDescriptor) -> String {
        // Resolve display names without changing the configured command inputs
        let target = protocol
            .sources
            .iter()
            .find(|source| Some(source.key) == self.target)
            .map_or("Unavailable target", |source| source.name.as_str());
        let descriptor = self.message.and_then(|key| protocol.message_schemas.get(&key));
        let mut summary = format!(
            "Command: {}\nTarget: {target}",
            descriptor.map_or("Unconfigured", |message| message.name.as_str())
        );
        if let Some(descriptor) = descriptor {
            visit_fields(&descriptor.fields, &mut |key, name, _| {
                let value = match self.fields.get(&key) {
                    Some(ParameterInput::Scalar(value) | ParameterInput::EnumVariant(value)) => value.as_str(),
                    None => "Not set",
                };
                summary.push_str(&format!("\n{name}: {value}"));
            });
        }
        summary
    }

    /// Returns editor drafts with enum names resolved against the current descriptors.
    fn drafts(&self, descriptors: &[FieldDescriptor]) -> HashMap<DataKey, FieldDraft> {
        let mut drafts = HashMap::new();
        command_editor::initialize_drafts(descriptors, &mut drafts);
        visit_fields(descriptors, &mut |key, _, descriptor| {
            let draft = drafts.get_mut(&key).expect("initialized field draft");
            match (self.fields.get(&key), descriptor) {
                (Some(ParameterInput::Scalar(text)), FieldDescriptor::Field { .. }) => {
                    draft.text.clone_from(text);
                    draft.touched = true;
                }
                (Some(ParameterInput::EnumVariant(name)), FieldDescriptor::EnumField { descriptor, .. }) => {
                    draft.selected_enum_variant = descriptor.variants.iter().position(|(variant, _)| variant == name);
                }
                _ => {}
            }
        });
        drafts
    }

    /// Edits a preset without sending it, preserving incomplete values across saves.
    fn show(&mut self, ui: &mut Ui, adapter: &DataAdapterInstance) {
        // Reuse searchable selectors and descriptor-driven field editors
        let protocol = adapter.describe_protocol();
        let choices = command_editor::protocol_choices(ui.ctx(), adapter);
        ui.label("Target");
        command_editor::target_selector(ui, ui.id().with("target"), &choices.targets, &mut self.target);
        ui.label("Command");
        if command_editor::message_selector(ui, ui.id().with("command"), &choices.commands, &mut self.message).changed()
        {
            self.fields.clear();
        }
        if let Some(message) = self.message.and_then(|key| protocol.message_schemas.get(&key)) {
            let mut drafts = self.drafts(&message.fields);
            ui.push_id(self.message, |ui| {
                command_editor::show_field_editors(ui, &message.fields, &mut drafts, true)
            });
            visit_fields(&message.fields, &mut |key, _, descriptor| {
                let draft = &drafts[&key];
                let input = match descriptor {
                    FieldDescriptor::Field { .. } if draft.touched => Some(ParameterInput::Scalar(draft.text.clone())),
                    FieldDescriptor::EnumField { descriptor, .. } => draft
                        .selected_enum_variant
                        .and_then(|index| descriptor.variants.get(index))
                        .map(|(name, _)| ParameterInput::EnumVariant(name.clone())),
                    _ => None,
                };
                if let Some(input) = input {
                    self.fields.insert(key, input);
                }
            });
        }
        if let Err(reason) = self.resolve(protocol) {
            ui.add(Label::new(RichText::new(reason).color(ui.visuals().error_fg_color)).wrap());
        }
    }
}

/// Edits outgoing commands grouped by source state while retaining inactive records.
pub fn show(
    ui: &mut Ui,
    mappings: &[IntegerTextMapping],
    transitions: &mut Vec<StateTransition>,
    adapter: Option<&DataAdapterInstance>,
) {
    let states = ordered_states(mappings);
    let mut remove = None;
    let mut add = None;

    // Index endpoints and group each first-occurrence command once in insertion order
    let mut state_indices = HashMap::with_capacity(states.len());
    let mut hasher = DefaultHasher::new();
    for (index, state) in states.iter().enumerate() {
        state_indices.insert(state.0, index);
        state.hash(&mut hasher);
    }
    let mut outgoing = vec![Vec::new(); states.len()];
    let mut configured = HashSet::with_capacity(transitions.len());
    let mut inactive = Vec::new();
    for (index, transition) in transitions.iter().enumerate() {
        let Some(&source_index) = state_indices.get(&transition.from) else {
            inactive.push((index, "Source state is missing"));
            continue;
        };
        if !state_indices.contains_key(&transition.to) {
            inactive.push((index, "Destination state is missing"));
        } else if transition.from == transition.to {
            inactive.push((index, "Self-transitions are not supported"));
        } else if !configured.insert((transition.from, transition.to)) {
            inactive.push((index, "Duplicate transition"));
        } else {
            outgoing[source_index].push(index);
        }
    }

    // Reuse destination lists until the available state values or names change
    let cache_id = ui.id().with("destination_choices");
    let fingerprint = hasher.finish();
    let mut cache = ui
        .data_mut(|data| data.remove_temp::<DestinationChoices>(cache_id))
        .unwrap_or_default();
    if cache.fingerprint != fingerprint {
        cache.fingerprint = fingerprint;
        cache.sources.clear();
    }

    // Render one expander per source and build choices only for expanded entries
    for (source_index, &(from, name)) in states.iter().enumerate() {
        ui.push_id(("source", from), |ui| {
            let width = ui.available_width().max(0.);
            let (mut header, inset) = state_header(ui, name, outgoing[source_index].len());
            header.show_body_unindented(ui, |ui| {
                // Let fields grow vertically while keeping their text within the panel
                ui.set_max_width(ui.available_width().min(width).max(0.));
                ui.style_mut().wrap_mode = Some(TextWrapMode::Wrap);
                // Align the card border with the caret instead of the header text
                ui.horizontal(|ui| {
                    ui.add_space(inset);
                    ui.vertical(|ui| {
                        let choices = cache.sources.entry(from).or_insert_with(|| {
                            Arc::new(SearchableComboBoxList::new(
                                states.iter().filter(|state| state.0 != from).copied(),
                            ))
                        });
                        if let Some(to) = destination_picker(ui, from, choices, &configured) {
                            add = Some(StateTransition {
                                from,
                                to,
                                command: CommandPreset::default(),
                            });
                        }
                        if !outgoing[source_index].is_empty() {
                            ui.add_space(3.);
                        }
                        for &index in &outgoing[source_index] {
                            let transition = &mut transitions[index];
                            let destination = states[state_indices[&transition.to]].1;
                            ui.push_id((from, transition.to), |ui| {
                                if transition_card(ui, name, destination, &mut transition.command, adapter) {
                                    remove = Some(index);
                                }
                            });
                        }
                    });
                });
            });
        });
    }
    ui.data_mut(|data| data.insert_temp(cache_id, cache));
    if states.len() < 2 {
        ui.weak("Add at least two valid states to configure transitions.");
    }

    // Retain missing endpoints and unsupported records without silently retargeting commands
    if !inactive.is_empty() {
        ui.strong("Inactive transitions");
    }
    for (index, reason) in inactive {
        let transition = &transitions[index];
        // Reserve the delete action before truncating the inactive transition description
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("Delete").clicked() {
                remove = Some(index);
            }
            let size = vec2(ui.available_width().max(0.), ui.spacing().interact_size.y);
            ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
                ui.add(
                    Label::new(format!("{} → {}: {reason}", transition.from, transition.to))
                        .halign(Align::Min)
                        .selectable(false)
                        .show_tooltip_when_elided(false)
                        .truncate(),
                )
            });
        });
    }
    // Defer structural edits until all indexed rows have finished rendering
    if let Some(index) = remove {
        transitions.remove(index);
    }
    if let Some(transition) = add {
        transitions.push(transition);
    }
}

/// Sorts commands stably by source and destination integers without dropping any records.
/// Call only at persistence boundaries so active edits keep their positions.
pub fn prepare_for_save(transitions: &mut [StateTransition]) {
    transitions.sort_by_key(|transition| (transition.from, transition.to));
}

/// Returns transitions sorted by source and destination, or an error for malformed wire data.
pub fn deserialize_transitions<'de, D>(deserializer: D) -> Result<Vec<StateTransition>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut transitions = Vec::<StateTransition>::deserialize(deserializer)?;
    prepare_for_save(&mut transitions);
    Ok(transitions)
}

/// Returns valid first-occurrence mappings in ascending numeric order.
/// Each tuple contains the parsed integer and its borrowed display text.
pub fn ordered_states(mappings: &[IntegerTextMapping]) -> Vec<(i64, &str)> {
    let mut states: Vec<_> = mappings
        .iter()
        .filter_map(|mapping| mapping.parsed_value().map(|value| (value, mapping.text.as_str())))
        .collect();
    states.sort_by_key(|state| state.0);
    states.dedup_by_key(|state| state.0);
    states
}

/// Returns the destination requested by Add, or `None` when no new transition was requested.
/// Keeps selection transient and rejects destinations already configured for this source.
fn destination_picker(
    ui: &mut Ui,
    from: i64,
    choices: &SearchableComboBoxList<i64>,
    configured: &HashSet<(i64, i64)>,
) -> Option<i64> {
    // Reset stale selections while preserving valid picks across ordinary frames
    let id = ui.id().with("destination");
    let mut selected = ui.data(|data| data.get_temp::<i64>(id));
    if selected.is_some_and(|value| choices.label_for(&value).is_none()) {
        selected = None;
    }
    // Reserve a compact add button beside the selector without widening the panel
    let mut added = None;
    let mut duplicate = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.;
        let size = ui.spacing().interact_size.y;
        let selector_width = (ui.available_width() - size - ui.spacing().item_spacing.x).max(0.);
        ui.allocate_ui_with_layout(vec2(selector_width, size), Layout::left_to_right(Align::Center), |ui| {
            ui.add_enabled_ui(!choices.is_empty(), |ui| {
                ui.add(
                    SearchableComboBox::new(id, choices, SingleSelection::new(&mut selected))
                        .empty_selection_text("Select a destination")
                        .search_hint("Search states…")
                        .empty_results_text("No matching states."),
                );
            });
        });
        duplicate = selected.is_some_and(|to| configured.contains(&(from, to)));
        if ui
            .add_enabled_ui(selected.is_some() && !duplicate, |ui| {
                ui.add_sized(vec2(size, size), egui::Button::new("+"))
            })
            .inner
            .on_hover_text("Add transition")
            .on_disabled_hover_text("Add transition")
            .clicked()
        {
            added = selected.take();
        }
    });

    // Explain unavailable actions without hiding any destination from the selector
    if duplicate {
        ui.weak("Transition already configured");
    }
    if choices.is_empty() {
        ui.weak("Add another valid state to configure transitions.");
    }
    ui.data_mut(|data| {
        if let Some(selected) = selected {
            data.insert_temp(id, selected);
        } else {
            data.remove::<i64>(id);
        }
    });
    added
}

/// Edits one directed command in a bounded card and returns whether removal was requested.
fn transition_card(
    ui: &mut Ui,
    from: &str,
    to: &str,
    command: &mut CommandPreset,
    adapter: Option<&DataAdapterInstance>,
) -> bool {
    Card::new()
        .show(ui, |ui| {
            // Fill only the content area left after the card reserves its own padding
            ui.take_available_width();
            ui.allocate_ui_with_layout(
                vec2(ui.available_width().max(0.), ui.spacing().interact_size.y),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.add(
                        Label::new(RichText::new(format!("{from} → {to}")).strong())
                            .halign(Align::Min)
                            .selectable(false)
                            .show_tooltip_when_elided(false)
                            .truncate(),
                    )
                },
            );
            ui.add_space(3.);

            // Preserve the command when no adapter is available to describe its fields
            if let Some(adapter) = adapter {
                ui.push_id(adapter.token(), |ui| command.show(ui, adapter));
            } else {
                ui.weak("Connect a data source to edit this command.");
            }
            ui.add_space(3.);
            ui.button("Remove command").clicked()
        })
        .inner
}

/// Draws a single clickable state row and returns its collapse state and caret inset in points.
/// Text is truncated without selection or hover tooltips, and clicking anywhere toggles the body.
fn state_header(ui: &mut Ui, title: &str, count: usize) -> (CollapsingState, f32) {
    // Use one interaction across the caret, title, count and remaining row space
    let id = ui.id().with("header");
    let mut state = CollapsingState::load_with_default_open(ui.ctx(), id, false);
    let (_, rect) = ui.allocate_space(vec2(ui.available_width().max(0.), ui.spacing().interact_size.y));
    let response = ui.interact(rect, id, Sense::click());
    if response.clicked() {
        state.toggle(ui);
    }
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::CollapsingHeader, ui.is_enabled(), title));
    let visuals = ui.style().interact(&response);
    let color = visuals.text_color();
    if response.hovered() || response.has_focus() || response.is_pointer_button_down_on() {
        ui.painter()
            .rect_filled(rect, visuals.corner_radius, visuals.weak_bg_fill);
    }

    // Paint the caret and reserve the right-hand preset count before truncating the title
    let (mut icon_rect, _) = ui.spacing().icon_rectangles(rect);
    icon_rect.set_center(pos2(rect.left() + ui.spacing().indent * 0.5, rect.center().y));
    paint_default_icon(ui, state.openness(ui.ctx()), &response.with_new_rect(icon_rect));
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let count = WidgetText::from(count.to_string()).into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        rect.width(),
        TextStyle::Body,
    );
    let count_x = rect.right() - count.size().x;
    painter.galley(pos2(count_x, rect.center().y - count.size().y * 0.5), count, color);
    let title_x = rect.left() + ui.spacing().indent + ui.spacing().item_spacing.x;
    let title = WidgetText::from(title).into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        (count_x - ui.spacing().item_spacing.x - title_x).max(0.),
        TextStyle::Body,
    );
    painter.galley(pos2(title_x, rect.center().y - title.size().y * 0.5), title, color);
    (state, (icon_rect.left() - rect.left()).max(0.))
}

/// Visits leaf fields while preserving protocol order and nested editor structure.
fn visit_fields(descriptors: &[FieldDescriptor], visit: &mut impl FnMut(DataKey, &str, &FieldDescriptor)) {
    for descriptor in descriptors {
        match descriptor {
            FieldDescriptor::Structure { fields, .. } => visit_fields(fields, visit),
            FieldDescriptor::Field { data_key, name, .. } | FieldDescriptor::EnumField { data_key, name, .. } => {
                visit(*data_key, name, descriptor)
            }
        }
    }
}

/// Lazily built destination choices scoped to one widget's current state definitions.
#[derive(Clone, Default)]
struct DestinationChoices {
    /// Fingerprint of ordered numeric state values and displayed names.
    fingerprint: u64,
    /// Shared lists excluding their own source, created only when a source is expanded.
    sources: HashMap<i64, Arc<SearchableComboBoxList<i64>>>,
}

/// Exact editable input rather than an enum index or a lossy formatted runtime value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum ParameterInput {
    /// Scalar text parsed into the protocol's exact field type before transmission.
    Scalar(String),
    /// Named enum variant resolved against the active protocol.
    EnumVariant(String),
}
