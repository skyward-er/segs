use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use egui::{
    Ui,
    cache::{ComputerMut, FrameCache},
};

use crate::dataflow::adapter::DataAdapterInstanceToken;
use crate::dataflow::protocol::descriptor_index::DescriptorIndex;

/// Stores one adapter's matching rows for egui's frame cache.
///
/// The cache itself lives in the field selector. This value owns the resulting
/// node indices and retains the adapter token used to identify the cache entry.
/// Keeping the token alive prevents its allocation address from being reused by
/// a replacement adapter while stale rows are still reachable.
#[derive(Debug)]
pub struct CachedSearch {
    /// Keeps the token allocation alive while its cached rows remain reachable.
    _adapter_token: DataAdapterInstanceToken,
    /// Matching indices into the associated [`DescriptorIndex`].
    rows: Vec<usize>,
}

impl CachedSearch {
    /// Resolves matching rows for one installed adapter and query.
    fn resolve(index: &DescriptorIndex, adapter_token: &DataAdapterInstanceToken, query: &str) -> Self {
        Self {
            _adapter_token: adapter_token.clone(),
            rows: index.filtered_rows(query),
        }
    }

    /// Returns the indexed rows matching the cached query.
    pub fn rows(&self) -> &[usize] {
        &self.rows
    }
}

/// Identifies reusable search results by adapter lifecycle and query.
#[derive(Clone, Copy)]
struct SearchRequest<'a> {
    index: &'a DescriptorIndex,
    adapter_token: &'a DataAdapterInstanceToken,
    query: &'a str,
}

impl Hash for SearchRequest<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.adapter_token.hash(state);
        self.query.hash(state);
    }
}

/// Computes filtered descriptor rows for egui's frame cache.
#[derive(Default)]
struct SearchComputer;

impl ComputerMut<SearchRequest<'_>, Arc<CachedSearch>> for SearchComputer {
    fn compute(&mut self, request: SearchRequest<'_>) -> Arc<CachedSearch> {
        Arc::new(CachedSearch::resolve(
            request.index,
            request.adapter_token,
            request.query,
        ))
    }
}

type SearchFrameCache = FrameCache<Arc<CachedSearch>, SearchComputer>;

/// Returns filtered rows from egui's automatically evicted frame cache.
pub fn resolve_search_cache(
    ui: &Ui,
    index: &DescriptorIndex,
    adapter_token: &DataAdapterInstanceToken,
    query: &str,
) -> Arc<CachedSearch> {
    ui.ctx().memory_mut(|memory| {
        memory
            .caches
            .cache::<SearchFrameCache>()
            .get(SearchRequest {
                index,
                adapter_token,
                query,
            })
            .clone()
    })
}
