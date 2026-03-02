use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnlineResourcesView;

impl super::ConfigurationViewTrait for OnlineResourcesView {}
