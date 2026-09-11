use std::{fmt::Display, ops::RangeInclusive, str::FromStr};

use egui::{
    Align, Align2, CornerRadius, CursorIcon, Direction, Id, Layout, Rect, Response, Sense, Shape, TextStyle, Ui,
    UiBuilder, Widget, WidgetInfo, WidgetType, layers::ShapeIdx, pos2, vec2,
};

use crate::{
    style::CtxStyleExt,
    widgets::text::{
        ValueEdit,
        text_edit::{default_singleline_height, frame_rect},
    },
};

const CORNER_RADIUS: u8 = 3;
const MINIMUM_TEXT_WIDTH: f32 = 32.;

/// A numeric editor with attached decrement and increment buttons.
pub struct NumericStepper<'a, V: StepperValue> {
    value: &'a mut V,
    range: RangeInclusive<V>,
    step: V,
    desired_width: Option<f32>,
    // Carries the optional ID salt into the child UI that scopes all internal interactions
    builder: UiBuilder,
}

impl<'a, V: StepperValue> NumericStepper<'a, V> {
    /// Creates a numeric stepper with the full finite range and a step of one.
    ///
    /// Returns a stepper that edits `value` directly.
    pub fn new(value: &'a mut V) -> Self {
        Self {
            value,
            range: V::MIN..=V::MAX,
            step: V::ONE,
            desired_width: None,
            builder: UiBuilder::new(),
        }
    }

    /// Sets the inclusive range accepted by typing and stepping.
    ///
    /// Returns the configured stepper. The range must be finite and not empty.
    pub fn range(mut self, range: RangeInclusive<V>) -> Self {
        debug_assert!(!range.is_empty(), "numeric stepper range must not be empty");
        debug_assert!(
            range.start().is_finite() && range.end().is_finite(),
            "numeric stepper range must be finite"
        );
        self.range = range;
        self
    }

    /// Sets the positive amount applied by either button.
    ///
    /// Returns the configured stepper. The step must be finite and greater than zero.
    pub fn step(mut self, step: V) -> Self {
        debug_assert!(step > V::ZERO, "numeric stepper step must be positive");
        debug_assert!(step.is_finite(), "numeric stepper step must be finite");
        self.step = step;
        self
    }

    /// Sets a stable identity within the containing UI.
    ///
    /// Returns the configured stepper with state scoped by `id_salt`.
    pub fn id_salt(mut self, id_salt: impl egui::AsIdSalt) -> Self {
        self.builder = self.builder.id_salt(id_salt);
        self
    }

    /// Sets the desired total width of the stepper in logical points.
    ///
    /// Returns the configured stepper. The button widths remain fixed and only
    /// the central text region changes width.
    pub fn desired_width(mut self, desired_width: f32) -> Self {
        self.desired_width = Some(desired_width);
        self
    }

    /// Adds the stepper and reports its transient validation status.
    ///
    /// The returned output contains the combined interaction response and
    /// whether the visible numeric draft is invalid.
    pub fn show(self, ui: &mut Ui) -> NumericStepperOutput {
        let builder = self.builder.clone();
        ui.scope_builder(builder, |ui| self.show_inner(ui)).inner
    }

    fn show_inner(self, ui: &mut Ui) -> NumericStepperOutput {
        let Self {
            value,
            range,
            step,
            desired_width,
            builder: _,
        } = self;

        // Normalize the model and calculate one exact compound-control rectangle
        let minimum = *range.start();
        let maximum = *range.end();
        *value = value.normalized().unwrap_or(minimum).clamped(minimum, maximum);
        let height = default_singleline_height(ui);
        let button_width = height;
        let minimum_width = button_width * 2. + MINIMUM_TEXT_WIDTH;
        let default_width = ui.spacing().text_edit_width + button_width * 2.;
        let width = desired_width.unwrap_or(default_width).max(minimum_width);
        let (rect, outer_response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let painted_rect = frame_rect(rect);
        let background = ui.painter().add(Shape::Noop);
        let editor_highlight = ui.painter().add(Shape::Noop);

        // Split the allocated rectangle without introducing layout spacing
        let decrement_rect = Rect::from_min_max(rect.min, pos2(rect.left() + button_width, rect.bottom()));
        let increment_rect = Rect::from_min_max(pos2(rect.right() - button_width, rect.top()), rect.max);
        let editor_rect = Rect::from_min_max(
            pos2(decrement_rect.right(), rect.top()),
            pos2(increment_rect.left(), rect.bottom()),
        );
        let decrement_painted_rect =
            Rect::from_min_max(painted_rect.min, pos2(decrement_rect.right(), painted_rect.bottom()));
        let increment_painted_rect =
            Rect::from_min_max(pos2(increment_rect.left(), painted_rect.top()), painted_rect.max);
        let editor_painted_rect = Rect::from_min_max(
            pos2(editor_rect.left(), painted_rect.top()),
            pos2(editor_rect.right(), painted_rect.bottom()),
        );

        // Register both button hit regions with stable identities and bound-aware states
        let decrement_enabled = ui.is_enabled() && *value > minimum;
        let increment_enabled = ui.is_enabled() && *value < maximum;
        let decrement_response = button_response(
            ui,
            decrement_rect,
            ui.id().with("decrement"),
            decrement_enabled,
            "Decrease value",
        );
        let increment_response = button_response(
            ui,
            increment_rect,
            ui.id().with("increment"),
            increment_enabled,
            "Increase value",
        );

        // Place a frameless editor precisely between the attached button regions
        let editor_id = ui.id().with("editor");
        let mut editor_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(editor_rect)
                .layout(Layout::centered_and_justified(Direction::TopDown)),
        );
        let editor_output = ValueEdit::new(value)
            .id(editor_id)
            .horizontal_align(Align::Center)
            .vertical_align(Align::Center)
            .with_width(editor_rect.width())
            .update_while_editing(true)
            .normalize(StepperValue::normalized)
            .empty_value(minimum)
            .range(minimum..=maximum)
            .frameless()
            .show_with_status(&mut editor_ui);
        let editor_response = editor_output.response;

        // Apply at most one bounded step using the current typed value
        let stepped = if decrement_enabled && decrement_response.clicked() {
            *value = value.decremented(step, minimum);
            true
        } else if increment_enabled && increment_response.clicked() {
            *value = value.incremented(step, maximum);
            true
        } else {
            false
        };
        if stepped {
            editor_response.surrender_focus();
        }

        // Paint the flat base and each interaction region independently
        paint_background(ui, background, painted_rect);
        paint_editor_highlight(
            ui,
            editor_highlight,
            editor_painted_rect,
            &editor_response,
            editor_output.invalid,
        );
        paint_button(
            ui,
            decrement_rect,
            decrement_painted_rect,
            &decrement_response,
            decrement_enabled,
            "−",
            true,
        );
        paint_button(
            ui,
            increment_rect,
            increment_painted_rect,
            &increment_response,
            increment_enabled,
            "+",
            false,
        );

        // Return one response covering typing and both step actions
        let mut response = outer_response
            .union(editor_response)
            .union(decrement_response)
            .union(increment_response);
        if stepped {
            response.mark_changed();
        }
        NumericStepperOutput {
            response,
            invalid: editor_output.invalid,
        }
    }
}

impl<V: StepperValue> Widget for NumericStepper<'_, V> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

/// Result of rendering a [`NumericStepper`] with validation status.
pub struct NumericStepperOutput {
    /// Combined interaction response for the editor and both buttons.
    pub response: Response,
    /// Whether the currently displayed numeric draft is invalid.
    pub invalid: bool,
}

/// A whole-number editor with attached decrement and increment buttons.
pub type IntegerStepper<'a> = NumericStepper<'a, i64>;

/// Result of rendering an [`IntegerStepper`] with validation status.
pub type IntegerStepperOutput = NumericStepperOutput;

/// A floating-point editor with attached decrement and increment buttons.
pub type FloatStepper<'a> = NumericStepper<'a, f64>;

/// Result of rendering a [`FloatStepper`] with validation status.
pub type FloatStepperOutput = NumericStepperOutput;

mod private {
    pub trait Sealed {}

    impl Sealed for i64 {}
    impl Sealed for f64 {}
}

/// A numeric value supported by [`NumericStepper`].
///
/// This trait is sealed and implemented for `i64` and `f64`.
pub trait StepperValue: private::Sealed + Copy + Display + FromStr + PartialOrd {
    /// Smallest finite value supported by the type.
    const MIN: Self;
    /// Largest finite value supported by the type.
    const MAX: Self;
    /// Additive identity used to validate positive steps.
    const ZERO: Self;
    /// Default increment used by [`NumericStepper::new`].
    const ONE: Self;

    /// Reports whether the value is finite.
    ///
    /// Returns `true` for every integer and for finite floating-point values.
    fn is_finite(self) -> bool;

    /// Normalizes a parsed or externally supplied value.
    ///
    /// Returns the value when it can be edited, or `None` for unsupported
    /// values such as floating-point `NaN`.
    fn normalized(self) -> Option<Self>;

    /// Clamps the value to an inclusive finite range.
    ///
    /// Returns the nearest value within `minimum..=maximum`.
    fn clamped(self, minimum: Self, maximum: Self) -> Self;

    /// Applies one decrement without crossing the lower bound.
    ///
    /// Returns the decremented value or `minimum` when subtraction would
    /// underflow or pass the lower bound.
    fn decremented(self, step: Self, minimum: Self) -> Self;

    /// Applies one increment without crossing the upper bound.
    ///
    /// Returns the incremented value or `maximum` when addition would overflow
    /// or pass the upper bound.
    fn incremented(self, step: Self, maximum: Self) -> Self;
}

impl StepperValue for i64 {
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
    const ZERO: Self = 0;
    const ONE: Self = 1;

    fn is_finite(self) -> bool {
        true
    }

    fn normalized(self) -> Option<Self> {
        Some(self)
    }

    fn clamped(self, minimum: Self, maximum: Self) -> Self {
        self.clamp(minimum, maximum)
    }

    fn decremented(self, step: Self, minimum: Self) -> Self {
        self.saturating_sub(step).max(minimum)
    }

    fn incremented(self, step: Self, maximum: Self) -> Self {
        self.saturating_add(step).min(maximum)
    }
}

impl StepperValue for f64 {
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
    const ZERO: Self = 0.;
    const ONE: Self = 1.;

    fn is_finite(self) -> bool {
        self.is_finite()
    }

    fn normalized(self) -> Option<Self> {
        (!self.is_nan()).then_some(self)
    }

    fn clamped(self, minimum: Self, maximum: Self) -> Self {
        self.clamp(minimum, maximum)
    }

    fn decremented(self, step: Self, minimum: Self) -> Self {
        (self - step).max(minimum)
    }

    fn incremented(self, step: Self, maximum: Self) -> Self {
        (self + step).min(maximum)
    }
}

fn button_response(ui: &mut Ui, rect: Rect, id: Id, enabled: bool, label: &'static str) -> Response {
    // Keep disabled regions hoverable while only enabled regions accept clicks
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let response = ui.interact(rect, id, sense);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, label));
    if enabled {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        response
    }
}

fn paint_background(ui: &Ui, background: ShapeIdx, rect: Rect) {
    // Paint one inactive app-style base across the complete compound rectangle
    let style = ui.app_style();
    let fill = if !ui.is_enabled() {
        style.widgets.noninteractive.bg_fill
    } else {
        style.text_edit.inactive_fill
    };
    ui.painter()
        .set(background, Shape::rect_filled(rect, CORNER_RADIUS, fill));
}

fn paint_editor_highlight(ui: &Ui, highlight: ShapeIdx, rect: Rect, response: &Response, invalid: bool) {
    // Highlight only the central text region for validation and interaction
    let style = ui.app_style();
    let fill = if invalid {
        Some(style.text_edit.invalid_fill)
    } else if response.has_focus() {
        Some(style.text_edit.active_fill)
    } else if response.hovered() {
        Some(style.text_edit.hover_fill)
    } else {
        None
    };
    if let Some(fill) = fill {
        ui.painter().set(highlight, Shape::rect_filled(rect, 0, fill));
    }
}

fn paint_button(
    ui: &Ui,
    interaction_rect: Rect,
    painted_rect: Rect,
    response: &Response,
    enabled: bool,
    glyph: &'static str,
    left: bool,
) {
    let style = ui.app_style();
    let interaction = if !enabled {
        &style.widgets.noninteractive
    } else if response.is_pointer_button_down_on() {
        &style.widgets.active
    } else if response.hovered() {
        &style.widgets.hovered
    } else {
        &style.widgets.inactive
    };

    // Overlay only interactive button states while retaining the shared base fill
    if enabled && (response.hovered() || response.is_pointer_button_down_on()) {
        let corners = if left {
            CornerRadius {
                nw: CORNER_RADIUS,
                ne: 0,
                sw: CORNER_RADIUS,
                se: 0,
            }
        } else {
            CornerRadius {
                nw: 0,
                ne: CORNER_RADIUS,
                sw: 0,
                se: CORNER_RADIUS,
            }
        };
        ui.painter().rect_filled(painted_rect, corners, interaction.bg_fill);
    }

    // Center the sign glyph in its fixed square button region
    let font_id = TextStyle::Button.resolve(ui.style());
    ui.painter().text(
        interaction_rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        font_id,
        interaction.fg_stroke_color,
    );
}
