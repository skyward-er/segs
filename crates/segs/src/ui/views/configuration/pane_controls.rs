use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneControlsView;

impl super::ConfigurationViewTrait for PaneControlsView {}
