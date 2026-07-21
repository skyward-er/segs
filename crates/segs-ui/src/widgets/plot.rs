use egui::{Color32, Id, Ui};
use segs_plot::{Line, Plot, PlotPoint};

#[derive(Clone, Debug)]
pub struct LineSettings {
    pub width: f32,
    pub color: Color32,
}

impl Default for LineSettings {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: Color32::BLUE,
        }
    }
}

pub struct PlotSeries {
    pub id: String,
    pub points: Vec<PlotPoint>,
    pub settings: LineSettings,
}

pub struct PlotOptions {
    pub show_axes: bool,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub auto_bounds: bool,
}

impl Default for PlotOptions {
    fn default() -> Self {
        Self {
            show_axes: true,
            x_label: None,
            y_label: None,
            auto_bounds: true,
        }
    }
}

pub fn plot_widget(ui: &mut Ui, id: impl Into<Id>, series: &[PlotSeries], opts: &PlotOptions) -> egui::Response {
    let mut plot = Plot::new(id.into())
        .auto_bounds([opts.auto_bounds, opts.auto_bounds])
        .allow_boxed_zoom(false)
        .show_axes(opts.show_axes);

    if let Some(label) = &opts.x_label {
        plot = plot.x_axis_label(label);
    }
    if let Some(label) = &opts.y_label {
        plot = plot.y_axis_label(label);
    }

    let response = plot.show(ui, |plot_ui| {
        for s in series {
            let line = Line::new(&s.id, s.points.as_slice())
                .width(s.settings.width)
                .color(s.settings.color);
            plot_ui.line(line);
        }
    });

    response.response
}
