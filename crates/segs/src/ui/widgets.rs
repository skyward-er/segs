mod mapped_value;
mod message_viewer;
mod plot;
mod state_machine;
mod value_display;

use enum_dispatch::enum_dispatch;
pub use mapped_value::MappedValueWidget;
pub use message_viewer::MessageViewerWidget;
pub use plot::PlotWidget;
pub use state_machine::StateMachineWidget;
pub use value_display::ValueDisplayWidget;

use egui::{Id, Ui, Vec2};
use serde::{Deserialize, Serialize};

use crate::{
    dataflow::store::DataStore,
    ui::{
        grid::GRect,
        widget_settings::{WidgetDataSetting, WidgetSetting},
    },
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetData {
    pub id: Id,
    /// Widget rect in grid space coordinates
    pub grect: GRect,

    /// The concrete type of widget
    pub variant: WidgetVariant,
}

#[enum_dispatch(WidgetTrait)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WidgetVariant {
    MappedValue(MappedValueWidget),
    MessageViewer(MessageViewerWidget),
    Plot(PlotWidget),
    /// Ordered telemetry states with commands for adjacent transitions.
    StateMachine(StateMachineWidget),
    ValueDisplay(ValueDisplayWidget),
}

impl WidgetVariant {
    /// Gallery defaults in display order.
    pub fn gallery() -> Vec<Self> {
        vec![
            ValueDisplayWidget::default().into(),
            MappedValueWidget::default().into(),
            StateMachineWidget::default().into(),
            PlotWidget::default().into(),
            MessageViewerWidget::default().into(),
        ]
    }
}

#[enum_dispatch]
pub trait WidgetTrait {
    /// Renders with explicit command authority and the active adapter.
    /// Read-only widgets use their existing datastore-only rendering implementation.
    fn show_with_context(&self, ui: &mut Ui, context: &mut WidgetRenderContext<'_>) {
        self.show(ui, context.data_store);
    }
    /// Binds a gallery-only clone to isolated sample data without changing defaults.
    /// Widgets may override this to provide their own sample configuration.
    fn configure_preview(&mut self, preview: &crate::dataflow::preview::PreviewContext) {
        for mut setting in self.data_settings() {
            setting.set_stream_if_empty(preview.numeric_stream);
        }
    }

    /// Normalizes transient drafts before persistence, doing nothing by default.
    /// Called on a save clone so failed writes leave the working widget untouched.
    fn prepare_for_save(&mut self) {}

    /// Show the content of the widget.
    fn show(&self, ui: &mut Ui, data_store: &mut DataStore);

    /// Data stream settings exposed by this widget.
    ///
    /// Implementations must explicitly return an empty vector when they do not
    /// consume data streams.
    fn data_settings(&mut self) -> Vec<WidgetDataSetting<'_>>;

    /// Settings exposed by this widget for the standard settings panel.
    fn settings(&mut self) -> Vec<WidgetSetting<'_>> {
        Vec::new()
    }

    /// Gallery display name.
    fn display_name(&self) -> &'static str;

    /// Minimum size of the widget in grid space units.
    fn min_size(&self) -> Vec2 {
        Vec2::ONE
    }

    /// Default size of the widget in grid space units. May be more than the minimum size.
    fn default_size(&self) -> Vec2 {
        self.min_size()
    }
}

/// Per-render data access and authority for interactive command widgets.
pub struct WidgetRenderContext<'a> {
    /// Live or isolated preview data owned by the caller.
    pub data_store: &'a mut DataStore,
    /// Active adapter used to validate command descriptors and lifecycle identity.
    pub adapter: Option<&'a crate::dataflow::adapter::DataAdapterInstance>,
    /// Whether this surface permits command transmission.
    pub allow_commands: bool,
}

impl<'a> WidgetRenderContext<'a> {
    /// Returns a context that cannot issue commands, suitable for all editor previews.
    pub fn preview(data_store: &'a mut DataStore) -> Self {
        Self {
            data_store,
            adapter: None,
            allow_commands: false,
        }
    }
}
