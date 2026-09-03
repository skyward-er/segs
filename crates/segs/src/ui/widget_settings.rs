// TODO: remove when ComboBox is used
#![allow(unused)]

use crate::dataflow::StreamKey;

/// One selectable value displayed by a widget settings combobox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComboBoxOption {
    pub key: &'static str,
    pub label: &'static str,
}

impl ComboBoxOption {
    pub const fn new(key: &'static str, label: &'static str) -> Self {
        Self { key, label }
    }
}

/// A widget configuration field rendered by the standard settings panel.
///
/// Values are borrowed directly from the widget configuration, so edits made
/// by the panel are immediately reflected by the widget.
pub enum WidgetSetting<'a> {
    Checkbox {
        id: &'static str,
        label: &'static str,
        value: &'a mut bool,
    },
    ComboBox {
        id: &'static str,
        label: &'static str,
        selected: &'a mut String,
        options: &'static [ComboBoxOption],
    },
    TextBox {
        id: &'static str,
        label: &'static str,
        value: &'a mut String,
    },
}

impl<'a> WidgetSetting<'a> {
    pub fn checkbox(id: &'static str, label: &'static str, value: &'a mut bool) -> Self {
        Self::Checkbox { id, label, value }
    }

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

    pub fn text_box(id: &'static str, label: &'static str, value: &'a mut String) -> Self {
        Self::TextBox { id, label, value }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::Checkbox { id, .. } | Self::ComboBox { id, .. } | Self::TextBox { id, .. } => id,
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
