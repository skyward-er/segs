use std::time::Duration;

use crate::{error::ErrInstrument, mavlink::reflection::plottable_fields};

use super::{
    LineSettings, PlotSettings,
    fields::{XPlotField, YPlotField},
};

#[profiling::function]
pub fn sources_window(ui: &mut egui::Ui, plot_settings: &mut PlotSettings) {
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

    let fields = plottable_fields();
    let y_fields: Vec<YPlotField> = fields.iter().map(|f| f.clone().into()).collect();
    let mut x_fields = vec![XPlotField::MsgReceiptTimestamp];
    x_fields.extend(fields.into_iter().map(|f| f.into()));

    // Reset fields if something changed that invalidates old selections
    if data_settings_digest != plot_settings.data_digest() {
        plot_settings.clear_fields();
    }

    // Validate current x_field
    let x_field = &plot_settings.x_field;
    let new_x = if x_fields.iter().any(|f| f == x_field) {
        x_field.to_owned()
    } else {
        XPlotField::MsgReceiptTimestamp
    };
    plot_settings.x_field = new_x;

    // X axis search + combo box
    let x_field = &mut plot_settings.x_field;
    let x_id = ui.auto_id_with("x_search");
    let mut x_search: String = ui.ctx().memory(|m| m.data.get_temp(x_id).unwrap_or_default());
    ui.add(egui::TextEdit::singleline(&mut x_search).hint_text("Filter X axis…").desired_width(f32::INFINITY));
    ui.ctx().memory_mut(|m| m.data.insert_temp(x_id, x_search.clone()));
    let x_lower = x_search.to_lowercase();
    egui::ComboBox::from_label("X Axis")
        .selected_text(x_field.name())
        .show_ui(ui, |ui| {
            for f in x_fields.iter().filter(|f| f.name().to_lowercase().contains(&x_lower)) {
                ui.selectable_value(x_field, f.to_owned(), f.name());
            }
        });

    // Retain only valid y_fields
    plot_settings
        .y_fields
        .retain(|(field, _)| y_fields.iter().any(|f: &YPlotField| f == field));

    // Auto-select first field if empty and fields exist
    if plot_settings.y_fields.is_empty() && y_fields.len() > 1 {
        plot_settings.add_field(y_fields[0].clone());
    }

    let plot_lines_len = plot_settings.y_fields.len();
    let mut delete_idx: Option<usize> = None;
    egui::Grid::new(ui.auto_id_with("y_axis"))
        .num_columns(4)
        .spacing([10.0, 2.5])
        .show(ui, |ui| {
            for (i, (field, line_settings)) in plot_settings.y_fields[..].iter_mut().enumerate() {
                let LineSettings { width, color } = line_settings;
                let widget_label = if plot_lines_len > 1 {
                    format!("Y Axis {}", i + 1)
                } else {
                    "Y Axis".to_owned()
                };
                let y_id = ui.auto_id_with(format!("y_search_{i}"));
                let mut y_search: String =
                    ui.ctx().memory(|m| m.data.get_temp(y_id).unwrap_or_default());
                ui.add(egui::TextEdit::singleline(&mut y_search).hint_text("Filter Y axis…").desired_width(f32::INFINITY));
                ui.ctx().memory_mut(|m| m.data.insert_temp(y_id, y_search.clone()));
                let y_lower = y_search.to_lowercase();
                egui::ComboBox::from_label(widget_label)
                    .selected_text(field.name())
                    .show_ui(ui, |ui| {
                        for f in y_fields.iter().filter(|f| f.name().to_lowercase().contains(&y_lower)) {
                            ui.selectable_value(field, f.to_owned(), f.name());
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
                if ui.button("✕").clicked() {
                    delete_idx = Some(i);
                }
                ui.end_row();
            }
        });
    if let Some(i) = delete_idx {
        plot_settings.y_fields.remove(i);
    }

    if y_fields.len().saturating_sub(plot_lines_len + 1) > 0
        && ui
            .button("Add Y Axis")
            .on_hover_text("Add another Y axis")
            .clicked()
    {
        let next_field = y_fields
            .iter()
            .find(|field| !plot_settings.y_fields.iter().any(|(f, _)| f == *field))
            .log_unwrap();
        plot_settings.add_field(next_field.to_owned());
    }
}
