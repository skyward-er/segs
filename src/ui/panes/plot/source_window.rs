use std::time::Duration;

use crate::{
    error::ErrInstrument, mavlink::reflection::plottable_fields, ui::widgets::filtered_select,
};

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

    // X axis picker
    filtered_select(
        ui,
        "x_axis",
        "X axis",
        &mut plot_settings.x_field,
        &x_fields,
        |f| f.name(),
    );

    ui.add_space(4.0);

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
    for (i, (field, line_settings)) in plot_settings.y_fields[..].iter_mut().enumerate() {
        let LineSettings { width, color } = line_settings;
        let widget_label = if plot_lines_len > 1 {
            format!("Y axis {}", i + 1)
        } else {
            "Y axis".to_owned()
        };

        ui.horizontal(|ui| {
            ui.color_edit_button_srgba(color);
            ui.add(
                egui::DragValue::new(width)
                    .range(0.0..=10.0)
                    .speed(0.02)
                    .suffix(" pt"),
            )
            .on_hover_text("Width of the line in points");
            if ui.button("🗑").on_hover_text("Remove this Y axis").clicked() {
                delete_idx = Some(i);
            }
            filtered_select(
                ui,
                ("y_axis", i),
                &widget_label,
                field,
                &y_fields,
                |f| f.name(),
            );
        });
        ui.add_space(2.0);
    }
    if let Some(i) = delete_idx {
        plot_settings.y_fields.remove(i);
    }

    if y_fields.len().saturating_sub(plot_lines_len + 1) > 0
        && ui
            .button("Add Y axis")
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
