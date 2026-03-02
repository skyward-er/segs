use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutComposerView;

impl super::ConfigurationViewTrait for LayoutComposerView {}
