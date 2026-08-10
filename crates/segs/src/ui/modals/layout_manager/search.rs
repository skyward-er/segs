use std::{cmp::Ordering, ops::Range, sync::Arc};

use aho_corasick::AhoCorasick;
use smallvec::SmallVec;

use crate::layout::Layout;

/// Owns the layout identity and match positions needed to render a search row.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub slug: String,
    pub name: String,
    pub matches: SmallVec<[Range<usize>; 3]>,
}

/// Stores only the latest query and its results for reuse across idle UI frames.
#[derive(Debug)]
pub struct CachedSearch {
    pub query: String,
    pub results: Vec<SearchResult>,
}

/// Filters layouts case-insensitively and ranks earlier name matches first.
pub fn search<'a>(layouts: impl IntoIterator<Item = &'a Layout>, query: &str) -> Vec<SearchResult> {
    if query.is_empty() {
        let mut results = layouts
            .into_iter()
            .map(|layout| SearchResult {
                slug: layout.slug.clone(),
                name: layout.name.clone(),
                matches: SmallVec::new(),
            })
            .collect::<Vec<_>>();
        results.sort_by(|a, b| a.name.cmp(&b.name));
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
            (!matches.is_empty()).then(|| SearchResult {
                slug: layout.slug.clone(),
                name: layout.name.clone(),
                matches,
            })
        })
        .collect::<Vec<_>>();
    results.sort_by(|a, b| compare_matches(&a.matches, &b.matches).then_with(|| a.name.cmp(&b.name)));
    results
}

/// Returns the latest cached search when its query matches, otherwise replaces it.
pub fn resolve_cached_search<'a>(
    cached: Option<Arc<CachedSearch>>,
    layouts: impl IntoIterator<Item = &'a Layout>,
    query: &str,
) -> (Arc<CachedSearch>, bool) {
    if let Some(cached) = cached
        && cached.query == query
    {
        return (cached, false);
    }

    (
        Arc::new(CachedSearch {
            query: query.to_owned(),
            results: search(layouts, query),
        }),
        true,
    )
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
        // Create names where the same query starts at different positions
        let first = Layout::empty("Alpha Flight".into(), "alpha-flight-00000001".into());
        let second = Layout::empty("Flight Alpha".into(), "flight-alpha-00000002".into());

        // Search results should prioritize the name with the earlier match
        let results = search([&first, &second], "flight");
        assert_eq!(results[0].name, "Flight Alpha");
    }

    #[test]
    fn unchanged_query_reuses_cached_results() {
        // Populate the latest-search cache for an initial query
        let layout = Layout::empty("Flight Alpha".into(), "flight-alpha-00000001".into());
        let (initial, recomputed) = resolve_cached_search(None, [&layout], "flight");
        assert!(recomputed);

        // Resolving the same query should return the exact cached allocation
        let (reused, recomputed) = resolve_cached_search(Some(initial.clone()), [&layout], "flight");
        assert!(!recomputed);
        assert!(Arc::ptr_eq(&initial, &reused));
    }

    #[test]
    fn changed_or_invalidated_query_recomputes_results() {
        // Cache a query that matches only the first layout
        let first = Layout::empty("Flight Alpha".into(), "flight-alpha-00000001".into());
        let second = Layout::empty("Ground Test".into(), "ground-test-00000002".into());
        let (initial, _) = resolve_cached_search(None, [&first, &second], "flight");
        assert_eq!(initial.results.len(), 1);

        // Changing the query should replace the cache with differently filtered results
        let (changed, recomputed) = resolve_cached_search(Some(initial.clone()), [&first, &second], "test");
        assert!(recomputed);
        assert!(!Arc::ptr_eq(&initial, &changed));
        assert_eq!(changed.results[0].name, "Ground Test");

        // Dropping the cache should force recomputation even when the query is unchanged
        let (invalidated, recomputed) = resolve_cached_search(None, [&first, &second], "flight");
        assert!(recomputed);
        assert!(!Arc::ptr_eq(&initial, &invalidated));
    }
}
