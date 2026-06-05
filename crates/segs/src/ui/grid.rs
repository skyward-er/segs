use egui::{Pos2, Rect, Vec2, vec2};

/// A rect in grid space coordinates.
/// Given in grid column and row count.
///
/// Use [`Grid::to_screen_rect`] to turn it into a usable [`Rect`] for screen space calculations.
/// A type-safe wrapper guarantees that grid rects don't get mixed up with screen space rects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GRect(Rect);

impl GRect {
    pub fn new(rect: Rect) -> Self {
        Self(rect)
    }
}

#[derive(Clone, Copy)]
pub enum GridSettings {
    Auto { granularity: f32 },
    Fixed { cols: u32, rows: u32 },
}

impl GridSettings {
    pub fn fixed(cols: u32, rows: u32) -> Self {
        Self::Fixed { cols, rows }
    }

    pub fn auto(granularity: f32) -> Self {
        Self::Auto { granularity }
    }
}

/// The widget grid.
///
/// Handles grid cell sizing, snapping and widget positioning/sizing in the grid.
///
/// When working with the widget grid, there's two types of coordinates you should be aware of, see below.
///
/// ### Screen space coordinates
/// Used by egui for rendering. They're given in screen points and are used in egui types such as [`Rect`] and [`Pos2`].
///
/// ### Grid space coordinates
/// They represent widget position in the grid, in a size-independent fashion. They're given in grid column and row
/// counts. They are stored in the types [`GRect`] and [`GPos2`].
pub struct Grid {
    /// The rect the grid is being drawn on.
    pub rect: Rect,
    /// Cell size in each direction.
    pub cell_size: Vec2,
}

impl Grid {
    /// Constructs a grid object in the given screen space area with the given settings.
    pub fn new(rect: Rect, settings: GridSettings) -> Self {
        let cell_count = match settings {
            GridSettings::Auto { granularity } => {
                // Compute how many granularity-sized cells fit in the grid space
                Vec2 {
                    x: (rect.width() / granularity).round().max(1.),
                    y: (rect.height() / granularity).round().max(1.),
                }
            }
            GridSettings::Fixed { cols, rows } => {
                // Grid cell count is fixed, just return the amounts
                vec2(cols as f32, rows as f32)
            }
        };

        let cell_size = rect.size() / cell_count;

        Self { rect, cell_size }
    }

    /// Transforms a [`GRect`] to a screen-space [`Rect`] usable for rendering with egui.
    pub fn to_screen_rect(&self, grect: GRect) -> Rect {
        let cell_size = self.cell_size;

        let min = (grect.0.min.to_vec2() * cell_size).to_pos2();
        let max = (grect.0.max.to_vec2() * cell_size).to_pos2();

        Rect::from_min_max(min, max).translate(self.origin().to_vec2())
    }

    /// Transforms a [`Rect`] to a grid space [`GRect`], implicitly snapping it to the grid.
    pub fn to_grid_rect(&self, rect: Rect) -> GRect {
        let cell_size = self.cell_size;

        let rect = rect.translate(-self.origin().to_vec2());

        let min = (rect.min.to_vec2() / cell_size).round().to_pos2();
        let max = (rect.max.to_vec2() / cell_size).round().to_pos2();

        GRect(Rect::from_min_max(min, max))
    }

    fn origin(&self) -> Pos2 {
        self.rect.min
    }
}
