mod value_display;

use enum_dispatch::enum_dispatch;
pub use value_display::ValueDisplayWidget;

use egui::{Id, Ui, Vec2};

use crate::{dataflow::DataStore, ui::grid::GRect};

pub struct WidgetData {
    pub id: Id,
    /// Widget rect in grid space coordinates
    pub grect: GRect,

    /// The concrete type of widget
    pub variant: WidgetVariant,
}

impl WidgetData {
    pub fn show(&self, ui: &mut Ui, data_store: &mut DataStore) {
        self.variant.show(ui, data_store);
    }
}

#[enum_dispatch(WidgetTrait)]
pub enum WidgetVariant {
    ValueDisplay(ValueDisplayWidget),
}

#[enum_dispatch]
pub trait WidgetTrait {
    /// Show the content of the widget.
    fn show(&self, ui: &mut Ui, data_store: &mut DataStore);

    /// Minimum size of the widget in grid space units.
    fn min_size(&self) -> Vec2 {
        Vec2::ONE
    }

    /// Default size of the widget in grid space units. May be more than the minimum size.
    fn default_size(&self) -> Vec2 {
        self.min_size()
    }
}
