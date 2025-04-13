use std::time::Duration;

use crate::{MAVLINK_PROFILE, error::ErrInstrument};

use super::{
    LineSettings, PlotSettings,
    fields::{XPlotField, YPlotField},
};

#[profiling::function]
pub fn sources_window(ui: &mut egui::Ui, plot_settings: &mut PlotSettings) {
    // select how many points are shown on the plot
    let mut points_lifespan_sec = plot_settings.points_lifespan.as_secs();
    ui.horizontal(|ui| {
        let res1 = ui.add(egui::Label::new("Points Lifespan: "));
        let res2 = ui.add(
            egui::DragValue::new(&mut points_lifespan_sec)
                .range(5..=1800)
                .speed(1)
                .update_while_editing(false)
                .suffix(" seconds"),
        );
        res1.union(res2)
    })
    .inner
    .on_hover_text("How long the data is shown on the plot");
    plot_settings.points_lifespan = Duration::from_secs(points_lifespan_sec);

    ui.add_sized([250., 10.], egui::Separator::default());

    let data_settings_digest = plot_settings.data_digest();
    // extract the msg name from the id to show it in the combo box
    let msg_name = MAVLINK_PROFILE
        .get_msg(plot_settings.plot_message_id)
        .map(|m| m.name.clone())
        .unwrap_or_default();

    // show the first combo box with the message name selection
    egui::ComboBox::from_label("Message Kind")
        .selected_text(msg_name)
        .show_ui(ui, |ui| {
            for msg in MAVLINK_PROFILE.get_sorted_msgs() {
                ui.selectable_value(&mut plot_settings.plot_message_id, msg.id, &msg.name);
            }
        });

    // reset fields if the message is changed
    if data_settings_digest != plot_settings.data_digest() {
        plot_settings.clear_fields();
    }

    // check fields and assign a default field_x and field_y once the msg is changed
    let fields = MAVLINK_PROFILE
        .get_plottable_fields(plot_settings.plot_message_id)
        .log_expect("Invalid message id");
    let mut x_fields = vec![XPlotField::MsgReceiptTimestamp];
    let y_fields = fields
        .clone()
        .into_iter()
        .map(|f| f.into())
        .collect::<Vec<_>>();
    x_fields.extend(fields.into_iter().map(|f| f.into()));
    // get the first field that is in the list of fields or the previous if valid
    let x_field = &plot_settings.x_field;
    let new_field_x = if x_fields.iter().any(|f| f == x_field) {
        x_field.to_owned()
    } else {
        XPlotField::MsgReceiptTimestamp
    };

    // update the field_x
    plot_settings.x_field = new_field_x;

    // if fields are valid, show the combo boxes for the x_axis
    let x_field = &mut plot_settings.x_field;
    egui::ComboBox::from_label("X Axis")
        .selected_text(x_field.name())
        .show_ui(ui, |ui| {
            for msg in x_fields.iter() {
                ui.selectable_value(x_field, msg.to_owned(), msg.name());
            }
        });

    // retain only the fields that are in y_fields
    plot_settings
        .y_fields
        .retain(|(field, _)| y_fields.iter().any(|f: &YPlotField| f == field));

    // populate the plot_lines with the first field if it is empty and there are more than 1 fields
    if plot_settings.y_fields.is_empty() && y_fields.len() > 1 {
        plot_settings.add_field(y_fields[0].clone());
    }

    // check how many fields are left and how many are selected
    let plot_lines_len = plot_settings.y_fields.len();
    egui::Grid::new(ui.auto_id_with("y_axis"))
        .num_columns(3)
        .spacing([10.0, 2.5])
        .show(ui, |ui| {
            for (i, (field, line_settings)) in plot_settings.y_fields[..].iter_mut().enumerate() {
                let LineSettings { width, color } = line_settings;
                let widget_label = if plot_lines_len > 1 {
                    format!("Y Axis {}", i + 1)
                } else {
                    "Y Axis".to_owned()
                };
                egui::ComboBox::from_label(widget_label)
                    .selected_text(field.name())
                    .show_ui(ui, |ui| {
                        for msg in y_fields.iter() {
                            ui.selectable_value(field, msg.to_owned(), msg.name());
                        }
                    });
                ui.color_edit_button_srgba(color);
                ui.add(
                    egui::DragValue::new(width)
                        .range(0.0..=10.0)
                        .speed(0.02)
                        .suffix(" pt"),
                )
                .on_hover_text("Width of the line in points");
                ui.end_row();
            }
        });

    // if we have fields left, show the add button
    if y_fields.len().saturating_sub(plot_lines_len + 1) > 0
        && ui
            .button("Add Y Axis")
            .on_hover_text("Add another Y axis")
            .clicked()
    {
        // get the first field that is not in the plot_lines
        let next_field = y_fields
            .iter()
            .find(|field| !plot_settings.y_fields.iter().any(|(f, _)| f == *field))
            .log_unwrap();
        plot_settings.add_field(next_field.to_owned());
    }
}
