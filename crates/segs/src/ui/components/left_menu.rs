use egui::{Response, Ui, Widget, vec2};
use segs_assets::icons::Icon;
use segs_ui::widgets::buttons::RibbonToggle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftMenuSelector {
    PaneControls,
    LayoutComposer,
    LevelEditor,
    DataflowEditor,
    OnlineResources,
    Charts,
}

impl LeftMenuSelector {
    fn tooltip(&self) -> &'static str {
        match self {
            LeftMenuSelector::PaneControls => "Pane controls",
            LeftMenuSelector::LayoutComposer => "Layout composer",
            LeftMenuSelector::LevelEditor => "Level editor",
            LeftMenuSelector::DataflowEditor => "Dataflow editor",
            LeftMenuSelector::OnlineResources => "Online resources",
            LeftMenuSelector::Charts => "Charts",
        }
    }
}

pub struct LeftBarMenuButton<'a> {
    selector: &'a mut Option<LeftMenuSelector>,
    selected_variant: LeftMenuSelector,
    inactive_icon: Box<dyn Icon>,
    active_icon: Box<dyn Icon>,
}

impl<'a> LeftBarMenuButton<'a> {
    pub fn new(
        selector: &'a mut Option<LeftMenuSelector>,
        selected_variant: LeftMenuSelector,
        inactive_icon: impl Icon + 'static,
        active_icon: impl Icon + 'static,
    ) -> Self {
        Self {
            selector,
            selected_variant,
            inactive_icon: Box::new(inactive_icon),
            active_icon: Box::new(active_icon),
        }
    }
}

impl<'a> Widget for LeftBarMenuButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            selector,
            selected_variant,
            inactive_icon,
            active_icon,
        } = self;

        let mut toggled = selector.as_ref().is_some_and(|v| *v == selected_variant);
        let widget = RibbonToggle::new(inactive_icon, active_icon, &mut toggled)
            .icon_size(vec2(20., 20.))
            .shadow_size(vec2(34., 26.))
            .tooltip(selected_variant.tooltip());
        let response = ui.add(widget);

        // Toggle logic: if the button is clicked, set the selector to the selected
        // variant if it was previously not selected, or to None if it was already
        // selected.
        if toggled && response.clicked() {
            *selector = Some(selected_variant);
        } else if !toggled && response.clicked() {
            *selector = None;
        }
        response
    }
}
