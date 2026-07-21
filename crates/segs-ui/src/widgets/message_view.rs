use egui::{Grid, Id, Response, ScrollArea, Ui};

#[derive(Clone, Debug)]
pub struct MessageRow {
    pub key: String,
    pub value: String,
}

impl MessageRow {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MessageViewOptions {
    pub auto_shrink: bool,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
}

impl Default for MessageViewOptions {
    fn default() -> Self {
        Self {
            auto_shrink: true,
            max_width: None,
            max_height: None,
        }
    }
}

pub fn message_view_widget(
    ui: &mut Ui,
    id: impl Into<Id>,
    rows: impl IntoIterator<Item = MessageRow>,
    opts: &MessageViewOptions,
) -> Response {
    let id = id.into();

    let mut scroll_area = ScrollArea::vertical().auto_shrink([opts.auto_shrink, true]);
    if let Some(max_width) = opts.max_width {
        scroll_area = scroll_area.max_width(max_width);
    }
    if let Some(max_height) = opts.max_height {
        scroll_area = scroll_area.max_height(max_height);
    }

    let inner = scroll_area.show(ui, |ui| {
        Grid::new(id).show(ui, |ui| {
            for r in rows {
                ui.label(r.key);
                ui.label(r.value);
                ui.end_row();
            }
        });
        ui.response()
    });

    inner.inner
}
