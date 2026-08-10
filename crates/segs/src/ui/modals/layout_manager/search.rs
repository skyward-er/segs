use std::{cmp::Ordering, ops::Range};

use aho_corasick::AhoCorasick;
use smallvec::SmallVec;

use crate::layout::Layout;

#[derive(Debug, Clone)]
pub struct SearchResult<'a> {
    pub layout: &'a Layout,
    pub matches: SmallVec<[Range<usize>; 3]>,
}

/// Filters layouts case-insensitively and ranks earlier name matches first.
pub fn search<'a>(layouts: impl IntoIterator<Item = &'a Layout>, query: &str) -> Vec<SearchResult<'a>> {
    if query.is_empty() {
        let mut results = layouts
            .into_iter()
            .map(|layout| SearchResult {
                layout,
                matches: SmallVec::new(),
            })
            .collect::<Vec<_>>();
        results.sort_by(|a, b| a.layout.name.cmp(&b.layout.name));
        return results;
    }

    let Ok(matcher) = AhoCorasick::new([query.to_ascii_lowercase()]) else {
        return Vec::new();
    };
    let mut results = layouts
        .into_iter()
        .filter_map(|layout| {
            let matches = matcher
                .find_iter(&layout.name.to_ascii_lowercase())
                .map(|found| found.range())
                .collect::<SmallVec<_>>();
            (!matches.is_empty()).then_some(SearchResult { layout, matches })
        })
        .collect::<Vec<_>>();
    results.sort_by(|a, b| compare_matches(&a.matches, &b.matches).then_with(|| a.layout.name.cmp(&b.layout.name)));
    results
}

fn compare_matches(a: &[Range<usize>], b: &[Range<usize>]) -> Ordering {
    for (a, b) in a.iter().zip(b) {
        match a.start.cmp(&b.start) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
    }
    a.len().cmp(&b.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earlier_matches_sort_first() {
        let first = Layout::empty("Alpha Flight".into(), "alpha-flight-00000001".into());
        let second = Layout::empty("Flight Alpha".into(), "flight-alpha-00000002".into());
        let results = search([&first, &second], "flight");
        assert_eq!(results[0].layout.name, "Flight Alpha");
    }
}
