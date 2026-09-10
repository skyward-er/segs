// TODO: remove when ComboBox is used
#![allow(unused)]

use std::ops::RangeInclusive;

use serde::de;

use crate::dataflow::StreamKey;

/// One selectable value displayed by a widget settings combobox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComboBoxOption {
    /// Stable value stored by the widget.
    pub key: &'static str,
    /// User-facing option label.
    pub label: &'static str,
}

impl ComboBoxOption {
    /// Creates one selectable combobox option.
    ///
    /// Returns an option that stores `key` and displays `label`.
    pub const fn new(key: &'static str, label: &'static str) -> Self {
        Self { key, label }
    }
}

/// A widget configuration field rendered by the standard settings panel.
///
/// Values are borrowed directly from the widget configuration, so edits made
/// by the panel are immediately reflected by the widget.
pub enum WidgetSetting<'a> {
    /// A boolean setting rendered as a checkbox.
    Checkbox {
        /// Stable interaction identifier.
        id: &'static str,
        /// User-facing setting label.
        label: &'static str,
        /// Boolean value edited in place.
        value: &'a mut bool,
    },
    /// A string setting selected from a fixed list.
    ComboBox {
        /// Stable interaction identifier.
        id: &'static str,
        /// User-facing setting label.
        label: &'static str,
        /// Selected option key edited in place.
        selected: &'a mut String,
        /// Options available for selection.
        options: &'static [ComboBoxOption],
    },
    /// A free-form string setting.
    TextBox {
        /// Stable interaction identifier.
        id: &'static str,
        /// User-facing setting label.
        label: &'static str,
        /// String value edited in place.
        value: &'a mut String,
    },
    /// A whole-number setting rendered with decrement and increment controls.
    Integer {
        /// Stable interaction identifier.
        id: &'static str,
        /// User-facing setting label.
        label: &'static str,
        /// Integer value edited in place.
        value: &'a mut i64,
        /// Inclusive limits applied to typed and stepped values.
        range: RangeInclusive<i64>,
        /// Positive amount applied by each decrement or increment.
        step: i64,
        /// Optional total control width in logical points.
        desired_width: Option<f32>,
    },
    /// A floating-point setting rendered as a validated numeric field.
    Float {
        /// Stable interaction identifier.
        id: &'static str,
        /// User-facing setting label.
        label: &'static str,
        /// Floating-point value edited in place.
        value: &'a mut f64,
        /// Inclusive finite limits applied to typed values.
        range: RangeInclusive<f64>,
        /// Optional positive amount applied by decrement and increment controls.
        step: Option<f64>,
    },
}

impl<'a> WidgetSetting<'a> {
    /// Creates a checkbox setting.
    ///
    /// Returns a setting that edits `value` directly.
    pub fn checkbox(id: &'static str, label: &'static str, value: &'a mut bool) -> Self {
        Self::Checkbox { id, label, value }
    }

    /// Creates a fixed-option string setting.
    ///
    /// Returns a setting that edits `selected` using `options`.
    pub fn combo_box(
        id: &'static str,
        label: &'static str,
        selected: &'a mut String,
        options: &'static [ComboBoxOption],
    ) -> Self {
        Self::ComboBox {
            id,
            label,
            selected,
            options,
        }
    }

    /// Creates a free-form string setting.
    ///
    /// Returns a setting that edits `value` directly.
    pub fn text_box(id: &'static str, label: &'static str, value: &'a mut String) -> Self {
        Self::TextBox { id, label, value }
    }

    /// Creates a bounded whole-number setting.
    ///
    /// Returns a setting that changes `value` by `step` and clamps edits to
    /// `range`. The step must be positive and the range must not be empty.
    pub fn integer(
        id: &'static str,
        label: &'static str,
        value: &'a mut i64,
        range: RangeInclusive<i64>,
        step: i64,
    ) -> Self {
        debug_assert!(step > 0, "integer setting step must be positive");
        debug_assert!(!range.is_empty(), "integer setting range must not be empty");
        Self::Integer {
            id,
            label,
            value,
            range,
            step,
            desired_width: None,
        }
    }

    /// Creates a bounded whole-number setting with an explicit total width.
    ///
    /// Returns a setting that changes `value` by `step`, clamps edits to
    /// `range`, and requests `desired_width` logical points. The step and width
    /// must be positive and the range must not be empty.
    pub fn integer_with_width(
        id: &'static str,
        label: &'static str,
        value: &'a mut i64,
        range: RangeInclusive<i64>,
        step: i64,
        desired_width: f32,
    ) -> Self {
        debug_assert!(step > 0, "integer setting step must be positive");
        debug_assert!(!range.is_empty(), "integer setting range must not be empty");
        debug_assert!(desired_width > 0., "integer setting width must be positive");
        Self::Integer {
            id,
            label,
            value,
            range,
            step,
            desired_width: Some(desired_width),
        }
    }

    /// Creates a bounded floating-point setting.
    ///
    /// Returns a setting that edits `value` and clamps finite input to `range`.
    /// Both range endpoints must be finite and the range must not be empty.
    pub fn float(id: &'static str, label: &'static str, value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        debug_assert!(range.start().is_finite() && range.end().is_finite());
        debug_assert!(!range.is_empty(), "float setting range must not be empty");
        Self::Float {
            id,
            label,
            value,
            range,
            step: None,
        }
    }

    /// Creates a bounded floating-point setting with step controls.
    ///
    /// Returns a setting that changes `value` by `step` and clamps edits to
    /// `range`. The step and range endpoints must be finite, the step must be
    /// positive, and the range must not be empty.
    pub fn float_stepper(
        id: &'static str,
        label: &'static str,
        value: &'a mut f64,
        range: RangeInclusive<f64>,
        step: f64,
    ) -> Self {
        debug_assert!(
            step.is_finite() && step > 0.,
            "float setting step must be finite and positive"
        );
        debug_assert!(range.start().is_finite() && range.end().is_finite());
        debug_assert!(!range.is_empty(), "float setting range must not be empty");
        Self::Float {
            id,
            label,
            value,
            range,
            step: Some(step),
        }
    }

    /// Returns the stable interaction identifier for this setting.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Checkbox { id, .. }
            | Self::ComboBox { id, .. }
            | Self::TextBox { id, .. }
            | Self::Integer { id, .. }
            | Self::Float { id, .. } => id,
        }
    }
}

/// A widget configuration field that selects data streams.
///
/// Data settings are kept separate from regular widget settings so the
/// settings panel can render them separately.
pub enum WidgetDataSetting<'a> {
    SingleStream {
        id: &'static str,
        label: &'static str,
        stream: &'a mut Option<StreamKey>,
    },
    MultipleStreams {
        id: &'static str,
        label: &'static str,
        streams: &'a mut Vec<StreamKey>,
        names: Option<&'a mut Vec<String>>,
    },
}

impl<'a> WidgetDataSetting<'a> {
    pub fn single_stream(id: &'static str, label: &'static str, stream: &'a mut Option<StreamKey>) -> Self {
        Self::SingleStream { id, label, stream }
    }

    pub fn multiple_streams(id: &'static str, label: &'static str, streams: &'a mut Vec<StreamKey>) -> Self {
        Self::MultipleStreams {
            id,
            label,
            streams,
            names: None,
        }
    }

    /// Creates a multiple-stream setting with parallel persistent display names.
    pub fn multiple_streams_with_names(
        id: &'static str,
        label: &'static str,
        streams: &'a mut Vec<StreamKey>,
        names: &'a mut Vec<String>,
    ) -> Self {
        Self::MultipleStreams {
            id,
            label,
            streams,
            names: Some(names),
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::SingleStream { id, .. } | Self::MultipleStreams { id, .. } => id,
        }
    }

    /// Assigns a stream to this setting when the widget has not configured one.
    pub fn set_stream_if_empty(&mut self, key: StreamKey) {
        match self {
            Self::SingleStream { stream, .. } => {
                stream.get_or_insert(key);
            }
            Self::MultipleStreams { streams, names, .. } => {
                if streams.is_empty() {
                    streams.push(key);
                    if let Some(names) = names {
                        names.push("Stream".to_owned());
                    }
                }
            }
        }
    }
}
