mod stream_selector;

use std::ops::RangeInclusive;

use egui::{Align, ComboBox, Layout, RichText, TextStyle, Ui, color_picker, vec2};
use segs_ui::{
    style::CtxStyleExt,
    widgets::{
        Separator, UiWidgetExt,
        text::{FloatStepper, IntegerStepper, TextEdit, ValueEdit, default_singleline_height},
    },
};

use crate::{
    dataflow::adapter::DataAdapterInstance,
    ui::{
        widget_settings::{WidgetDataSetting, WidgetSetting},
        widgets::{WidgetData, WidgetTrait},
    },
};

const DATA_SETTINGS_SEPARATOR_SPACING: f32 = 10.;
const SETTINGS_COLUMN_SPACING: f32 = 8.;
const SETTINGS_ROW_SPACING: f32 = 8.;

pub fn show(ui: &mut Ui, widget: Option<&mut WidgetData>, adapter: Option<&DataAdapterInstance>) {
    let Some(widget) = widget else {
        ui.weak("Select a widget to edit its settings.");
        return;
    };

    let widget_id = widget.id;
    ui.push_id(widget_id.with("_settings"), |ui| {
        let has_data_settings = {
            let data_settings = widget.variant.data_settings();
            let has_data_settings = !data_settings.is_empty();
            show_data_settings(ui, data_settings, adapter);
            has_data_settings
        };

        let settings = widget.variant.settings();
        if has_data_settings && !settings.is_empty() {
            let horizontal_margin = ui.spacing().window_margin.leftf();
            ui.add(
                Separator::default()
                    .spacing(DATA_SETTINGS_SEPARATOR_SPACING)
                    .grow(horizontal_margin),
            );
        }

        if settings.is_empty() {
            if !has_data_settings {
                ui.weak("This widget has no settings.");
            }
        } else {
            show_widget_settings(ui, settings);
        }
    });
}

fn show_data_settings(ui: &mut Ui, settings: Vec<WidgetDataSetting<'_>>, adapter: Option<&DataAdapterInstance>) {
    for setting in settings {
        let setting_id = setting.id();
        ui.push_id(setting_id, |ui| match setting {
            WidgetDataSetting::SingleStream { label, stream, .. } => {
                stream_selector::show(ui, label, stream, adapter);
            }
            WidgetDataSetting::MultipleStreams {
                label, streams, names, ..
            } => {
                stream_selector::show_multiple(ui, label, streams, names, adapter);
            }
        });
    }
}

fn show_widget_settings(ui: &mut Ui, settings: Vec<WidgetSetting<'_>>) {
    // Measure labels once so top-aligned rows retain one shared value column
    let label_width = settings_label_width(ui, &settings);
    let additional_row_spacing = (SETTINGS_ROW_SPACING - ui.spacing().item_spacing.y).max(0.);
    let setting_count = settings.len();

    for (index, setting) in settings.into_iter().enumerate() {
        let label = setting_label(&setting);
        let label_height = setting_control_height(ui, &setting);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = SETTINGS_COLUMN_SPACING;
            ui.allocate_ui_with_layout(
                vec2(label_width, label_height),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.label(label);
                },
            );
            show_widget_setting(ui, setting);
        });

        if index + 1 < setting_count {
            ui.add_space(additional_row_spacing);
        }
    }
}

/// Renders the editable value for one widget setting.
fn show_widget_setting(ui: &mut Ui, setting: WidgetSetting<'_>) {
    let setting_id = setting.id();
    match setting {
        WidgetSetting::Checkbox { value, .. } => {
            ui.push_id(setting_id, |ui| ui.check(value));
        }
        WidgetSetting::ComboBox { selected, options, .. } => {
            let selected_label = options
                .iter()
                .find(|option| option.key == selected)
                .map_or(selected.as_str(), |option| option.label);
            ComboBox::from_id_salt(setting_id)
                .width(ui.available_width())
                .truncate()
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    for option in options {
                        ui.selectable_value(selected, option.key.to_owned(), option.label);
                    }
                });
        }
        WidgetSetting::TextBox { value, .. } => {
            let width = ui.available_width();
            ui.add(TextEdit::singleline(value).id_source(setting_id).desired_width(width));
        }
        WidgetSetting::Color { value, .. } => {
            // Keep widget colors opaque even when loading a manually edited layout
            *value = value.to_opaque();
            ui.push_id(setting_id, |ui| {
                color_picker::color_edit_button_srgba(ui, value, color_picker::Alpha::Opaque)
            });
        }
        WidgetSetting::Integer {
            value,
            range,
            step,
            desired_width,
            ..
        } => {
            show_integer_setting(ui, setting_id, value, range, step, desired_width);
        }
        WidgetSetting::Float {
            value,
            range,
            step,
            desired_width,
            ..
        } => {
            show_float_setting(ui, setting_id, value, range, step, desired_width);
        }
    }
}

/// Returns the widest setting label in logical points.
fn settings_label_width(ui: &Ui, settings: &[WidgetSetting<'_>]) -> f32 {
    let font_id = TextStyle::Body.resolve(ui.style());
    let color = ui.visuals().text_color();
    settings.iter().fold(0., |width, setting| {
        let label = setting_label(setting).to_owned();
        width.max(ui.painter().layout_no_wrap(label, font_id.clone(), color).size().x)
    })
}

/// Returns the user-facing label for a setting.
fn setting_label(setting: &WidgetSetting<'_>) -> &'static str {
    match setting {
        WidgetSetting::Checkbox { label, .. }
        | WidgetSetting::ComboBox { label, .. }
        | WidgetSetting::TextBox { label, .. }
        | WidgetSetting::Color { label, .. }
        | WidgetSetting::Integer { label, .. }
        | WidgetSetting::Float { label, .. } => label,
    }
}

/// Returns the height against which the setting label is vertically centered.
fn setting_control_height(ui: &Ui, setting: &WidgetSetting<'_>) -> f32 {
    match setting {
        WidgetSetting::TextBox { .. } | WidgetSetting::Integer { .. } | WidgetSetting::Float { .. } => {
            default_singleline_height(ui)
        }
        WidgetSetting::Checkbox { .. } | WidgetSetting::ComboBox { .. } | WidgetSetting::Color { .. } => {
            ui.spacing().interact_size.y
        }
    }
}

fn show_integer_setting(
    ui: &mut Ui,
    setting_id: &'static str,
    value: &mut i64,
    range: RangeInclusive<i64>,
    step: Option<i64>,
    desired_width: Option<f32>,
) {
    let hint = integer_range_hint(&range);
    ui.vertical(|ui| {
        // Respect compact settings without overflowing the available value column
        let width = desired_width
            .unwrap_or_else(|| ui.available_width())
            .min(ui.available_width());
        let invalid = if let Some(step) = step {
            IntegerStepper::new(value)
                .range(range)
                .step(step)
                .id_salt(setting_id)
                .desired_width(width)
                .show(ui)
                .invalid
        } else {
            ValueEdit::new(value)
                .id(setting_id)
                .with_width(width)
                .update_while_editing(true)
                .range(range)
                .show_with_status(ui)
                .invalid
        };
        if invalid {
            show_range_hint(ui, hint);
        }
    });
}

fn show_float_setting(
    ui: &mut Ui,
    setting_id: &'static str,
    value: &mut f64,
    range: RangeInclusive<f64>,
    step: Option<f64>,
    desired_width: Option<f32>,
) {
    let hint = float_range_hint(&range);
    ui.vertical(|ui| {
        // Use attached controls only for floating-point settings with an explicit step
        let width = desired_width
            .unwrap_or_else(|| ui.available_width())
            .min(ui.available_width());
        let invalid = if let Some(step) = step {
            FloatStepper::new(value)
                .range(range)
                .step(step)
                .id_salt(setting_id)
                .desired_width(width)
                .show(ui)
                .invalid
        } else {
            ValueEdit::new(value)
                .id(setting_id)
                .with_width(width)
                .update_while_editing(true)
                .normalize(|value| (!value.is_nan()).then_some(value))
                .range(range)
                .show_with_status(ui)
                .invalid
        };
        if invalid {
            show_range_hint(ui, hint);
        }
    });
}

/// Displays a numeric range hint using the established validation style.
fn show_range_hint(ui: &mut Ui, hint: String) {
    ui.label(RichText::new(hint).size(10.).color(ui.app_style().error_fg_color));
}

/// Returns a concise range hint for an integer setting.
fn integer_range_hint(range: &RangeInclusive<i64>) -> String {
    let minimum = *range.start();
    let maximum = *range.end();
    if minimum == i64::MIN && maximum == i64::MAX {
        "Enter a whole number".to_owned()
    } else if maximum == i64::MAX {
        format!("Enter a value of at least {minimum}")
    } else if minimum == i64::MIN {
        format!("Enter a value of at most {maximum}")
    } else {
        format!("Enter a value from {minimum} to {maximum}")
    }
}

/// Returns a concise range hint for a floating-point setting.
fn float_range_hint(range: &RangeInclusive<f64>) -> String {
    let minimum = *range.start();
    let maximum = *range.end();
    if minimum == f64::MIN && maximum == f64::MAX {
        "Enter a finite number".to_owned()
    } else if minimum == f64::from_bits(1) && maximum == f64::MAX {
        "Enter a value greater than 0".to_owned()
    } else if maximum == f64::MAX {
        format!("Enter a value of at least {minimum}")
    } else if minimum == f64::MIN {
        format!("Enter a value of at most {maximum}")
    } else {
        format!("Enter a value from {minimum} to {maximum}")
    }
}
