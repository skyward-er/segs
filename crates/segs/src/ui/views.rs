pub mod configuration;
pub mod operator;

use egui::Ui;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

pub const VIEW_MODE_ID: &str = "view_mode";

/// View represents what the user is currently looking at, imagine this as the
/// index of a document, but instead of pages, we index over possible layouts of
/// the UI. This is useful to keep track of which panels should be visible, and
/// which should not, as well as to keep track of the state of each view.
#[enum_dispatch(ViewTrait)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum View {
    Configuration(configuration::ConfigurationView),
    Operator(operator::OperatorView),
}

#[enum_dispatch]
trait ViewTrait {
    fn show_main_view(&mut self, ui: &mut Ui, appctx: &mut AppContext);
}

impl View {
    pub fn show(&mut self, ui: &mut Ui, appctx: &mut AppContext) {
        self.show_main_view(ui, appctx);
    }
}

impl Default for View {
    fn default() -> Self {
        Self::Configuration(configuration::ConfigurationView::default())
    }
}
