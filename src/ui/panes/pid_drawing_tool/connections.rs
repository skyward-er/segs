use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Connection {
    /// Index of the start element
    pub start: usize,

    /// Index of the end element
    pub end: usize,
}
