use std::time::Duration;

use egui::{Button, Color32, RichText, Ui, Vec2, vec2};
use segs_ui::style::CtxStyleExt;
use serde::{Deserialize, Serialize};

use crate::{
    dataflow::{
        CommandId, CommandStatus, adapter::DataAdapterInstanceToken, preview::PreviewContext, store::DataStore,
    },
    ui::{
        components::{centered_value, state_transitions::CommandPreset},
        widget_settings::{WidgetDataSetting, WidgetSetting},
        widgets::{WidgetRenderContext, WidgetTrait},
    },
};

/// Seconds a failed request keeps tinting the button.
const FAILURE_TINT_SECONDS: f64 = 5.;
/// Seconds the button face stays darkened after sending, fading back to its color.
const SEND_FLASH_SECONDS: f64 = 0.3;
/// Fraction of the way toward black the face is darkened right after sending.
const SEND_FLASH_DARKENING: f32 = 0.3;
/// Gap in logical points kept between the button face and the grid cell border.
const BUTTON_INSET: i8 = 4;
/// Corner radius of the button face in logical points.
const BUTTON_CORNER_RADIUS: u8 = 4;

/// A labeled button that sends one configured command on every click.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonWidget {
    /// Text shown on the button face.
    label: String,
    /// Face color shown while no failure tint is active, always drawn opaque.
    color: Color32,
    /// Command sent when the button is clicked, resolved against the active protocol.
    command: CommandPreset,
}

impl Default for ButtonWidget {
    /// Returns an unconfigured button labeled "Send".
    fn default() -> Self {
        Self {
            label: "Send".into(),
            color: Color32::from_rgb(35, 100, 190),
            command: CommandPreset::default(),
        }
    }
}

impl WidgetTrait for ButtonWidget {
    /// Renders a disabled preview that never sends commands.
    fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        self.render(ui, &mut WidgetRenderContext::preview(data_store));
    }

    /// Renders the button, sending its command on click when the context allows commands.
    fn show_with_context(&self, ui: &mut Ui, context: &mut WidgetRenderContext<'_>) {
        self.render(ui, context);
    }

    /// Returns no data settings because the button does not consume streams.
    fn data_settings(&mut self) -> Vec<WidgetDataSetting<'_>> {
        Vec::new()
    }

    /// Returns the label, color, and command editors.
    fn settings(&mut self) -> Vec<WidgetSetting<'_>> {
        vec![
            WidgetSetting::text_box("label", "Label", &mut self.label),
            WidgetSetting::color("color", "Color", &mut self.color),
            WidgetSetting::command("command", "Command", &mut self.command),
        ]
    }

    /// Returns the name shown in the widget gallery.
    fn display_name(&self) -> &'static str {
        "Button"
    }

    /// Returns a compact initial footprint in grid units.
    fn default_size(&self) -> Vec2 {
        vec2(2., 1.)
    }

    /// Leaves the gallery preview unbound so it never creates executable commands.
    fn configure_preview(&mut self, _preview: &PreviewContext) {}
}

impl ButtonWidget {
    /// Draws the button, tints it with the latest request outcome, and enqueues a command per click.
    ///
    /// The face briefly darkens after each sent command.
    fn render(&self, ui: &mut Ui, context: &mut WidgetRenderContext<'_>) {
        // Load feedback for the latest request, discarding it when the adapter changed
        let runtime_id = ui.id().with(("button_command", context.allow_commands));
        let token = context.adapter.map(|adapter| adapter.token().clone());
        let mut runtime = ui
            .data_mut(|data| data.remove_temp::<CommandRuntime>(runtime_id))
            .filter(|runtime| runtime.adapter == token)
            .unwrap_or_else(|| CommandRuntime {
                adapter: token,
                ..Default::default()
            });

        // Find the latest failure tint and when it expires
        let now = ui.input(|input| input.time);
        let style = ui.app_style();
        let failure = match &mut runtime.feedback {
            Some(Feedback::Request { latest, failure_until }) => {
                let tint = match context.data_store.command_sequence(*latest).status {
                    CommandStatus::Pending | CommandStatus::Success => None,
                    CommandStatus::TimedOut => Some(style.timeout_fill),
                    CommandStatus::LocalError => Some(style.local_error_fill),
                    CommandStatus::Rejected => Some(style.error_fill),
                };
                // The tint deadline starts when the failure is first observed
                tint.map(|tint| (tint, *failure_until.get_or_insert(now + FAILURE_TINT_SECONDS)))
            }
            Some(Feedback::LocalError { until }) => Some((style.local_error_fill, *until)),
            None => None,
        };

        // Show the tint until it expires, otherwise the configured color
        let mut fill = self.color;
        if let Some((tint, until)) = failure.filter(|(_, until)| now < *until) {
            fill = tint;
            ui.ctx().request_repaint_after(Duration::from_secs_f64(until - now));
        }
        fill = fill.to_opaque();

        // Darken the face right after sending and fade back over the flash duration
        if let Some(sent_at) = runtime.sent_at {
            let remaining = (1. - (now - sent_at) / SEND_FLASH_SECONDS).clamp(0., 1.) as f32;
            if remaining > 0. {
                fill = fill.lerp_to_gamma(Color32::BLACK, SEND_FLASH_DARKENING * remaining);
                ui.ctx().request_repaint(); // Animate until the flash ends
            } else {
                runtime.sent_at = None;
            }
        }

        // Draw the inset button, enabled only when it can send
        let enabled = context.allow_commands && context.adapter.is_some() && self.command.is_configured();
        let text = RichText::new(&self.label)
            .strong()
            .color(centered_value::contrasting_text(fill));
        let button = Button::new(text).fill(fill).corner_radius(BUTTON_CORNER_RADIUS);
        let mut response = egui::Frame::NONE
            .inner_margin(BUTTON_INSET)
            .show(ui, |ui| {
                ui.add_enabled(enabled, |ui: &mut Ui| {
                    ui.add_sized(ui.available_size().max(Vec2::ZERO), button)
                })
            })
            .inner;

        // Show the bound command on hover
        if enabled && let Some(adapter) = context.adapter {
            response = response.on_hover_ui(|ui| {
                ui.label(self.command.summary(adapter.describe_protocol()));
            });
        }

        // Send a fresh command and track it as the latest request
        if enabled
            && response.clicked()
            && let Some(adapter) = context.adapter
        {
            runtime.feedback = Some(match self.command.resolve(adapter.describe_protocol()) {
                Ok(command) => {
                    runtime.sent_at = Some(now);
                    Feedback::Request {
                        latest: context.data_store.enqueue_command(command),
                        failure_until: None,
                    }
                }
                Err(_) => Feedback::LocalError {
                    until: now + FAILURE_TINT_SECONDS,
                },
            });
            ui.ctx().request_repaint();
        }
        ui.data_mut(|data| data.insert_temp(runtime_id, runtime));
    }
}

/// Request feedback scoped to one adapter lifecycle, never persisted.
#[derive(Clone, Default)]
struct CommandRuntime {
    /// Adapter identity owning the tracked command sequence.
    adapter: Option<DataAdapterInstanceToken>,
    /// Feedback for the newest click, if any.
    feedback: Option<Feedback>,
    /// Egui monotonic time in seconds of the latest successful send, cleared once its flash ends.
    sent_at: Option<f64>,
}

/// Feedback for the newest attempt, independent of older requests still in flight.
#[derive(Clone)]
enum Feedback {
    /// A resolved command tracked through its datastore sequence.
    Request {
        /// Datastore-issued sequence identifier of the latest request.
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
