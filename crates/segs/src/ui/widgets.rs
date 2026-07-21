mod message_view;
mod plot;
mod value_display;

use enum_dispatch::enum_dispatch;
pub use message_view::MessageViewWidget;
pub use plot::PlotWidget;
pub use value_display::ValueDisplayWidget;

use egui::{Id, Ui, Vec2};
use serde::{Deserialize, Serialize};

use crate::{
    dataflow::DataStore,
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
    MessageView(MessageViewWidget),
    Plot(PlotWidget),
    ValueDisplay(ValueDisplayWidget),
}

impl WidgetVariant {
    /// Gallery defaults in display order.
    pub fn gallery() -> Vec<Self> {
        vec![
            ValueDisplayWidget::default().into(),
            PlotWidget.into(),
            MessageViewWidget.into(),
        ]
    }
}

#[enum_dispatch]
pub trait WidgetTrait {
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
