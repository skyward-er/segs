mod text_layout;

use std::time::Duration;

use ahash::{HashMap, HashMapExt};
use egui::{Align2, Color32, Rect, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use segs_ui::style::CtxStyleExt;
use serde::{Deserialize, Serialize};

use crate::{
    dataflow::{CommandId, CommandStatus, DataStream, StreamKey, adapter::DataAdapterInstanceToken, store::DataStore},
    ui::{
        components::{
            centered_value,
            mapping_table::{self, IntegerTextMapping},
            state_transitions::{self, StateTransition},
        },
        widget_settings::{ComboBoxOption, WidgetDataSetting, WidgetSetting},
        widgets::{WidgetRenderContext, WidgetTrait},
    },
};

/// Displays numerically ordered telemetry states and explicit directed command actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateMachineWidget {
    /// Integer telemetry stream that determines the active cell.
    stream: Option<StreamKey>,
    /// Editable state definitions, sorted at persistence boundaries.
    #[serde(deserialize_with = "mapping_table::deserialize_mappings")]
    mappings: Vec<IntegerTextMapping>,
    /// Commands bound to exact directed state pairs, including inactive pairs.
    #[serde(deserialize_with = "state_transitions::deserialize_transitions")]
    transitions: Vec<StateTransition>,
    /// Progression direction: `horizontal` or `vertical`.
    orientation: String,
    /// Opaque fill applied to the entire telemetry-selected cell.
    active_color: Color32,
    /// Whether all cells share a wrapped text size fitted to the widget.
    auto_size: bool,
    /// Fixed text size in logical points when automatic sizing is disabled.
    text_size: i64,
}

impl Default for StateMachineWidget {
    /// Returns an unconfigured horizontal progression with automatic sizing.
    fn default() -> Self {
        Self {
            stream: None,
            mappings: Vec::new(),
            transitions: Vec::new(),
            orientation: "horizontal".into(),
            active_color: Color32::from_rgb(35, 100, 190),
            auto_size: true,
            text_size: centered_value::DEFAULT_TEXT_SIZE,
        }
    }
}

impl WidgetTrait for StateMachineWidget {
    /// Renders without command authority when called outside the contextual renderer.
    fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        self.render(ui, &mut WidgetRenderContext::preview(data_store));
    }

    /// Renders telemetry and permits commands only on an authorized operator surface.
    fn show_with_context(&self, ui: &mut Ui, context: &mut WidgetRenderContext<'_>) {
        self.render(ui, context);
    }

    /// Returns the integer-only stream selector.
    fn data_settings(&mut self) -> Vec<WidgetDataSetting<'_>> {
        vec![WidgetDataSetting::integer_stream("stream", "Stream", &mut self.stream)]
    }

    /// Returns appearance controls and the shared state/transition editing section.
    fn settings(&mut self) -> Vec<WidgetSetting<'_>> {
        let show_text_size = !self.auto_size;
        const ORIENTATIONS: &[ComboBoxOption] = &[
            ComboBoxOption::new("horizontal", "Horizontal"),
            ComboBoxOption::new("vertical", "Vertical"),
        ];
        let mut settings = vec![
            WidgetSetting::combo_box("orientation", "Orientation", &mut self.orientation, ORIENTATIONS),
            WidgetSetting::color("active_color", "Active background", &mut self.active_color),
            WidgetSetting::checkbox("auto_size", "Auto size", &mut self.auto_size),
        ];
        if show_text_size {
            settings.push(WidgetSetting::integer(
                "text_size",
                "Text size",
                &mut self.text_size,
                2..=500,
                Some(1),
                Some(centered_value::TEXT_SIZE_SETTING_WIDTH),
            ));
        }
        settings.push(WidgetSetting::StateTransitions {
            id: "states",
            label: "States",
            mappings: &mut self.mappings,
            transitions: &mut self.transitions,
        });
        settings
    }

    /// Returns the name shown in the widget gallery.
    fn display_name(&self) -> &'static str {
        "State machine"
    }

    /// Returns a wide initial footprint in grid units.
    fn default_size(&self) -> Vec2 {
        vec2(6., 2.)
    }

    /// Configures isolated gallery telemetry without creating executable commands.
    fn configure_preview(&mut self, preview: &crate::dataflow::preview::PreviewContext) {
        self.stream = Some(preview.integer_stream);
        self.mappings = [("0", "INIT"), ("1", "INIT ERROR"), ("2", "OK")]
            .into_iter()
            .map(|(value, text)| IntegerTextMapping {
                value: value.into(),
                text: text.into(),
                color: None,
            })
            .collect();
    }

    /// Normalizes state drafts while retaining commands bound to inactive pairs.
    fn prepare_for_save(&mut self) {
        mapping_table::prepare_for_save(&mut self.mappings);
        state_transitions::prepare_for_save(&mut self.transitions);
    }
}

impl StateMachineWidget {
    /// Draws equal state cells and handles repeatable, validated transition requests.
    fn render(&self, ui: &mut Ui, context: &mut WidgetRenderContext<'_>) {
        // Refresh every tracked transition even when its source state is not visible
        let runtime_id = ui.id().with(("state_machine_command", context.allow_commands));
        let mut runtime = ui
            .data_mut(|data| data.remove_temp::<CommandRuntime>(runtime_id))
            .unwrap_or_default();
        let token = context.adapter.map(|adapter| adapter.token().clone());
        if runtime.adapter != token {
            runtime = CommandRuntime {
                adapter: token,
                transitions: HashMap::default(),
            };
        }
        let now = ui.input(|input| input.time);
        for feedback in runtime.transitions.values_mut() {
            let deadline = match feedback {
                TransitionFeedback::Request { latest, failure_until } => {
                    if matches!(
                        context.data_store.command_sequence(*latest).status,
                        CommandStatus::TimedOut | CommandStatus::LocalError | CommandStatus::Rejected
                    ) {
                        Some(*failure_until.get_or_insert(now + 5.))
                    } else {
                        None
                    }
                }
                TransitionFeedback::LocalError { until } => Some(*until),
            };
            if let Some(deadline) = deadline.filter(|deadline| *deadline > now) {
                ui.ctx().request_repaint_after(Duration::from_secs_f64(deadline - now));
            }
        }

        // Use the entire widget area for the equally divided progression
        let states = state_transitions::ordered_states(&self.mappings);
        if states.is_empty() {
            let (rect, _) = ui.allocate_exact_size(ui.available_size().max(Vec2::ZERO), Sense::hover());
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "No states configured",
                egui::TextStyle::Body.resolve(ui.style()),
                ui.visuals().weak_text_color(),
            );
            ui.data_mut(|data| data.insert_temp(runtime_id, runtime));
            return;
        }
        let active = self.current_value(context.data_store);
        let active_index = active
            .as_ref()
            .ok()
            .and_then(|value| states.iter().position(|state| state.0 == *value));

        // Index first-occurrence outgoing commands once instead of scanning per destination
        let mut outgoing = HashMap::new();
        if let Some(index) = active_index {
            let source = states[index].0;
            for transition in &self.transitions {
                if transition.from == source && transition.to != source {
                    outgoing.entry(transition.to).or_insert(transition);
                }
            }
        }
        let vertical = self.orientation == "vertical";
        let (bounds, _) = ui.allocate_exact_size(ui.available_size().max(Vec2::ZERO), Sense::hover());
        let count = states.len() as f32;
        let cell_size = if vertical {
            vec2(bounds.width(), bounds.height() / count)
        } else {
            vec2(bounds.width() / count, bounds.height())
        };
        let inset = vec2(2., 2.);
        let text_area = (cell_size - inset * 2.).max(Vec2::ZERO);
        let labels = text_layout::layout(ui, &states, text_area, self.auto_size, self.text_size);
        let painter = ui.painter().with_clip_rect(bounds.intersect(ui.clip_rect()));
        let separator = Stroke::new(1., ui.visuals().widgets.noninteractive.bg_stroke.color);

        // Paint each state and overlay its incoming control only when the active source allows it
        for (index, galley) in labels.iter().enumerate() {
            let offset = if vertical {
                vec2(0., cell_size.y * index as f32)
            } else {
                vec2(cell_size.x * index as f32, 0.)
            };
            let cell = Rect::from_min_size(bounds.min + offset, cell_size);
            let selected = active_index == Some(index);
            let foreground = if selected {
                let fill = self.active_color.to_opaque();
                painter.rect_filled(cell, 0., fill);
                centered_value::contrasting_text(fill)
            } else {
                ui.visuals().text_color()
            };
            if let Some(transition) = outgoing.get(&states[index].0) {
                show_transition(ui, context, &mut runtime, transition, cell, vertical, foreground);
            }

            // Keep labels and separators above transition backgrounds
            let text_rect = Rect::from_center_size(cell.center(), text_area);
            let text_pos = pos2(cell.center().x, cell.center().y - galley.size().y * 0.5);
            painter
                .with_clip_rect(text_rect.intersect(painter.clip_rect()))
                .galley_with_override_text_color(text_pos, galley.clone(), foreground);

            // Divide cells at their shared boundary without consuming layout space
            if index > 0 {
                let line = if vertical {
                    [cell.left_top(), cell.right_top()]
                } else {
                    [cell.left_top(), cell.left_bottom()]
                };
                painter.line_segment(line, separator);
            }
        }
        ui.data_mut(|data| data.insert_temp(runtime_id, runtime));
    }

    /// Returns the latest integer state, or a user-facing reason telemetry is unavailable.
    fn current_value(&self, store: &DataStore) -> Result<i64, String> {
        let key = self.stream.ok_or("No stream")?;
        match store.stream(key).ok_or("No data")? {
            DataStream::I64(points) => points.last().map(|point| point.value).ok_or_else(|| "No data".into()),
            _ => Err("Expected integer stream".into()),
        }
    }
}

/// Draws one stable clickable control and enqueues a fresh request for every eligible click.
/// Pending status changes only its appearance and never its interaction eligibility.
fn show_transition(
    ui: &mut Ui,
    context: &mut WidgetRenderContext<'_>,
    runtime: &mut CommandRuntime,
    transition: &StateTransition,
    rect: Rect,
    vertical: bool,
    foreground: Color32,
) {
    // Check only selections and command authority until the destination is clicked
    let pair = (transition.from, transition.to);
    let enabled = context.allow_commands && context.adapter.is_some() && transition.command.is_configured();
    let now = ui.input(|input| input.time);

    // Follow only the newest attempt so older requests cannot overwrite local feedback
    let (pending, tint) = match runtime.transitions.get(&pair) {
        Some(TransitionFeedback::Request { latest, failure_until }) => {
            let status = &context.data_store.command_sequence(*latest).status;
            let tint = if failure_until.is_some_and(|deadline| now < deadline) {
                match status {
                    CommandStatus::TimedOut => Some(ui.app_style().timeout_fill),
                    CommandStatus::LocalError => Some(ui.app_style().local_error_fill),
                    CommandStatus::Rejected => Some(ui.app_style().error_fill),
                    CommandStatus::Pending | CommandStatus::Success => None,
                }
            } else {
                None
            };
            (matches!(status, CommandStatus::Pending), tint)
        }
        Some(TransitionFeedback::LocalError { until }) => {
            (false, (now < *until).then(|| ui.app_style().local_error_fill))
        }
        None => (false, None),
    };

    // Keep the whole destination clickable without reserving space for an icon
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            let response = ui.interact(rect, ui.id().with(("transition", pair)), Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    ui.is_enabled(),
                    format!("{} → {}", pair.0, pair.1),
                )
            });
            let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
            if let Some(color) = tint {
                painter.rect_filled(rect, 0., color.gamma_multiply(0.25));
            }
            if response.hovered() || response.is_pointer_button_down_on() || response.has_focus() {
                let alpha = if response.is_pointer_button_down_on() { 36 } else { 18 };
                painter.rect_filled(
                    rect,
                    0.,
                    Color32::from_rgba_unmultiplied(foreground.r(), foreground.g(), foreground.b(), alpha),
                );
                if response.has_focus() {
                    painter.rect_stroke(rect, 0., Stroke::new(1., foreground), StrokeKind::Inside);
                }
            }

            // Keep request feedback on the clicked destination while its source remains active
            if pending && rect.width() > 4. && rect.height() > 4. {
                let edge = if vertical {
                    let x = rect.right() - 2.;
                    [pos2(x, rect.top() + 2.), pos2(x, rect.bottom() - 2.)]
                } else {
                    let y = rect.bottom() - 2.;
                    [pos2(rect.left() + 2., y), pos2(rect.right() - 2., y)]
                };
                ui.ctx().request_repaint();
                let start = ((now * 4.).sin() as f32 + 1.) * 0.35;
                let delta = edge[1] - edge[0];
                let line = [edge[0] + delta * start, edge[0] + delta * (start + 0.3)];
                painter.line_segment(line, Stroke::new(2., foreground));
            }
            response
        })
        .inner;

    // Format command details only when an actionable destination's tooltip is actually displayed
    if enabled && let Some(adapter) = context.adapter {
        response.clone().on_hover_ui(|ui| {
            ui.label(transition.command.summary(adapter.describe_protocol()));
        });
    }

    // Replace feedback for this directed pair while leaving older queued requests untouched
    if enabled
        && response.clicked()
        && let Some(adapter) = context.adapter
    {
        let feedback = match transition.command.resolve(adapter.describe_protocol()) {
            Ok(command) => TransitionFeedback::Request {
                latest: context.data_store.enqueue_command(command),
                failure_until: None,
            },
            Err(_) => TransitionFeedback::LocalError { until: now + 5. },
        };
        runtime.transitions.insert(pair, feedback);
        ui.ctx().request_repaint();
    }
}

/// Per-transition request feedback scoped to one adapter lifecycle, never persisted.
#[derive(Clone, Default)]
struct CommandRuntime {
    /// Adapter identity owning every stored command sequence.
    adapter: Option<DataAdapterInstanceToken>,
    /// Latest attempted transition for each exact directed state pair.
    transitions: HashMap<(i64, i64), TransitionFeedback>,
}

/// Feedback for the newest attempt, independent of older requests still in flight.
#[derive(Clone)]
enum TransitionFeedback {
    /// A successfully resolved command tracked through its datastore sequence.
    Request {
        /// Latest datastore-issued sequence identifier for this directed pair.
        latest: CommandId,
        /// Tint expiration in egui monotonic seconds, set when a terminal failure is first observed.
        failure_until: Option<f64>,
    },
    /// A resolution failure that did not create a datastore command.
    LocalError {
        /// Tint expiration in egui monotonic seconds, measured from the failed click.
        until: f64,
    },
}
