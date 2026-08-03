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
