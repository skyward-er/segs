mod stream_selector;

use egui::{ComboBox, Grid, Ui};
use segs_ui::widgets::{Separator, UiWidgetExt, text::TextEdit};

use crate::{
    dataflow::adapter::DataAdapterInstance,
    ui::{
        widget_settings::{WidgetDataSetting, WidgetSetting},
        widgets::{WidgetData, WidgetTrait},
    },
};

const DATA_SETTINGS_SEPARATOR_SPACING: f32 = 10.;

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
    Grid::new("settings_grid")
        .num_columns(2)
        .spacing([8., 8.])
        .show(ui, |ui| {
            for setting in settings {
                let setting_id = setting.id();
                match setting {
                    WidgetSetting::Checkbox { label, value, .. } => {
                        ui.label(label);
                        ui.push_id(setting_id, |ui| ui.check(value));
                    }
                    WidgetSetting::ComboBox {
                        label,
                        selected,
                        options,
                        ..
                    } => {
                        ui.label(label);
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
                    WidgetSetting::TextBox { label, value, .. } => {
                        ui.label(label);
                        let width = ui.available_width();
                        ui.add(TextEdit::singleline(value).id_source(setting_id).desired_width(width));
                    }
                }
                ui.end_row();
            }
        });
}
