mod choices;
mod popup;
mod rows;
mod selection;

use std::hash::Hash;

use egui::{Id, Response, Ui, Widget, WidgetText};

pub use choices::{SearchableComboBoxHierarchy, SearchableComboBoxHierarchyBuilder, SearchableComboBoxList};
pub use selection::{MultipleSelection, SingleSelection};

const INDICATOR_SIZE: f32 = 14.;
const INDICATOR_RIGHT_PADDING: f32 = 6.;
const INDICATOR_TEXT_SPACING: f32 = 4.;
const INDICATOR_ANIMATION_DURATION_FACTOR: f32 = 2.;
const DEFAULT_MAX_VISIBLE_ROWS: usize = 8;
const SEARCH_VERTICAL_MARGIN: i8 = 6;

/// A searchable combo box backed by a reusable choice model and selection strategy.
#[must_use = "Pass this widget to Ui::add"]
pub struct SearchableComboBox<'a, C, S> {
    id: Id,
    choices: &'a C,
    selection: S,
    empty_selection_text: WidgetText,
    max_visible_rows: usize,
    search_hint: WidgetText,
    empty_results_text: String,
    singular_selection_noun: String,
    plural_selection_noun: String,
}

impl<'a, C, S> SearchableComboBox<'a, C, S> {
    /// Creates an identified searchable combo box from immutable choices and mutable selection state.
    ///
    /// `id` must uniquely identify this component and change when its choices are replaced.
    /// Returns a widget configured with generic item text and an eight-row height limit.
    pub fn new(id: Id, choices: &'a C, selection: S) -> Self {
        Self {
            id,
            choices,
            selection,
            empty_selection_text: "Select an item".into(),
            max_visible_rows: DEFAULT_MAX_VISIBLE_ROWS,
            search_hint: "Search…".into(),
            empty_results_text: "No matching items.".to_owned(),
            singular_selection_noun: "item".to_owned(),
            plural_selection_noun: "items".to_owned(),
        }
    }

    /// Sets the prompt displayed when there is no available selected value.
    ///
    /// Returns the combo box with the requested empty-selection prompt.
    pub fn empty_selection_text(mut self, text: impl Into<WidgetText>) -> Self {
        self.empty_selection_text = text.into();
        self
    }

    /// Sets the maximum number of list rows visible without vertical scrolling.
    ///
    /// Returns the combo box with the requested limit, clamped to at least one row.
    pub fn max_visible_rows(mut self, rows: usize) -> Self {
        self.max_visible_rows = rows.max(1);
        self
    }

    /// Sets the placeholder shown by the popup search field.
    ///
    /// Returns the combo box with the requested search placeholder.
    pub fn search_hint(mut self, hint: impl Into<WidgetText>) -> Self {
        self.search_hint = hint.into();
        self
    }

    /// Sets the message shown when search produces no rows.
    ///
    /// Returns the combo box with the requested empty-result message.
    pub fn empty_results_text(mut self, text: impl Into<WidgetText>) -> Self {
        self.empty_results_text = text.into().text().to_owned();
        self
    }

    /// Sets the singular and plural nouns used by multiple-selection summaries.
    ///
    /// Returns the combo box configured to show text such as `1 field selected`.
    pub fn selection_nouns(mut self, singular: impl Into<String>, plural: impl Into<String>) -> Self {
        self.singular_selection_noun = singular.into();
        self.plural_selection_noun = plural.into();
        self
    }
}

impl<T> Widget for SearchableComboBox<'_, SearchableComboBoxList<T>, SingleSelection<'_, T>>
where
    T: Clone + Eq + Hash + 'static,
{
    fn ui(self, ui: &mut Ui) -> Response {
        popup::show_combo_box(ui, self, false)
    }
}

impl<T> Widget for SearchableComboBox<'_, SearchableComboBoxHierarchy<T>, SingleSelection<'_, T>>
where
    T: Clone + Eq + Hash + 'static,
{
    fn ui(self, ui: &mut Ui) -> Response {
        popup::show_combo_box(ui, self, true)
    }
}

impl<T> Widget for SearchableComboBox<'_, SearchableComboBoxList<T>, MultipleSelection<'_, T>>
where
    T: Clone + Eq + Hash + 'static,
{
    fn ui(self, ui: &mut Ui) -> Response {
        popup::show_combo_box(ui, self, false)
    }
}

impl<T> Widget for SearchableComboBox<'_, SearchableComboBoxHierarchy<T>, MultipleSelection<'_, T>>
where
    T: Clone + Eq + Hash + 'static,
{
    fn ui(self, ui: &mut Ui) -> Response {
        popup::show_combo_box(ui, self, true)
    }
}
