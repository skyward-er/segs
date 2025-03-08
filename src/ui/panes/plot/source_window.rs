use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::{MAVLINK_PROFILE, ui::panes::plot::FieldWithID};

use crate::error::ErrInstrument;

use super::{LineSettings, PlotSettings};

#[profiling::function]
pub fn sources_window(ui: &mut egui::Ui, plot_settings: &mut PlotSettings) {
    let settings_hash = ChangeTracker::record_initial_state(&plot_settings);

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
    if settings_hash.has_changed(plot_settings) {
        plot_settings.clear_fields();
    }

    // check fields and assign a default field_x and field_y once the msg is changed
    let fields: Vec<FieldWithID> = MAVLINK_PROFILE
        .get_plottable_fields(plot_settings.get_msg_id())
        .log_expect("Invalid message id")
        .into_iter()
        .map(|f| f.into())
        .collect::<Vec<_>>();
    // get the first field that is in the list of fields or the previous if valid
    let x_field = plot_settings.get_x_field();
    let new_field_x = fields
        .iter()
        .any(|f| f == x_field)
        .then(|| x_field.to_owned())
        .or(fields.first().map(|s| s.to_owned()));

    // if there are no fields, reset the field_x and plot_lines
    let Some(new_field_x) = new_field_x else {
        plot_settings.clear_fields();
        return;
    };
    // update the field_x
    plot_settings.set_x_field(new_field_x);
    let x_field = plot_settings.get_mut_x_field();

    // if fields are valid, show the combo boxes for the x_axis
    egui::ComboBox::from_label("X Axis")
        .selected_text(&x_field.field.name)
        .show_ui(ui, |ui| {
            for msg in fields.iter() {
                ui.selectable_value(x_field, msg.to_owned(), &msg.field.name);
            }
        });

    // populate the plot_lines with the first field if it is empty and there are more than 1 fields
    if plot_settings.fields_empty() && fields.len() > 1 {
        plot_settings.add_field(fields[1].to_owned());
    }

    // check how many fields are left and how many are selected
    let plot_lines_len = plot_settings.fields_len();
    egui::Grid::new(ui.auto_id_with("y_axis"))
        .num_columns(3)
        .spacing([10.0, 2.5])
        .show(ui, |ui| {
            for (i, (field, line_settings)) in
                plot_settings.get_mut_y_fields().into_iter().enumerate()
            {
                let LineSettings { width, color } = line_settings;
                let widget_label = if plot_lines_len > 1 {
                    format!("Y Axis {}", i + 1)
                } else {
                    "Y Axis".to_owned()
                };
                egui::ComboBox::from_label(widget_label)
                    .selected_text(&field.field.name)
                    .show_ui(ui, |ui| {
                        for msg in fields.iter() {
                            ui.selectable_value(field, msg.to_owned(), &msg.field.name);
                        }
                    });
                ui.color_edit_button_srgba(color);
                ui.add(egui::DragValue::new(width).speed(0.1).suffix(" pt"))
                    .on_hover_text("Width of the line in points");
                ui.end_row();
            }
        });

    // if we have fields left, show the add button
    if fields.len().saturating_sub(plot_lines_len + 1) > 0
        && ui
            .button("Add Y Axis")
            .on_hover_text("Add another Y axis")
            .clicked()
    {
        let next_field = fields
            .iter()
            .find(|f| !plot_settings.contains_field(f))
            .log_unwrap();
        plot_settings.add_field(next_field.to_owned());
    }
}

pub struct ChangeTracker {
    integrity_digest: u64,
}

impl ChangeTracker {
    pub fn record_initial_state<T: Hash>(state: &T) -> Self {
        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        let integrity_digest = hasher.finish();
        Self { integrity_digest }
    }

    pub fn has_changed<T: Hash>(&self, state: &T) -> bool {
        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        self.integrity_digest != hasher.finish()
    }
}

// pub struct SourceSettings<'a> {
//     msg_sources: &'a mut PlotSettings,
// }

// impl<'a> SourceSettings<'a> {
//     pub fn new(
//         msg_sources: &'a mut PlotSettings,
//         line_settings: &'a mut Vec<LineSettings>,
//     ) -> Self {
//         Self {
//             old_msg_sources: msg_sources.clone(),
//             msg_sources,
//             line_settings,
//         }
//     }

//     pub fn are_sources_changed(&self) -> bool {
//         self.msg_sources != &self.old_msg_sources
//     }

//     pub fn fields_empty(&self) -> bool {
//         self.msg_sources.y_field_ids.is_empty()
//     }

//     fn get_msg_id(&self) -> u32 {
//         self.msg_sources.plot_message_id
//     }

//     fn get_x_field_id(&self) -> usize {
//         self.msg_sources.x_field_id
//     }

//     fn get_mut_msg_id(&mut self) -> &mut u32 {
//         &mut self.msg_sources.plot_message_id
//     }

//     fn get_mut_x_field_id(&mut self) -> &mut usize {
//         &mut self.msg_sources.x_field_id
//     }

//     fn set_x_field_id(&mut self, field_id: usize) {
//         self.msg_sources.x_field_id = field_id;
//     }

//     fn fields_len(&self) -> usize {
//         self.msg_sources.y_field_ids.len()
//     }

//     fn is_msg_id_changed(&self) -> bool {
//         self.msg_sources.plot_message_id != self.old_msg_sources.plot_message_id
//     }

//     fn contains_field(&self, field_id: usize) -> bool {
//         self.msg_sources.y_field_ids.contains(&field_id)
//     }

//     fn sync_fields_with_lines(&mut self) {
//         self.msg_sources.y_field_ids = self
//             .line_settings
//             .iter()
//             .map(|ls| ls.field_id.clone())
//             .collect();
//     }

//     fn add_field(&mut self, field_id: usize) {
//         self.line_settings.push(LineSettings::new(field_id));
//         self.msg_sources.y_field_ids.push(field_id);
//     }

//     fn clear_fields(&mut self) {
//         self.msg_sources.y_field_ids.clear();
//         self.line_settings.clear();
//         self.msg_sources.x_field_id = 0;
//     }
// }
