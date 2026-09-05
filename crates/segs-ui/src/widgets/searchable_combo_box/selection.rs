use std::hash::Hash;

use egui::ahash::HashSet;

/// Adapts an optional value to single-choice combo-box behavior.
pub struct SingleSelection<'a, T> {
    selected: &'a mut Option<T>,
}

impl<'a, T> SingleSelection<'a, T> {
    /// Creates a single-selection strategy bound to `selected`.
    ///
    /// Returns a strategy that replaces the optional value when a row is chosen.
    pub fn new(selected: &'a mut Option<T>) -> Self {
        Self { selected }
    }
}

/// Adapts a value set to multiple-choice combo-box behavior.
pub struct MultipleSelection<'a, T> {
    selected: &'a mut HashSet<T>,
}

impl<'a, T> MultipleSelection<'a, T> {
    /// Creates a multiple-selection strategy bound to `selected`.
    ///
    /// Returns a strategy that toggles values without closing the popup.
    pub fn new(selected: &'a mut HashSet<T>) -> Self {
        Self { selected }
    }
}

pub(super) trait SelectionState<T> {
    const MULTIPLE: bool;

    fn selected_count(&self) -> usize;
    fn single_value(&self) -> Option<&T>;
    fn is_selected(&self, value: &T) -> bool;
    fn set_selected(&mut self, value: &T, selected: bool) -> bool
    where
        T: Clone;
    fn retain_available(&mut self, available: &mut dyn FnMut(&T) -> bool) -> bool;
}

impl<T> SelectionState<T> for SingleSelection<'_, T>
where
    T: PartialEq,
{
    const MULTIPLE: bool = false;

    fn selected_count(&self) -> usize {
        usize::from(self.selected.is_some())
    }

    fn single_value(&self) -> Option<&T> {
        self.selected.as_ref()
    }

    fn is_selected(&self, value: &T) -> bool {
        self.selected.as_ref() == Some(value)
    }

    fn set_selected(&mut self, value: &T, _selected: bool) -> bool
    where
        T: Clone,
    {
        if self.selected.as_ref() == Some(value) {
            false
        } else {
            *self.selected = Some(value.clone());
            true
        }
    }

    fn retain_available(&mut self, _available: &mut dyn FnMut(&T) -> bool) -> bool {
        false
    }
}

impl<T> SelectionState<T> for MultipleSelection<'_, T>
where
    T: Eq + Hash,
{
    const MULTIPLE: bool = true;

    fn selected_count(&self) -> usize {
        self.selected.len()
    }

    fn single_value(&self) -> Option<&T> {
        None
    }

    fn is_selected(&self, value: &T) -> bool {
        self.selected.contains(value)
    }

    fn set_selected(&mut self, value: &T, selected: bool) -> bool
    where
        T: Clone,
    {
        if selected {
            self.selected.insert(value.clone())
        } else {
            self.selected.remove(value)
        }
    }

    fn retain_available(&mut self, available: &mut dyn FnMut(&T) -> bool) -> bool {
        let before = self.selected.len();
        self.selected.retain(available);
        self.selected.len() != before
    }
}
