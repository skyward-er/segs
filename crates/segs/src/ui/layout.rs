use egui::{Id, Rect, pos2, vec2};

use crate::ui::{
    grid::{GRect, GridSettings},
    widgets::{ValueDisplayWidget, WidgetData, WidgetVariant},
};

const ADDED_WIDGET_ID_NAMESPACE: &str = "layout_added_widget";

pub struct Layout {
    pub widgets: Vec<WidgetData>,
    pub grid_settings: GridSettings,
    next_widget_id: u64,
}

impl Layout {
    pub fn new() -> Self {
        // Example test layout
        let widgets = vec![
            WidgetData {
                id: Id::new("example_widget1"),
                grect: GRect::new(Rect::from_min_size(pos2(1., 0.), vec2(1., 1.))),
                variant: WidgetVariant::ValueDisplay(ValueDisplayWidget {
                    value: "15.024".to_string(),
                    ..Default::default()
                }),
            },
            WidgetData {
                id: Id::new("example_widget2"),
                grect: GRect::new(Rect::from_min_size(pos2(1., 5.), vec2(2., 2.))),
                variant: WidgetVariant::ValueDisplay(ValueDisplayWidget {
                    value: "25.024".to_string(),
                    ..Default::default()
                }),
            },
            WidgetData {
                id: Id::new("example_widget3"),
                grect: GRect::new(Rect::from_min_size(pos2(6., 2.), vec2(2., 6.))),
                variant: WidgetVariant::ValueDisplay(ValueDisplayWidget {
                    value: "189.024".to_string(),
                    ..Default::default()
                }),
            },
        ];

        let grid_settings = GridSettings::fixed(8, 8);

        Self {
            widgets,
            grid_settings,
            next_widget_id: 0,
        }
    }

    /// Adds a widget and returns its id.
    pub fn add_widget(&mut self, variant: WidgetVariant, grect: GRect) -> Id {
        let id = Id::new((ADDED_WIDGET_ID_NAMESPACE, self.next_widget_id));
        self.next_widget_id += 1;
        self.widgets.push(WidgetData { id, grect, variant });
        id
    }

    /// Removes the widget with the given id from the layout, if present.
    pub fn remove_widget(&mut self, id: Id) {
        self.widgets.retain(|widget| widget.id != id);
    }
}
