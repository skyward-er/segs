use std::collections::{HashMap, HashSet};

use crate::dataflow::{DataKey, protocol::FieldDescriptor};

/// Identifies whether an indexed row groups descendants or selects a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexedNodeKind {
    Structure {
        /// The first node after this structure's descendants.
        subtree_end: usize,
    },
    Field {
        /// The field data key
        data_key: DataKey,
    },
}

/// Stores one fixed-height row in depth-first descriptor traversal order.
///
/// The UI renders these nodes as virtualized rows rather than recursively
/// rendering the original [`FieldDescriptor`] hierarchy.
#[derive(Debug)]
pub struct IndexedNode {
    /// The original structure or field name shown in the tree.
    pub name: String,
    /// The hierarchy depth used to calculate visual indentation.
    pub depth: usize,
    /// The row behavior and its structure or field payload.
    pub kind: IndexedNodeKind,
    /// The lowercase name used for case-insensitive substring matching.
    normalized_name: String,
    /// The parent node used to restore hierarchy context around matches.
    parent: Option<usize>,
}

/// Owns a protocol hierarchy flattened for searching and virtualized rendering.
///
/// A hierarchy such as:
///
/// ```text
/// Flight
///   Timing
///     Timestamp
///     Sequence
///   Roll
/// GPS
///   Latitude
/// ```
///
/// is stored in depth-first order as:
///
/// ```text
/// 0  Flight       Structure { subtree_end: 5 }
/// 1    Timing     Structure { subtree_end: 4 }
/// 2      Timestamp
/// 3      Sequence
/// 4    Roll
/// 5  GPS          Structure { subtree_end: 7 }
/// 6    Latitude
/// ```
///
/// Structure descendants are contiguous, so their exclusive `subtree_end`
/// index lets browsing skip a collapsed subtree without inspecting each child.
/// Search results and visible rows are represented as indices into `nodes`,
/// preserving hierarchy order without copying complete nodes.
///
/// Field paths are indexed separately by [`DataKey`] so the selected-field
/// preview can retrieve a breadcrumb such as `Flight › Timing › Timestamp`
/// without traversing the hierarchy again.
#[derive(Debug)]
pub struct DescriptorIndex {
    /// The flattened protocol hierarchy in depth-first traversal order.
    nodes: Vec<IndexedNode>,
    /// Complete display breadcrumbs indexed by field key.
    field_paths: HashMap<DataKey, String>,
}

impl DescriptorIndex {
    /// Builds the index from a protocol descriptor hierarchy.
    pub fn build(descriptors: &[FieldDescriptor]) -> Self {
        let mut index = Self {
            nodes: Vec::new(),
            field_paths: HashMap::new(),
        };
        let mut path = Vec::new();
        index.push_descriptors(descriptors, 0, None, &mut path);
        index
    }

    /// Returns all indexed nodes in hierarchy traversal order.
    pub fn nodes(&self) -> &[IndexedNode] {
        &self.nodes
    }

    /// Returns the breadcrumb associated with a selected data key.
    pub fn field_path(&self, data_key: DataKey) -> Option<&str> {
        self.field_paths.get(&data_key).map(String::as_str)
    }

    /// Returns browsable rows according to the expanded structure nodes.
    ///
    /// Every structure is included in the result. Children of expanded
    /// structures are visited normally, while a collapsed structure advances
    /// directly to its exclusive `subtree_end` index.
    pub fn visible_rows(&self, expanded: &HashSet<usize>) -> Vec<usize> {
        let mut rows = Vec::new();
        let mut index = 0;
        while index < self.nodes.len() {
            rows.push(index);
            match self.nodes[index].kind {
                IndexedNodeKind::Structure { subtree_end } if !expanded.contains(&index) => {
                    index = subtree_end;
                }
                _ => index += 1,
            }
        }
        rows
    }

    /// Filters matching nodes while retaining their hierarchy context.
    ///
    /// The query is trimmed and lowercased before substring matching. A matching
    /// field includes each ancestor required to show its complete path. A
    /// matching structure includes both its ancestors and its entire subtree.
    /// Marked nodes are returned in their original depth-first order.
    ///
    /// For example, matching `timestamp` can produce:
    ///
    /// ```text
    /// Flight
    ///   Timing
    ///     Timestamp
    /// GPS
    ///   Timestamp
    /// ```
    ///
    /// Matching `flight` instead includes `Flight` and all of its descendants.
    pub fn filtered_rows(&self, query: &str) -> Vec<usize> {
        let query = normalize_query(query);
        if query.is_empty() {
            return Vec::new();
        }

        let mut included = vec![false; self.nodes.len()];
        for (index, node) in self.nodes.iter().enumerate() {
            if !node.normalized_name.contains(&query) {
                continue;
            }

            let mut ancestor = Some(index);
            while let Some(index) = ancestor {
                included[index] = true;
                ancestor = self.nodes[index].parent;
            }

            if let IndexedNodeKind::Structure { subtree_end } = node.kind {
                included[index..subtree_end].fill(true);
            }
        }

        included
            .into_iter()
            .enumerate()
            .filter_map(|(index, included)| included.then_some(index))
            .collect()
    }

    /// Appends one hierarchy level to the flattened descriptor index.
    ///
    /// Descriptors are emitted in depth-first order. A structure row is inserted
    /// before its children with an initially empty range ending at `index + 1`.
    /// Its children are appended recursively, then `subtree_end` is updated to
    /// the first index after those descendants. This produces the contiguous
    /// half-open range used to skip collapsed subtrees and include complete
    /// structures in search results.
    ///
    /// `path` is a stack containing the names of the active ancestor structures.
    /// Structure recursion pushes and later removes one name, while fields join
    /// the stack with their own name to build the selected-field breadcrumb.
    ///
    /// - `descriptors` contains the siblings appended during this invocation.
    /// - `depth` becomes each sibling's indentation level.
    /// - `parent` identifies their containing structure, if one exists.
    /// - `path` contains their ancestor names and is restored before returning.
    fn push_descriptors(
        &mut self,
        descriptors: &[FieldDescriptor],
        depth: usize,
        parent: Option<usize>,
        path: &mut Vec<String>,
    ) {
        for descriptor in descriptors {
            match descriptor {
                FieldDescriptor::Structure { name, fields } => {
                    // Insert the structure before its contiguous descendants
                    let index = self.nodes.len();
                    self.nodes.push(IndexedNode {
                        name: name.clone(),
                        normalized_name: name.to_lowercase(),
                        depth,
                        parent,
                        kind: IndexedNodeKind::Structure { subtree_end: index + 1 },
                    });

                    // Extend the ancestor context while indexing child fields
                    path.push(name.clone());
                    self.push_descriptors(fields, depth + 1, Some(index), path);
                    path.pop();

                    // Close the half-open subtree range after all descendants
                    self.nodes[index].kind = IndexedNodeKind::Structure {
                        subtree_end: self.nodes.len(),
                    };
                }
                FieldDescriptor::Field { name, data_key, .. } => {
                    // Build the display path from active ancestors and this field
                    let mut breadcrumb = path.join(" › ");
                    if !breadcrumb.is_empty() {
                        breadcrumb.push_str(" › ");
                    }
                    breadcrumb.push_str(name);

                    self.field_paths.insert(*data_key, breadcrumb);
                    self.nodes.push(IndexedNode {
                        name: name.clone(),
                        normalized_name: name.to_lowercase(),
                        depth,
                        parent,
                        kind: IndexedNodeKind::Field { data_key: *data_key },
                    });
                }
            }
        }
    }
}

/// Normalizes a user query for matching and cache comparison.
fn normalize_query(query: &str) -> String {
    query.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataflow::{DataType, StreamKey};

    /// Creates a field descriptor for hierarchy filtering tests.
    fn field(name: &str) -> FieldDescriptor {
        FieldDescriptor::Field {
            name: name.into(),
            field_type: DataType::F64,
            data_key: StreamKey::mock().data_key,
        }
    }

    /// Creates a nested descriptor hierarchy for filtering tests.
    fn descriptors() -> Vec<FieldDescriptor> {
        vec![
            FieldDescriptor::Structure {
                name: "Flight".into(),
                fields: vec![
                    FieldDescriptor::Structure {
                        name: "Timing".into(),
                        fields: vec![field("Timestamp"), field("Sequence")],
                    },
                    field("Roll"),
                ],
            },
            FieldDescriptor::Structure {
                name: "GPS".into(),
                fields: vec![field("Timestamp"), field("Latitude")],
            },
        ]
    }

    #[test]
    fn filtering_retains_ancestors_and_expands_structure_matches() {
        let index = DescriptorIndex::build(&descriptors());

        let field_rows = index.filtered_rows(" TIMESTAMP ");
        let field_names = field_rows
            .iter()
            .map(|row| index.nodes()[*row].name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(field_names, ["Flight", "Timing", "Timestamp", "GPS", "Timestamp"]);

        let structure_rows = index.filtered_rows("flight");
        let structure_names = structure_rows
            .iter()
            .map(|row| index.nodes()[*row].name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(structure_names, ["Flight", "Timing", "Timestamp", "Sequence", "Roll"]);
    }
}
