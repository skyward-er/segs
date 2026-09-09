use std::ops::RangeInclusive;

use egui::{Color32, PopupAnchor, Pos2, Shape, Stroke, Ui, pos2};
use egui_plot::{
    ClosestElem, Cursor, HoverPosition, LabelFormatterFn, PlotBounds, PlotConfig, PlotGeometry, PlotItem, PlotItemBase,
    PlotPoint, PlotTransform,
};

/// Creates a solid plot line by mapping borrowed caller-owned values directly into plot coordinates.
///
/// `mapper` returns `(x, y)` in plot coordinates for each value. The returned
/// item borrows `points` and stores the mapper, without allocating an
/// intermediate collection of [`PlotPoint`] values. Rendering allocates only
/// the screen-space path consumed by egui.
///
/// The returned item supports upstream-compatible bounds, highlighting, and
/// closest-segment hover behavior.
pub fn mapped_line<'a, T, F>(name: impl Into<String>, points: &'a [T], mapper: F, stroke: Stroke) -> impl PlotItem + 'a
where
    T: 'a,
    F: Fn(&T) -> (f64, f64) + 'a,
{
    MappedLine {
        base: PlotItemBase::new(name.into()),
        points,
        mapper,
        stroke,
    }
}

struct MappedLine<'a, T, F> {
    base: PlotItemBase,
    points: &'a [T],
    mapper: F,
    stroke: Stroke,
}

impl<T, F> MappedLine<'_, T, F>
where
    F: Fn(&T) -> (f64, f64),
{
    #[inline]
    fn plot_point(&self, index: usize) -> PlotPoint {
        let (x, y) = (self.mapper)(&self.points[index]);
        PlotPoint::new(x, y)
    }

    #[inline]
    fn screen_position(&self, point: &T, transform: &PlotTransform) -> Pos2 {
        let (x, y) = (self.mapper)(point);
        pos2(transform.position_from_point_x(x), transform.position_from_point_y(y))
    }
}

impl<T, F> PlotItem for MappedLine<'_, T, F>
where
    F: Fn(&T) -> (f64, f64),
{
    fn shapes(&self, _ui: &Ui, transform: &PlotTransform, shapes: &mut Vec<Shape>) {
        // Transform borrowed source values directly into the path owned by the shape
        let positions: Vec<_> = self
            .points
            .iter()
            .map(|point| self.screen_position(point, transform))
            .collect();

        // Match upstream line behavior for empty, singleton, and multi-point paths
        match positions.len() {
            0 => {}
            1 => {
                let mut radius = self.stroke.width / 2.0;
                if self.highlighted() {
                    radius *= 2_f32.sqrt();
                }
                shapes.push(Shape::circle_filled(positions[0], radius, self.stroke.color));
            }
            _ => {
                let mut stroke = self.stroke;
                if self.highlighted() {
                    stroke.width *= 2.0;
                }
                shapes.push(Shape::line(positions, stroke));
            }
        }
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {}

    fn color(&self) -> Color32 {
        self.stroke.color
    }

    fn geometry(&self) -> PlotGeometry<'_> {
        PlotGeometry::None
    }

    fn bounds(&self) -> PlotBounds {
        // Fold mapped coordinates into bounds without retaining canonical points
        let mut bounds = PlotBounds::NOTHING;
        for point in self.points {
            let (x, y) = (self.mapper)(point);
            bounds.extend_with_x(x);
            bounds.extend_with_y(y);
        }
        bounds
    }

    fn base(&self) -> &PlotItemBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut PlotItemBase {
        &mut self.base
    }

    fn find_closest(&self, pointer: Pos2, transform: &PlotTransform) -> Option<ClosestElem> {
        // Preserve hover behavior for empty and singleton lines
        if self.points.len() <= 1 {
            return self
                .points
                .iter()
                .enumerate()
                .map(|(index, point)| ClosestElem {
                    index,
                    dist_sq: pointer.distance_sq(self.screen_position(point, transform)),
                })
                .min_by(|left, right| left.dist_sq.total_cmp(&right.dist_sq));
        }

        // Find the nearest line segment and report its nearest real endpoint
        self.points
            .windows(2)
            .enumerate()
            .map(|(index, pair)| {
                let first = self.screen_position(&pair[0], transform);
                let second = self.screen_position(&pair[1], transform);
                let dist_sq = dist_sq_to_segment(pointer, [first, second]);
                let index = if pointer.distance_sq(first) <= pointer.distance_sq(second) {
                    index
                } else {
                    index + 1
                };
                ClosestElem { index, dist_sq }
            })
            .min_by(|left, right| left.dist_sq.total_cmp(&right.dist_sq))
    }

    fn on_hover(
        &self,
        plot_area_response: &egui::Response,
        elem: ClosestElem,
        shapes: &mut Vec<Shape>,
        cursors: &mut Vec<Cursor>,
        plot: &PlotConfig<'_>,
        label_formatter: Option<&LabelFormatterFn<'_>>,
    ) {
        // Highlight the selected sample with the same marker as upstream lines
        let value = self.plot_point(elem.index);
        let pointer = plot.transform.position_from_point(&value);
        let marker_color = if plot.ui.visuals().dark_mode {
            Color32::from_gray(100).additive()
        } else {
            Color32::from_black_alpha(180)
        };
        shapes.push(Shape::circle_filled(pointer, 3.0, marker_color));

        show_rulers_and_tooltip(
            plot_area_response,
            value,
            self.name(),
            elem.index,
            plot,
            cursors,
            label_formatter,
        );
    }
}

fn dist_sq_to_segment(point: Pos2, [start, end]: [Pos2; 2]) -> f32 {
    // Project the pointer onto the segment and clamp it to both endpoints
    let segment = end - start;
    let segment_len_sq = segment.length_sq();
    if segment_len_sq == 0.0 {
        return point.distance_sq(start);
    }

    let projection = segment.dot(point - start) / segment_len_sq;
    let closest = start + projection.clamp(0.0, 1.0) * segment;
    point.distance_sq(closest)
}

fn show_rulers_and_tooltip(
    plot_area_response: &egui::Response,
    value: PlotPoint,
    name: &str,
    index: usize,
    plot: &PlotConfig<'_>,
    cursors: &mut Vec<Cursor>,
    label_formatter: Option<&LabelFormatterFn<'_>>,
) {
    // Add the configured crosshair rulers at the selected value
    if plot.show_crosshair {
        if plot.show_x {
            cursors.push(Cursor::Vertical { x: value.x });
        }
        if plot.show_y {
            cursors.push(Cursor::Horizontal { y: value.y });
        }
    }

    // Format the nearest-point label when the plot has a formatter
    let Some(formatter) = label_formatter else {
        return;
    };
    let Some(text) = formatter(&HoverPosition::NearDataPoint {
        plot_name: name,
        position: value,
        index,
    }) else {
        return;
    };

    // Display the label using the upstream tooltip layout
    let mut tooltip = egui::Tooltip::always_open(
        plot_area_response.ctx.clone(),
        plot_area_response.layer_id,
        plot_area_response.id,
        PopupAnchor::Pointer,
    );
    let tooltip_width = plot_area_response.ctx.global_style().spacing.tooltip_width;
    tooltip.popup = tooltip.popup.width(tooltip_width);
    tooltip.gap(12.0).show(|ui| {
        ui.set_max_width(tooltip_width);
        ui.label(text);
    });
}
