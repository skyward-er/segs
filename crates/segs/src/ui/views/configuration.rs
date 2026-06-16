mod gallery;
mod settings;

use egui::{Id, ScrollArea, Ui, UiBuilder, Vec2};
use segs_ui::components::panel_header::PanelHeader;
use serde::{Deserialize, Serialize};

use segs_assets::icons;
use segs_memory::MemoryExt;

use self::Activity::{WidgetGallery, WidgetSettings};
use crate::app::AppContext;
use crate::ui::components::widget_editor::WidgetEditor;
use crate::ui::components::widget_grid::WidgetGrid;
use crate::ui::grid::Grid;
use crate::ui::views::LEFT_PANEL_VISIBLE_ID;
use crate::ui::{components::left_menu::LeftBarMenuButton, views::ViewTrait};

const LEFT_PANEL_CONFIGURATION_ACTIVITY_ID: &str = "left_panel_configuration_activity";

/// View subtype representing the different configuration views available when
/// the user is in the Configuration mode.
#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigurationView {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Activity {
    #[default]
    WidgetSettings,
    WidgetGallery,
}

impl Activity {
    pub fn hint(&self) -> &'static str {
        match self {
            WidgetSettings => "Edit the selected widget",
            WidgetGallery => "Drag to add to the layout",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            WidgetSettings => "Widget Settings",
            WidgetGallery => "Widget Gallery",
        }
    }

    pub fn show_panel_content(&self, ui: &mut Ui) {
        match self {
            WidgetSettings => settings::show(ui),
            WidgetGallery => gallery::show(ui),
        }
    }
}

impl ViewTrait for ConfigurationView {
    fn show_activities(&mut self, ui: &mut Ui, _appctx: &mut AppContext) {
        let left_panel_id = Id::new(LEFT_PANEL_VISIBLE_ID);
        let last_selected_id = Id::new(LEFT_PANEL_CONFIGURATION_ACTIVITY_ID);

        let mut left_panel_visible = ui.mem().get_perm_or_default(left_panel_id);
        let mut last_selected: Activity = ui.mem().get_perm_or_default(last_selected_id);

        let mut selected: Option<Activity> = if left_panel_visible { Some(last_selected) } else { None };

        ui.spacing_mut().item_spacing = Vec2::ZERO;
        ui.add_space(5.);

        ui.add(LeftBarMenuButton::new(
            &mut selected,
            Activity::WidgetSettings,
            icons::Settings::outline(),
            icons::Settings::solid(),
        ));

        ui.add(LeftBarMenuButton::new(
            &mut selected,
            Activity::WidgetGallery,
            icons::Layout::outline(),
            icons::Layout::solid(),
        ));

        // Store last selected
        if let Some(activity) = selected {
            last_selected = activity;
            left_panel_visible = true;
        } else {
            left_panel_visible = false;
        }

        ui.mem().insert_perm(left_panel_id, left_panel_visible);
        ui.mem().insert_perm(last_selected_id, last_selected);
    }

    fn show_left_panel(&mut self, ui: &mut Ui, _appctx: &mut AppContext) {
        let selected_id = Id::new(LEFT_PANEL_CONFIGURATION_ACTIVITY_ID);
        let selected: Activity = ui.mem().get_perm_or_default(selected_id);

        let title = selected.tooltip().to_uppercase();
        let subtitle = selected.hint();

        ui.scope_builder(UiBuilder::new().id_salt(selected), |ui| {
            ui.add(PanelHeader::new(title).subtitle(subtitle));

            ScrollArea::vertical()
                .auto_shrink(false)
                .content_margin(ui.spacing().window_margin)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        selected.show_panel_content(ui);
                    });
                });
        });
    }

    fn show_main_view(&mut self, ui: &mut Ui, appctx: &mut AppContext) {
        let rect = ui.available_rect_before_wrap();

        let widgets = &mut appctx.layout.widgets;
        let data_store = &mut appctx.data_store;
        let grid = Grid::new(rect, appctx.layout.grid_settings);

        let res = WidgetGrid::new(widgets, &grid).edit_mode(true).show(ui, data_store);

        if let Some((widget, response)) = res {
            WidgetEditor::new(&grid, widget, response).show(ui);
        }
    }
}
