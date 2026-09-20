use egui::{
    Align, Color32, CornerRadius, Id, InnerResponse, Layout, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui,
    UiBuilder, Vec2, Widget, WidgetText, emath::GuiRounding, layers::ShapeIdx, pos2, vec2,
};

use crate::style::CtxStyleExt;

/// A bounded table with rounded outer borders and explicit cell allocation.
/// Unlike `egui::Grid`, ordinary rows have a fixed allocated height and oversized
/// contents are clipped, not measured to expand their row or column.
pub struct RoundedGrid {
    /// Locally unique identity for the table and its cells.
    id: Id,
    /// Column width policies, defaulting to one remainder column.
    columns: Vec<RoundedGridColumn>,
    /// Optional requested width, capped to the available parent width.
    width: Option<f32>,
    /// Ordinary row height in logical points.
    row_height: Option<f32>,
    /// Padding inside each cell in logical points.
    padding: Vec2,
    /// Number of initial rendered rows with header fill.
    header_rows: usize,
    /// Outer corner radius in logical points.
    radius: u8,
    /// Optional override for the themed border.
    stroke: Option<Stroke>,
    /// Optional override for the themed header fill.
    header_fill: Option<Color32>,
}

impl RoundedGrid {
    /// Creates a table with one expanding column and no header rows.
    pub fn new(id_salt: impl egui::AsId) -> Self {
        Self {
            id: Id::new(id_salt),
            columns: vec![RoundedGridColumn::Remainder],
            width: None,
            row_height: None,
            padding: vec2(4., 2.),
            header_rows: 0,
            radius: 4,
            stroke: None,
            header_fill: None,
        }
    }

    /// Sets column policies, using one remainder column if the iterator is empty.
    pub fn columns(mut self, columns: impl IntoIterator<Item = RoundedGridColumn>) -> Self {
        self.columns = columns.into_iter().collect();
        if self.columns.is_empty() {
            self.columns.push(RoundedGridColumn::Remainder);
        }
        self
    }

    /// Sets a requested logical width, clamped to the parent when shown.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.));
        self
    }

    /// Sets ordinary row height and the minimum merged-row height in logical points.
    pub fn min_row_height(mut self, height: f32) -> Self {
        self.row_height = Some(height.max(0.));
        self
    }

    /// Sets nonnegative horizontal and vertical cell padding in logical points.
    pub fn cell_padding(mut self, padding: impl Into<Vec2>) -> Self {
        self.padding = padding.into().max(Vec2::ZERO);
        self
    }

    /// Shades the first `count` rendered rows as headers.
    pub fn header_rows(mut self, count: usize) -> Self {
        self.header_rows = count;
        self
    }

    /// Sets the outer corner radius in logical points.
    pub fn corner_radius(mut self, radius: u8) -> Self {
        self.radius = radius;
        self
    }

    /// Overrides the border color and logical width, snapped to physical pixels.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = Some(stroke.into());
        self
    }

    /// Overrides the fill used for header rows.
    pub fn header_fill(mut self, fill: Color32) -> Self {
        self.header_fill = Some(fill);
        self
    }

    /// Renders content once and returns its result and the table's layout response.
    /// Finishes an incomplete final row without adding an extra empty row.
    pub fn show<R>(self, ui: &mut Ui, contents: impl FnOnce(&mut RoundedGridUi<'_>) -> R) -> InnerResponse<R> {
        ui.push_id(self.id, |ui| {
            // Resolve bounded columns before running any cell content
            let width = self
                .width
                .unwrap_or(ui.available_width())
                .min(ui.available_width())
                .max(0.);
            let fixed: f32 = self
                .columns
                .iter()
                .map(|column| match column {
                    RoundedGridColumn::Fixed(width) => width.max(0.),
                    RoundedGridColumn::Remainder => 0.,
                })
                .sum();
            let remainders = self
                .columns
                .iter()
                .filter(|column| matches!(column, RoundedGridColumn::Remainder))
                .count();
            let scale = if fixed > width { width / fixed } else { 1. };
            let remainder = (width - fixed).max(0.) / remainders.max(1) as f32;
            let widths = self
                .columns
                .iter()
                .map(|column| match column {
                    RoundedGridColumn::Fixed(width) => width.max(0.) * scale,
                    RoundedGridColumn::Remainder => remainder,
                })
                .collect();
            let mut stroke = self
                .stroke
                .unwrap_or_else(|| Stroke::new(1., ui.app_style().widgets.noninteractive.bg_stroke_color));
            let ppp = ui.ctx().pixels_per_point();
            stroke.width = if stroke.width > 0. {
                (stroke.width * ppp).round().max(1.) / ppp
            } else {
                0.
            };
            let header_fill = self
                .header_fill
                .unwrap_or_else(|| ui.app_style().widgets.noninteractive.bg_fill);
            let row_height = self.row_height.unwrap_or(ui.spacing().interact_size.y);

            // Own row spacing locally so adjacent rows share a single boundary
            ui.scope(|ui| {
                ui.spacing_mut().item_spacing.y = 0.;
                let mut grid = RoundedGridUi {
                    ui,
                    widths,
                    width,
                    row_height,
                    padding: self.padding,
                    rows: Vec::new(),
                    column: 0,
                    next_x: 0.,
                    ordinary_rows: 0,
                    pending: None,
                    header_rows: self.header_rows,
                };
                let result = contents(&mut grid);
                grid.end_row();
                grid.paint(stroke, self.radius, header_fill);
                result
            })
            .inner
        })
    }
}

/// Width policy for a bounded table column.
#[derive(Clone, Copy, Debug)]
pub enum RoundedGridColumn {
    /// Requested width in logical points, proportionally reduced if space runs out.
    Fixed(f32),
    /// Equal share of space remaining after fixed columns.
    Remainder,
}

/// Grid-like content API with bounded cells and explicit row completion.
pub struct RoundedGridUi<'a> {
    /// Parent UI that allocates rows.
    ui: &'a mut Ui,
    /// Resolved column widths in logical points.
    widths: Vec<f32>,
    /// Total bounded table width.
    width: f32,
    /// Fixed ordinary row height.
    row_height: f32,
    /// In-cell padding.
    padding: Vec2,
    /// Allocated rows retained for final border painting.
    rows: Vec<GridRow>,
    /// Next column of the pending row.
    column: usize,
    /// Running horizontal offset within the current row.
    next_x: f32,
    /// Ordinary row count, unaffected by inserted merged feedback rows.
    ordinary_rows: usize,
    /// Current ordinary row rectangle, if unfinished.
    pending: Option<Rect>,
    /// Number of leading rows to shade.
    header_rows: usize,
}

impl RoundedGridUi<'_> {
    /// Adds a widget to one bounded cell and returns its interaction response.
    pub fn add(&mut self, widget: impl Widget) -> Response {
        self.cell(|ui| ui.add(widget)).inner
    }

    /// Adds a label to one cell and returns its interaction response.
    pub fn label(&mut self, text: impl Into<WidgetText>) -> Response {
        self.add(egui::Label::new(text))
    }

    /// Runs arbitrary egui content in one cell and returns its result and response.
    /// Automatically starts another row once all columns have been consumed.
    pub fn cell<R>(&mut self, contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        // Allocate each ordinary row once, independent of widget desired sizes
        if self.column == self.widths.len() {
            self.end_row();
        }
        let row = match self.pending {
            Some(row) => row,
            None => {
                let row = self.allocate_row(self.row_height, false);
                self.ordinary_rows += 1;
                self.pending = Some(row);
                row
            }
        };
        let left = row.left() + self.next_x;
        let rect = Rect::from_min_size(pos2(left, row.top()), vec2(self.widths[self.column], row.height()));
        let id = self.ui.id().with(("cell", self.ordinary_rows - 1, self.column));
        self.next_x += self.widths[self.column];
        self.column += 1;
        self.cell_ui(rect, id, contents)
    }

    /// Completes the current row, leaving unused cells empty; otherwise does nothing.
    pub fn end_row(&mut self) {
        self.pending = None;
        self.column = 0;
        self.next_x = 0.;
    }

    /// Returns usable merged-row content width in logical points, after padding.
    pub fn available_width(&self) -> f32 {
        (self.width - self.padding.x * 2.).max(0.)
    }

    /// Adds one merged row and returns its content result and response.
    /// Completes any pending row first and clamps height to the ordinary row height.
    pub fn full_width_row<R>(&mut self, height: f32, contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.end_row();
        let rect = self.allocate_row(height.max(self.row_height), true);
        self.cell_ui(rect, self.ui.id().with(("merged", self.rows.len() - 1)), contents)
    }

    /// Allocates a row and reserves a background paint slot behind its children.
    fn allocate_row(&mut self, height: f32, merged: bool) -> Rect {
        let (rect, _) = self.ui.allocate_exact_size(vec2(self.width, height), Sense::hover());
        let background = self.ui.painter().add(Shape::Noop);
        self.rows.push(GridRow {
            rect,
            merged,
            background,
        });
        rect
    }

    /// Returns bounded child content and its response without expanding the parent.
    fn cell_ui<R>(&mut self, rect: Rect, id: Id, contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        // Keep padding valid even when narrow columns collapse to zero width
        let padding = self.padding.min(rect.size() * 0.5);
        let content_rect = rect.shrink2(padding);
        let mut child = self.ui.new_child(
            UiBuilder::new()
                .id(id)
                .max_rect(content_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        child.set_clip_rect(self.ui.clip_rect().intersect(content_rect));
        let inner = contents(&mut child);
        InnerResponse {
            inner,
            response: child.response(),
        }
    }

    /// Paints header fills, clipped separators, and one rounded outer perimeter.
    fn paint(&self, stroke: Stroke, radius: u8, header_fill: Color32) {
        let (Some(first), Some(last)) = (self.rows.first(), self.rows.last()) else {
            return;
        };
        let ppp = self.ui.ctx().pixels_per_point();
        let table = first.rect.union(last.rect).round_to_pixels(ppp);
        let interior = table.shrink(stroke.width);
        let painter = self
            .ui
            .painter()
            .with_clip_rect(self.ui.clip_rect().intersect(interior));

        // Use filled rectangles instead of stroked lines to avoid protruding end caps
        for (index, row) in self.rows.iter().enumerate() {
            if index < self.header_rows {
                let corners = CornerRadius {
                    nw: if index == 0 { radius } else { 0 },
                    ne: if index == 0 { radius } else { 0 },
                    sw: if index + 1 == self.rows.len() { radius } else { 0 },
                    se: if index + 1 == self.rows.len() { radius } else { 0 },
                };
                self.ui.painter().set(
                    row.background,
                    Shape::rect_filled(row.rect.round_to_pixels(ppp), corners, header_fill),
                );
            }
            let separator = |rect: Rect| {
                let rect = rect.round_to_pixels(ppp).intersect(interior);
                if interior.is_positive() && rect.is_positive() {
                    painter.rect_filled(rect, 0., stroke.color);
                }
            };
            if index > 0 {
                separator(Rect::from_min_size(row.rect.min, vec2(self.width, stroke.width)));
            }
            if !row.merged {
                let mut x = row.rect.left();
                for width in self.widths.iter().take(self.widths.len() - 1) {
                    x += width;
                    separator(Rect::from_min_size(
                        pos2(x, row.rect.top()),
                        vec2(stroke.width, row.rect.height()),
                    ));
                }
            }
        }
        self.ui.painter().rect_stroke(table, radius, stroke, StrokeKind::Inside);
    }
}

/// Deferred row geometry and background slot for final table decoration.
struct GridRow {
    /// Allocated row bounds.
    rect: Rect,
    /// Whether column separators are omitted.
    merged: bool,
    /// Paint slot behind row contents.
    background: ShapeIdx,
}
