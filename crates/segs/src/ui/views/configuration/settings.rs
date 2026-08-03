use egui::{ComboBox, Grid, Ui};
use segs_ui::widgets::{UiWidgetExt, text::TextEdit};

use crate::ui::{
    widget_settings::WidgetSetting,
    widgets::{WidgetData, WidgetTrait},
};

pub fn show(ui: &mut Ui, widget: Option<&mut WidgetData>) {
    let Some(widget) = widget else {
        ui.weak("Select a widget to edit its settings.");
        return;
    };

    let widget_id = widget.id;
    let settings = widget.variant.settings();
    if settings.is_empty() {
        ui.weak("This widget has no settings.");
        return;
    }

    ui.push_id(widget_id.with("_settings"), |ui| {
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
    });
}
