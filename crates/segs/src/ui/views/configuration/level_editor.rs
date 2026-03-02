use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelEditorView;

impl super::ConfigurationViewTrait for LevelEditorView {}
