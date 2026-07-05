use crate::ui::{grid::GridSettings, widgets::WidgetData};

pub struct Layout {
    pub widgets: Vec<WidgetData>,
    pub grid_settings: GridSettings,
}

impl Layout {
    pub fn new() -> Self {
        // Example test layout
        let widgets = vec![
            WidgetData {
                id: egui::Id::new("example_widget1"),
                grect: crate::ui::grid::GRect::new(egui::Rect::from_min_size(egui::pos2(1., 0.), egui::vec2(1., 1.))),
                variant: crate::ui::widgets::WidgetVariant::ValueDisplay(crate::ui::widgets::ValueDisplayWidget {
                    value: "15.024".to_string(),
                }),
            },
            WidgetData {
                id: egui::Id::new("example_widget2"),
                grect: crate::ui::grid::GRect::new(egui::Rect::from_min_size(egui::pos2(1., 5.), egui::vec2(2., 2.))),
                variant: crate::ui::widgets::WidgetVariant::ValueDisplay(crate::ui::widgets::ValueDisplayWidget {
                    value: "25.024".to_string(),
                }),
            },
            WidgetData {
                id: egui::Id::new("example_widget3"),
                grect: crate::ui::grid::GRect::new(egui::Rect::from_min_size(egui::pos2(6., 2.), egui::vec2(2., 6.))),
                variant: crate::ui::widgets::WidgetVariant::ValueDisplay(crate::ui::widgets::ValueDisplayWidget {
                    value: "189.024".to_string(),
                }),
            },
        ];

        let grid_settings = GridSettings::fixed(8, 8);

        Self { widgets, grid_settings }
    }

    /// Removes the widget with the given id from the layout, if present.
    pub fn remove_widget(&mut self, id: egui::Id) {
        self.widgets.retain(|widget| widget.id != id);
    }
}
