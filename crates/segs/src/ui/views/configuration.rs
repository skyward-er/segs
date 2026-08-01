mod gallery;
mod settings;

use egui::{CentralPanel, Frame, Panel, ScrollArea, Ui};
use segs_ui::{components::panel_header::PanelHeader, style::CtxStyleExt};
use serde::{Deserialize, Serialize};

use crate::app::AppContext;
use crate::ui::components::widget_editor::{WidgetEditor, WidgetEditorResponse};
use crate::ui::components::widget_grid::{WidgetGrid, WidgetGridResponse, set_selected_widget};
use crate::ui::grid::Grid;
use crate::ui::views::ViewTrait;

/// View subtype representing the different configuration views available when
/// the user is in the Configuration mode.
#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigurationView {}

impl ViewTrait for ConfigurationView {
    fn show_main_view(&mut self, ui: &mut Ui, appctx: &mut AppContext) {
        let app_style = ui.app_style();
        let panel_frame = Frame::new().fill(app_style.main_panels_fill);

        Panel::left("configuration_widget_gallery")
            .default_size(200.)
            .min_size(180.)
            .max_size(300.)
            .frame(panel_frame)
            .show_inside(ui, |ui| {
                show_panel(ui, "WIDGET GALLERY", "Drag to add to the layout", gallery::show);
            });

        Panel::right("configuration_widget_settings")
            .default_size(200.)
            .min_size(180.)
            .max_size(300.)
            .frame(panel_frame)
            .show_inside(ui, |ui| {
                show_panel(ui, "WIDGET SETTINGS", "Edit the selected widget", settings::show);
            });

        CentralPanel::default()
            .frame(Frame::new().fill(app_style.main_panels_fill))
            .show_inside(ui, |ui| show_widget_grid(ui, appctx));
    }
}

fn show_widget_grid(ui: &mut Ui, appctx: &mut AppContext) {
    let rect = ui.available_rect_before_wrap();

    let widgets = &mut appctx.layout.widgets;
    let data_store = &mut appctx.data_store;
    let grid = Grid::new(rect, appctx.layout.grid_settings);

    let WidgetGridResponse { active, selected_rect } =
        WidgetGrid::new(widgets, &grid).edit_mode(true).show(ui, data_store);

    WidgetEditor::show_selection(ui, selected_rect);

    let WidgetEditorResponse { remove_requested } = if let Some((widget, response)) = active {
        WidgetEditor::new(&grid, widget, response).show(ui)
    } else {
        WidgetEditorResponse { remove_requested: None }
    };

    // Applied after `active`'s borrow of `appctx.layout.widgets` ends.
    if let Some(id) = remove_requested {
        appctx.layout.remove_widget(id);
        set_selected_widget(ui, None);
    }
}

fn show_panel(ui: &mut Ui, title: &str, subtitle: &str, content: impl FnOnce(&mut Ui)) {
    ui.add(PanelHeader::new(title).subtitle(subtitle));

    ScrollArea::vertical()
        .auto_shrink(false)
        .content_margin(ui.spacing().window_margin)
        .show(ui, |ui| {
            ui.vertical(content);
        });
}
