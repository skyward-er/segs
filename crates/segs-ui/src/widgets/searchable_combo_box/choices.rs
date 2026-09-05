use std::hash::Hash;

use egui::ahash::{HashMap, HashMapExt};

/// An immutable ordered list of searchable combo-box choices.
pub struct SearchableComboBoxList<T> {
    items: Vec<Choice<T>>,
    value_indices: HashMap<T, usize>,
}

impl<T> SearchableComboBoxList<T> {
    /// Builds a reusable flat list from `(value, label)` pairs.
    ///
    /// Returns a list with normalized labels.
    pub fn new<I, L>(items: I) -> Self
    where
        I: IntoIterator<Item = (T, L)>,
        L: Into<String>,
        T: Clone + Eq + Hash,
    {
        let items = items.into_iter();
        let (minimum, _) = items.size_hint();
        let mut choices = Vec::with_capacity(minimum);
        let mut value_indices = HashMap::with_capacity(minimum);

        // Normalize choices and index their first occurrence in one pass
        for (value, label) in items {
            let index = choices.len();
            value_indices.entry(value.clone()).or_insert(index);
            choices.push(Choice::new(value, label.into(), None));
        }
        Self {
            items: choices,
            value_indices,
        }
    }

    /// Returns the number of selectable choices in the list.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the list contains no choices.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Iterates over values and labels in display order.
    ///
    /// Returns an iterator whose entries borrow this list.
    pub fn iter(&self) -> impl Iterator<Item = (&T, &str)> {
        self.items.iter().map(|item| (&item.value, item.label.as_str()))
    }

    /// Finds the display label for a value.
    ///
    /// Returns `None` when the value is not available in this list.
    pub fn label_for(&self, value: &T) -> Option<&str>
    where
        T: Eq + Hash,
    {
        self.value_indices
            .get(value)
            .map(|index| self.items[*index].label.as_str())
    }
}

impl<T> Clone for SearchableComboBoxList<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            value_indices: self.value_indices.clone(),
        }
    }
}

/// An immutable searchable hierarchy stored in depth-first display order.
pub struct SearchableComboBoxHierarchy<T> {
    nodes: Vec<HierarchyNode<T>>,
    value_indices: HashMap<T, usize>,
}

impl<T> SearchableComboBoxHierarchy<T> {
    /// Builds a reusable hierarchy through a nested group-and-item builder.
    ///
    /// Returns the flattened hierarchy produced by `add_contents`.
    pub fn build(add_contents: impl FnOnce(&mut SearchableComboBoxHierarchyBuilder<'_, T>)) -> Self
    where
        T: Clone + Eq + Hash,
    {
        let mut nodes = Vec::new();
        let mut path = Vec::new();
        let mut value_indices = HashMap::new();
        add_contents(&mut SearchableComboBoxHierarchyBuilder {
            nodes: &mut nodes,
            value_indices: &mut value_indices,
            path: &mut path,
            depth: 0,
            parent: None,
        });
        Self { nodes, value_indices }
    }

    /// Returns the number of flattened group and item rows.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns whether the hierarchy contains no rows.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Iterates over selectable values and labels in hierarchy traversal order.
    ///
    /// Returns an iterator that excludes structure rows.
    pub fn items(&self) -> impl Iterator<Item = (&T, &str)> {
        self.nodes.iter().filter_map(|node| {
            let HierarchyNodeKind::Item(choice) = &node.kind else {
                return None;
            };
            Some((&choice.value, choice.label.as_str()))
        })
    }

    /// Finds the leaf label for a selectable value.
    ///
    /// Returns `None` when the value is not available in this hierarchy.
    pub fn label_for(&self, value: &T) -> Option<&str>
    where
        T: Eq + Hash,
    {
        self.value_indices
            .get(value)
            .map(|index| self.nodes[*index].label.as_str())
    }
}

/// Builds a [`SearchableComboBoxHierarchy`] without retaining a nested copy.
pub struct SearchableComboBoxHierarchyBuilder<'a, T> {
    nodes: &'a mut Vec<HierarchyNode<T>>,
    value_indices: &'a mut HashMap<T, usize>,
    path: &'a mut Vec<String>,
    depth: usize,
    parent: Option<usize>,
}

impl<T> SearchableComboBoxHierarchyBuilder<'_, T>
where
    T: Clone + Eq + Hash,
{
    /// Adds a selectable value at the builder's current hierarchy depth.
    pub fn item(&mut self, value: T, label: impl Into<String>) {
        let label = label.into();
        let item_index = self.nodes.len();
        let mut breadcrumb = self.path.join(" › ");
        if !breadcrumb.is_empty() {
            breadcrumb.push_str(" › ");
        }
        breadcrumb.push_str(&label);
        self.value_indices.entry(value.clone()).or_insert(item_index);
        self.nodes.push(HierarchyNode {
            label: label.clone(),
            normalized_label: normalize_query(&label),
            depth: self.depth,
            parent: self.parent,
            kind: HierarchyNodeKind::Item(Choice::new(value, label, Some(breadcrumb))),
        });
    }

    /// Adds a group and all nested contents at the next hierarchy depth.
    pub fn group(&mut self, label: impl Into<String>, add_contents: impl FnOnce(&mut Self)) {
        let label = label.into();
        let group_index = self.nodes.len();
        self.nodes.push(HierarchyNode {
            normalized_label: normalize_query(&label),
            label: label.clone(),
            depth: self.depth,
            parent: self.parent,
            kind: HierarchyNodeKind::Group {
                subtree_end: group_index + 1,
            },
        });

        // Append descendants directly into the final depth-first storage
        self.path.push(label);
        let parent = self.parent.replace(group_index);
        self.depth += 1;
        add_contents(self);
        self.depth -= 1;
        self.parent = parent;
        self.path.pop();
        self.nodes[group_index].kind = HierarchyNodeKind::Group {
            subtree_end: self.nodes.len(),
        };
    }
}

#[derive(Clone)]
struct Choice<T> {
    value: T,
    label: String,
    normalized_label: String,
    breadcrumb: Option<String>,
}

impl<T> Choice<T> {
    fn new(value: T, label: String, breadcrumb: Option<String>) -> Self {
        Self {
            normalized_label: normalize_query(&label),
            value,
            label,
            breadcrumb,
        }
    }
}

struct HierarchyNode<T> {
    label: String,
    normalized_label: String,
    depth: usize,
    parent: Option<usize>,
    kind: HierarchyNodeKind<T>,
}

enum HierarchyNodeKind<T> {
    Group { subtree_end: usize },
    Item(Choice<T>),
}

pub(super) trait ChoiceSource {
    type Value;

    fn len(&self) -> usize;
    fn label(&self, index: usize) -> &str;
    fn normalized_label(&self, index: usize) -> &str;
    fn depth(&self, index: usize) -> usize;
    fn group_end(&self, index: usize) -> Option<usize>;
    fn parent(&self, index: usize) -> Option<usize>;
    fn value(&self, index: usize) -> Option<&Self::Value>;
    fn value_index(&self, value: &Self::Value) -> Option<usize>;
    fn selected_text(&self, value: &Self::Value) -> Option<&str>
    where
        Self::Value: Eq + Hash;
}

impl<T> ChoiceSource for SearchableComboBoxList<T>
where
    T: Eq + Hash,
{
    type Value = T;

    fn len(&self) -> usize {
        self.items.len()
    }

    fn label(&self, index: usize) -> &str {
        &self.items[index].label
    }

    fn normalized_label(&self, index: usize) -> &str {
        &self.items[index].normalized_label
    }

    fn depth(&self, _index: usize) -> usize {
        0
    }

    fn group_end(&self, _index: usize) -> Option<usize> {
        None
    }

    fn parent(&self, _index: usize) -> Option<usize> {
        None
    }

    fn value(&self, index: usize) -> Option<&Self::Value> {
        Some(&self.items[index].value)
    }

    fn value_index(&self, value: &Self::Value) -> Option<usize> {
        self.value_indices.get(value).copied()
    }

    fn selected_text(&self, value: &Self::Value) -> Option<&str>
    where
        Self::Value: Eq + Hash,
    {
        self.label_for(value)
    }
}

impl<T> ChoiceSource for SearchableComboBoxHierarchy<T>
where
    T: Eq + Hash,
{
    type Value = T;

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn label(&self, index: usize) -> &str {
        &self.nodes[index].label
    }

    fn normalized_label(&self, index: usize) -> &str {
        &self.nodes[index].normalized_label
    }

    fn depth(&self, index: usize) -> usize {
        self.nodes[index].depth
    }

    fn group_end(&self, index: usize) -> Option<usize> {
        match self.nodes[index].kind {
            HierarchyNodeKind::Group { subtree_end } => Some(subtree_end),
            HierarchyNodeKind::Item(_) => None,
        }
    }

    fn parent(&self, index: usize) -> Option<usize> {
        self.nodes[index].parent
    }

    fn value(&self, index: usize) -> Option<&Self::Value> {
        match &self.nodes[index].kind {
            HierarchyNodeKind::Group { .. } => None,
            HierarchyNodeKind::Item(choice) => Some(&choice.value),
        }
    }

    fn value_index(&self, value: &Self::Value) -> Option<usize> {
        self.value_indices.get(value).copied()
    }

    fn selected_text(&self, value: &Self::Value) -> Option<&str>
    where
        Self::Value: Eq + Hash,
    {
        let node = &self.nodes[self.value_index(value)?];
        let HierarchyNodeKind::Item(choice) = &node.kind else {
            return None;
        };
        Some(choice.breadcrumb.as_deref().unwrap_or(&choice.label))
    }
}

pub(super) fn normalize_query(query: &str) -> String {
    query.trim().to_lowercase()
}
