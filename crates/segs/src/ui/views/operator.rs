use serde::{Deserialize, Serialize};

/// View subtype representing the different operator views available when the
/// user is in the Operator mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorView {
    selected_layout: String,
}

impl super::ViewTrait for OperatorView {}
