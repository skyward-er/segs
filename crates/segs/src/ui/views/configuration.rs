use egui::{Id, Ui, Vec2};
use serde::{Deserialize, Serialize};

use segs_assets::icons;
use segs_memory::MemoryExt;

use self::Activity::{WidgetGallery, WidgetSettings};
use crate::app::AppContext;
use crate::ui::components::widget_grid::WidgetGrid;
use crate::ui::views::LEFT_PANEL_VISIBLE_ID;
use crate::ui::{components::left_menu::LeftBarMenuButton, views::ViewTrait};

const LEFT_PANEL_CONFIGURATION_ACTIVITY_ID: &str = "left_panel_configuration_activity";

/// View subtype representing the different configuration views available when
/// the user is in the Configuration mode.
#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigurationView {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    #[default]
    WidgetSettings,
    WidgetGallery,
}

impl Activity {
    pub fn tooltip(&self) -> &'static str {
        match self {
            WidgetSettings => "Widget Settings",
            WidgetGallery => "Widget Gallery",
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
        let selected = ui.mem().get_perm_or_default(selected_id);

        match selected {
            WidgetSettings => ui.vertical(|ui| {
                ui.heading("Widget Settings");
                ui.label("Settings go here");
            }),
            WidgetGallery => ui.vertical(|ui| {
                ui.heading("Widget Gallery");
                ui.label("Gallery goes here");
            }),
        };
    }

    fn show_main_view(&mut self, ui: &mut Ui, appctx: &mut AppContext) {
        let widgets = &mut appctx.layout;
        let data_store = &mut appctx.data_store;

        WidgetGrid::new()
            .show_snap_guide(true)
            .with_widgets(widgets)
            .show(ui, data_store);
    }
}
