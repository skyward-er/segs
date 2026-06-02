mod value_display;

use enum_dispatch::enum_dispatch;
pub use value_display::ValueDisplayWidget;

use egui::{Id, Pos2, Rect, Ui, Vec2};

use crate::app::AppContext;

/// A widget position in the main view.
///
/// Given in column and row count.
///
/// A custom type guarantees that Pos2 coordinates don't get mixed up with widget positions.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WPos2(Vec2);

impl WPos2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

#[inline(always)]
pub const fn wpos2(x: f32, y: f32) -> WPos2 {
    WPos2::new(x, y)
}

pub struct WidgetData {
    pub id: Id,
    pub pos: WPos2,
    /// Size in [x: columns, y: rows]
    pub size: Vec2,

    /// The concrete type of widget
    pub variant: WidgetVariant,
}

impl WidgetData {
    /// Compute the [`Rect`] this widget wants to be drawn in
    pub fn rect(&self, origin: Pos2, grid_size: Vec2) -> Rect {
        let min = origin + self.pos.0 * grid_size;
        let size = self.size * grid_size;
        Rect::from_min_size(min, size)
    }

    pub fn show(&self, ui: &mut Ui, appctx: &mut AppContext) {
        self.variant.show(ui, appctx);
    }
}

#[enum_dispatch(WidgetTrait)]
pub enum WidgetVariant {
    ValueDisplay(ValueDisplayWidget),
}

#[enum_dispatch]
pub trait WidgetTrait {
    /// Show the content of the widget
    fn show(&self, ui: &mut Ui, appctx: &mut AppContext);
}
