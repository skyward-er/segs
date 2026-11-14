//! Undo and redo history of the P&ID editor.

use std::mem;

use super::PidWidget;

/// Oldest steps are dropped beyond this many, to bound memory.
const MAX_STEPS: usize = 100;

/// Snapshots of the draft taken whenever an edit settles.
///
/// The draft is never stored here while it is changing (during a drag or while typing);
/// it is compared against `committed` once the edit is finished and recorded as one step.
#[derive(Clone)]
pub(super) struct History {
    /// Earlier states, the most recent last.
    undo: Vec<PidWidget>,
    /// States undone and available to redo, the next one last.
    redo: Vec<PidWidget>,
    /// Last recorded state of the draft.
    committed: PidWidget,
}

impl History {
    /// Starts a history whose first state is `diagram`.
    pub(super) fn new(diagram: &PidWidget) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            committed: diagram.clone(),
        }
    }

    /// Records `draft` as a new step if it differs from the last recorded state.
    /// A new step discards everything that could have been redone.
    pub(super) fn record(&mut self, draft: &PidWidget) {
        if *draft == self.committed {
            return;
        }
        self.undo.push(mem::replace(&mut self.committed, draft.clone()));
        if self.undo.len() > MAX_STEPS {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Restores the state before the last step into `draft`.
    ///
    /// Returns `false`, leaving `draft` unchanged, when there is nothing to undo.
    pub(super) fn undo(&mut self, draft: &mut PidWidget) -> bool {
        self.record(draft);
        let Some(previous) = self.undo.pop() else {
            return false;
        };
        self.redo.push(mem::replace(&mut self.committed, previous.clone()));
        *draft = previous;
        true
    }

    /// Reapplies the last undone step into `draft`.
    ///
    /// Returns `false`, leaving `draft` unchanged, when there is nothing to redo
    /// or `draft` was edited since the undo.
    pub(super) fn redo(&mut self, draft: &mut PidWidget) -> bool {
        self.record(draft);
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push(mem::replace(&mut self.committed, next.clone()));
        *draft = next;
        true
    }

    /// Returns whether [`Self::undo`] would change `draft`.
    pub(super) fn can_undo(&self, draft: &PidWidget) -> bool {
        !self.undo.is_empty() || *draft != self.committed
    }

    /// Returns whether [`Self::redo`] would change `draft`.
    pub(super) fn can_redo(&self, draft: &PidWidget) -> bool {
        !self.redo.is_empty() && *draft == self.committed
    }
}
