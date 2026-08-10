pub mod configuration;
pub mod operator;
pub mod welcome;

use egui::Ui;
use enum_dispatch::enum_dispatch;

use crate::app::AppContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewTarget {
    Welcome,
    Operator,
    Configuration,
}

#[enum_dispatch(ViewTrait)]
pub enum View {
    Welcome(welcome::WelcomeView),
    Configuration(configuration::ConfigurationView),
    Operator(operator::OperatorView),
}

#[enum_dispatch]
pub trait ViewTrait {
    fn show_main_view(&mut self, ui: &mut Ui, appctx: &mut AppContext);
}

impl View {
    /// Shows the active application view.
    pub fn show(&mut self, ui: &mut Ui, appctx: &mut AppContext) {
        self.show_main_view(ui, appctx);
    }

    /// Creates the view associated with a requested transition target.
    pub fn from_target(target: ViewTarget) -> Self {
        match target {
            ViewTarget::Welcome => Self::Welcome(welcome::WelcomeView),
            ViewTarget::Operator => Self::Operator(operator::OperatorView),
            ViewTarget::Configuration => Self::Configuration(configuration::ConfigurationView::default()),
        }
    }
}
