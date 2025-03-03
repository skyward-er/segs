use crate::{
    MAVLINK_PROFILE,
    mavlink::{MavMessage, Message},
};

use crate::error::ErrInstrument;

use super::{LineSettings, MsgSources};

#[profiling::function]
pub fn sources_window(ui: &mut egui::Ui, plot_settings: &mut SourceSettings) {
    // extract the msg name from the id to show it in the combo box
    let msg_name = MAVLINK_PROFILE
        .get_name_from_id(*plot_settings.get_msg_id())
        .unwrap_or_default();

    // show the first combo box with the message name selection
    egui::ComboBox::from_label("Message Kind")
        .selected_text(msg_name)
        .show_ui(ui, |ui| {
            for msg in MAVLINK_PROFILE.sorted_messages() {
                ui.selectable_value(
                    plot_settings.get_mut_msg_id(),
                    MavMessage::message_id_from_name(msg).log_expect("Invalid message name"),
                    msg,
                );
            }
        });

    // reset fields if the message is changed
    if plot_settings.is_msg_id_changed() {
        plot_settings.clear_fields();
    }

    // check fields and assign a default field_x and field_y once the msg is changed
    let fields = MAVLINK_PROFILE
        .get_plottable_fields_by_id(*plot_settings.get_msg_id())
        .log_expect("Invalid message id");
    // get the first field that is in the list of fields or the previous if valid
    let x_field = plot_settings.get_x_field();
    let new_field_x = fields
        .contains(&x_field)
        .then(|| x_field.to_owned())
        .or(fields.first().map(|s| s.to_string()));

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
        .selected_text(x_field.as_str())
        .show_ui(ui, |ui| {
            for msg in fields.iter() {
                ui.selectable_value(x_field, (*msg).to_owned(), *msg);
            }
        });

    // populate the plot_lines with the first field if it is empty and there are more than 1 fields
    if plot_settings.fields_empty() && fields.len() > 1 {
        plot_settings.add_field(fields[1].to_string());
    }

    // check how many fields are left and how many are selected
    let plot_lines_len = plot_settings.fields_len();
    egui::Grid::new(ui.auto_id_with("y_axis"))
        .num_columns(3)
        .spacing([10.0, 2.5])
        .show(ui, |ui| {
            for (i, line_settings) in plot_settings.line_settings.iter_mut().enumerate() {
                let LineSettings {
                    field,
                    width,
                    color,
                } = line_settings;
                let widget_label = if plot_lines_len > 1 {
                    format!("Y Axis {}", i + 1)
                } else {
                    "Y Axis".to_owned()
                };
                egui::ComboBox::from_label(widget_label)
                    .selected_text(field.as_str())
                    .show_ui(ui, |ui| {
                        for msg in fields.iter() {
                            ui.selectable_value(field, (*msg).to_owned(), *msg);
                        }
                    });
                ui.color_edit_button_srgba(color);
                ui.add(egui::DragValue::new(width).speed(0.1).suffix(" pt"))
                    .on_hover_text("Width of the line in points");
                ui.end_row();
            }
        });
    // Sync changes applied to line_settings with msg_sources
    plot_settings.sync_fields_with_lines();

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
        plot_settings.add_field(next_field.to_string());
    }
}

pub struct SourceSettings<'a> {
    msg_sources: &'a mut MsgSources,
    old_msg_sources: MsgSources,
    line_settings: &'a mut Vec<LineSettings>,
}

impl<'a> SourceSettings<'a> {
    pub fn new(msg_sources: &'a mut MsgSources, line_settings: &'a mut Vec<LineSettings>) -> Self {
        Self {
            old_msg_sources: msg_sources.clone(),
            msg_sources,
            line_settings,
        }
    }

    pub fn are_sources_changed(&self) -> bool {
        self.msg_sources != &self.old_msg_sources
    }

    pub fn fields_empty(&self) -> bool {
        self.msg_sources.y_fields.is_empty()
    }

    fn get_msg_id(&self) -> &u32 {
        &self.msg_sources.msg_id
    }

    fn get_x_field(&self) -> &str {
        &self.msg_sources.x_field
    }

    fn get_mut_msg_id(&mut self) -> &mut u32 {
        &mut self.msg_sources.msg_id
    }

    fn get_mut_x_field(&mut self) -> &mut String {
        &mut self.msg_sources.x_field
    }

    fn set_x_field(&mut self, field: String) {
        self.msg_sources.x_field = field;
    }

    fn fields_len(&self) -> usize {
        self.msg_sources.y_fields.len()
    }

    fn is_msg_id_changed(&self) -> bool {
        self.msg_sources.msg_id != self.old_msg_sources.msg_id
    }

    fn contains_field(&self, field: &str) -> bool {
        self.msg_sources.y_fields.contains(&field.to_owned())
    }

    fn sync_fields_with_lines(&mut self) {
        self.msg_sources.y_fields = self
            .line_settings
            .iter()
            .map(|ls| ls.field.clone())
            .collect();
    }

    fn add_field(&mut self, field: String) {
        self.line_settings.push(LineSettings::new(field.clone()));
        self.msg_sources.y_fields.push(field);
    }

    fn clear_fields(&mut self) {
        self.msg_sources.y_fields.clear();
        self.line_settings.clear();
        self.msg_sources.x_field = "".to_owned();
    }
}
