use std::ops::RangeInclusive;

use egui::{Color32, CornerRadius, Id, StrokeKind, TextStyle, TextWrapMode, Ui, Vec2, WidgetText, pos2, vec2};
use segs_plot::{HoverPosition, Plot, PlotUi};

const OVERLAY_MARGIN: f32 = 4.;
const OVERLAY_HORIZONTAL_PADDING: f32 = 6.;
const OVERLAY_VERTICAL_PADDING: f32 = 3.;

/// Visual settings applied to one plotted line.
#[derive(Clone, Debug)]
pub struct LineSettings {
    /// Stroke width in logical points.
    pub width: f32,
    /// Stroke color.
    pub color: Color32,
}

impl Default for LineSettings {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: Color32::BLUE,
        }
    }
}

/// Presentation options shared by every series in a plot widget.
pub struct PlotOptions {
    /// Whether both plot axes are visible.
    pub show_axes: bool,
    /// Optional horizontal-axis label.
    pub x_label: Option<String>,
    /// Optional vertical-axis label.
    pub y_label: Option<String>,
    /// Whether plot bounds follow the supplied series automatically.
    pub auto_bounds: bool,
    /// Optional horizontal range included while automatic bounds are active.
    pub x_auto_range: Option<RangeInclusive<f64>>,
    /// Optional vertical bounds used initially and when the plot is reset.
    pub y_bounds: Option<RangeInclusive<f64>>,
    /// Fractional margin added around automatically calculated bounds.
    pub margin_fraction: Vec2,
    /// Whether retained plot interaction state is reset for this frame.
    pub reset: bool,
}

impl Default for PlotOptions {
    fn default() -> Self {
        Self {
            show_axes: true,
            x_label: None,
            y_label: None,
            auto_bounds: true,
            x_auto_range: None,
            y_bounds: None,
            margin_fraction: Vec2::splat(0.05),
            reset: false,
        }
    }
}

/// Draws plot contents supplied by `contents` and returns the plot's interaction response.
///
/// The callback receives the [`PlotUi`] used to add plot items and is invoked
/// exactly once while the widget is built. `label_formatter` controls labels
/// shown near plot data, while a provided `persistent_label` is painted in the
/// top-right corner. The returned response contains the interaction state for
/// the complete plot widget.
pub fn plot_widget<'a>(
    ui: &mut Ui,
    id: impl Into<Id>,
    opts: &PlotOptions,
    persistent_label: Option<&str>,
    label_formatter: impl Fn(&HoverPosition<'_>) -> Option<String> + 'a,
    contents: impl FnOnce(&mut PlotUi<'a>) + 'a,
) -> egui::Response {
    // Configure plot-wide behavior and optional axis labels
    let mut plot = Plot::new(id.into())
        .auto_bounds([opts.auto_bounds, opts.auto_bounds])
        .allow_boxed_zoom(false)
        .show_axes(opts.show_axes)
        .label_formatter(label_formatter)
        .set_margin_fraction(opts.margin_fraction);

    // Include a caller-defined horizontal range while the plot is following data
    if let Some(range) = &opts.x_auto_range {
        plot = plot.include_x(*range.start()).include_x(*range.end());
    }

    // Apply valid vertical bounds as the initial and reset range
    if let Some(range) = &opts.y_bounds
        && range.start().is_finite()
        && range.end().is_finite()
        && range.start() < range.end()
    {
        plot = plot.default_y_bounds(*range.start(), *range.end());
    }

    if let Some(label) = &opts.x_label {
        plot = plot.x_axis_label(label);
    }
    if let Some(label) = &opts.y_label {
        plot = plot.y_axis_label(label);
    }

    // Discard retained interaction state after a plot configuration change
    if opts.reset {
        plot = plot.reset();
    }

    // Build caller-provided plot items within the configured plot
    let response = plot.show(ui, contents);
    if let Some(persistent_label) = persistent_label {
        paint_persistent_label(ui, response.response.rect, persistent_label);
    }

    response.response
}

/// Paints a compact non-interactive label in the plot's top-right corner.
fn paint_persistent_label(ui: &Ui, plot_rect: egui::Rect, text: &str) {
    // Fit one truncated line inside the plot while retaining the outer margin
    let maximum_text_width = (plot_rect.width() - 2. * (OVERLAY_MARGIN + OVERLAY_HORIZONTAL_PADDING)).max(0.);
    let galley =
        WidgetText::from(text).into_galley(ui, Some(TextWrapMode::Truncate), maximum_text_width, TextStyle::Body);
    let badge_size = galley.size() + vec2(2. * OVERLAY_HORIZONTAL_PADDING, 2. * OVERLAY_VERTICAL_PADDING);
    let badge_rect = egui::Rect::from_min_size(
        pos2(
            plot_rect.right() - OVERLAY_MARGIN - badge_size.x,
            plot_rect.top() + OVERLAY_MARGIN,
        ),
        badge_size,
    );

    // Draw the badge without registering an interaction over the plot
    let visuals = ui.visuals();
    let painter = ui.painter().with_clip_rect(plot_rect);
    painter.rect(
        badge_rect,
        CornerRadius::same(3),
        visuals.extreme_bg_color.gamma_multiply(0.85),
        visuals.window_stroke,
        StrokeKind::Inside,
    );
    painter.galley(
        badge_rect.min + vec2(OVERLAY_HORIZONTAL_PADDING, OVERLAY_VERTICAL_PADDING),
        galley,
        visuals.text_color(),
    );
}
