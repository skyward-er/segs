use std::{collections::HashSet, ops::Deref};

use egui::Id;
use serde::{Deserialize, Serialize};

/// A simple id generator that wraps around when it reaches the maximum value.
#[derive(Debug, Clone, Default)]
pub struct IdGenerator {
    current: u32,
    already_used: HashSet<u32>,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the next id, wrapping around if necessary.
    pub fn next_id(&mut self) -> u32 {
        loop {
            self.current = self.current.wrapping_add(1);
            if !self.already_used.contains(&self.current) {
                self.already_used.insert(self.current);
                return self.current;
            }
        }
    }

    pub fn sync_used_ids(&mut self, used_ids: &[u32]) {
        self.already_used.clear();
        for id in used_ids {
            self.already_used.insert(*id);
        }
    }

    /// Release an id, making it available for reuse. Do not change the current
    /// id to avoid useless checks and exploit wrapping increment.
    pub fn release_id(&mut self, id: u32) {
        self.already_used.remove(&id);
    }
}

/// `PaneID` is a wrapper around `egui::Id` used to uniquely identify panes within the application.
///
/// This type is intended for use as an identifier for panes, allowing for persistent memory usage
/// in the egui context. By encapsulating `egui::Id`, `PaneID` provides type safety and convenient
/// operations for constructing hierarchical or derived IDs (e.g., via the `/` operator).
///
/// # Usage
/// Use `PaneID` whenever you need to store or retrieve state associated with a specific pane in
/// the egui context, ensuring that each pane has a unique and consistent identifier across frames.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PaneId(Id);

impl PaneId {
    /// Creates a new `PaneID` from an `egui::Id`.
    pub fn new(hash: impl std::hash::Hash) -> Self {
        Self(Id::new(hash))
    }

    /// Creates a new `PaneID` from a string slice.
    pub fn from_str(s: &str) -> Self {
        Self(Id::new(s))
    }

    /// Returns the inner `egui::Id`.
    pub fn id(&self) -> Id {
        self.0
    }

    pub fn next_id(&mut self) -> Self {
        Self(Id::new(self.0.value().wrapping_add(1)))
    }
}

impl<H: std::hash::Hash> std::ops::Div<H> for PaneId {
    type Output = Self;

    fn div(self, rhs: H) -> Self::Output {
        PaneId(self.0.with(rhs))
    }
}

impl Deref for PaneId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PaneId> for Id {
    fn from(pane_id: PaneId) -> Self {
        pane_id.0
    }
}
