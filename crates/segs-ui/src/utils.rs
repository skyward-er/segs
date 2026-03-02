use egui::{Context, Response};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

pub fn pointer_clicked_outside(ctx: &Context, response: &Response) -> bool {
    // If the pointer clicked this frame, but NOT on our area
    if ctx.input(|i| i.pointer.any_click()) && !response.clicked_by(egui::PointerButton::Primary) {
        // Additionally check if the pointer is actually outside the rect
        if let Some(pos) = ctx.pointer_interact_pos()
            && !response.rect.contains(pos)
        {
            return true;
        }
    }
    false
}

/// A group of mutually exclusive boolean flags, where exactly one flag is
/// always selected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadioGroup {
    flags: SmallVec<[bool; 8]>,
}

impl RadioGroup {
    /// Creates a new `RadioGroup` with `num_flags` flags, where the flag at
    /// `selected` is set to `true`.
    ///
    /// # Panics
    /// Panics if `num_flags` is 0 or `selected >= num_flags`.
    pub fn new(selected: usize, num_flags: usize) -> Self {
        debug_assert!(num_flags > 0, "num_flags must be greater than 0");
        debug_assert!(selected < num_flags, "selected must be less than num_flags");
        let mut flags = SmallVec::from_elem(false, num_flags);
        flags[selected] = true;
        Self { flags }
    }

    /// Applies a callback `f` to the flag at `index`, then re-enforces the
    /// radio group invariant:
    /// - If the flag was turned **on**, all other flags are turned off.
    /// - If the flag was turned **off**, it is restored to `true` to prevent an
    ///   empty selection.
    pub fn with_flag<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut bool),
    {
        f(&mut self.flags[index]);
        if self.flags[index] {
            // Turn off all other flags
            for (i, flag) in self.flags.iter_mut().enumerate() {
                if i != index {
                    *flag = false;
                }
            }
        } else {
            // Restore the flag to prevent an empty selection
            self.flags[index] = true;
        }
    }

    /// Returns the index of the currently selected flag.
    ///
    /// # Panics
    /// Panics if no flag is selected (should never happen if the invariant is
    /// upheld).
    pub fn selected(&self) -> usize {
        self.flags.iter().position(|&b| b).unwrap()
    }
}
