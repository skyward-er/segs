//! Module for caching expensive UI calls using egui's temporary memory storage.
//! It provides utilities for caching the results of functions to avoid frequent recalculations.

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};

use egui::Context;
use serialport::SerialPortInfo;

use crate::{communication, error::ErrInstrument};

const SERIAL_PORT_REFRESH_INTERVAL: Duration = Duration::from_millis(500);
const SHORT_REFRESH_INTERVAL: Duration = Duration::from_millis(500);
const INDEF_REFRESH_INTERVAL: Duration = Duration::MAX;

/// Internal helper function that caches the result of a given function call for a specified duration.
///
/// # Arguments
/// * `ctx` - The egui context used for caching.
/// * `id` - The unique identifier for the cached item.
/// * `fun` - The function whose return value is to be cached.
/// * `expiration_duration` - The duration after which the cache should be refreshed.
fn call<T, F>(ctx: &egui::Context, id: egui::Id, fun: F, expiration_duration: Duration) -> T
where
    F: Fn() -> T,
    T: Clone + Send + Sync + 'static,
{
    ctx.memory_mut(|m| {
        match m.data.get_temp::<(T, Instant)>(id) {
            None => {
                m.data.insert_temp(id, (fun(), Instant::now()));
            }
            Some((_, i)) if i.elapsed() >= expiration_duration => {
                m.data.insert_temp(id, (fun(), Instant::now()));
            }
            _ => {}
        }
        m.data.get_temp::<(T, Instant)>(id).log_unwrap().0
    })
}

/// A trait to extend egui's Context with a caching function.
pub trait CacheCall {
    /// Calls the provided function and caches its result. Every time this
    /// function is called, it will return the cached value if it is still
    /// valid.
    ///
    /// # Arguments
    /// * `id` - A unique identifier for the cached value.
    /// * `fun` - The function to be cached.
    /// * `expiration_duration` - The cache expiration duration.
    fn call_cached<F, T>(&self, id: egui::Id, fun: F, expiration_duration: Duration) -> T
    where
        F: Fn() -> T,
        T: Clone + Send + Sync + 'static;

    fn call_cached_short<F, T, H>(&self, hashable: &H, fun: F) -> T
    where
        F: Fn() -> T,
        T: Clone + Send + Sync + 'static,
        H: Hash,
    {
        let id = egui::Id::new(hashable);
        self.call_cached(id, fun, SHORT_REFRESH_INTERVAL)
    }

    fn call_cached_indef<F, T, H>(&self, hashable: &H, fun: F) -> T
    where
        F: Fn() -> T,
        T: Clone + Send + Sync + 'static,
        H: Hash,
    {
        let id = egui::Id::new(hashable);
        self.call_cached(id, fun, INDEF_REFRESH_INTERVAL)
    }
}

impl CacheCall for egui::Context {
    /// Implements the caching call using the internal `call` function.
    fn call_cached<F, T>(&self, id: egui::Id, fun: F, expiration_duration: Duration) -> T
    where
        F: Fn() -> T,
        T: Clone + Send + Sync + 'static,
    {
        call(self, id, fun, expiration_duration)
    }
}

/// ChangeTracker manages the tracking of state changes using an integrity digest.
///
/// The `integrity_digest` field holds a 64-bit unsigned integer that represents
/// a summary (or hash) of the current state. This can be used to verify that the
/// cached UI state remains consistent, and to quickly detect any modifications.
pub struct ChangeTracker {
    integrity_digest: u64,
}

impl ChangeTracker {
    /// Records the initial state of a hashable value by computing its hash digest.
    ///
    /// This method takes a reference to any value that implements the `Hash` trait,
    /// computes its hash using the default hasher, and stores the resulting digest in a
    /// newly created `ChangeTracker` instance. This digest serves as a reference point
    /// for future state comparisons.
    ///
    /// # Parameters
    ///
    /// - `state`: A reference to the value whose state is to be recorded.
    ///
    /// # Returns
    ///
    /// A `ChangeTracker` initialized with the computed hash digest.
    ///
    /// # Examples
    ///
    /// ```
    /// let initial_tracker = ChangeTracker::record_initial_state(&state);
    /// ```
    pub fn record_initial_state<T: Hash>(state: &T) -> Self {
        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        let integrity_digest = hasher.finish();
        Self { integrity_digest }
    }

    /// Checks whether the hash of the current state differs from the initially recorded state.
    ///
    /// This method computes the hash digest of the current state (which must implement the
    /// `Hash` trait) and compares it with the digest stored in the `ChangeTracker`. If the digests
    /// differ, it indicates that the state has changed since the initial recording.
    ///
    /// # Parameters
    ///
    /// - `state`: A reference to the current state to be checked for changes.
    ///
    /// # Returns
    ///
    /// `true` if the current state's hash digest does not match the initially recorded digest,
    /// indicating a change; `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// if tracker.has_changed(&state) {
    ///     println!("The state has changed.");
    /// }
    /// ```
    pub fn has_changed<T: Hash>(&self, state: &T) -> bool {
        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        self.integrity_digest != hasher.finish()
    }
}
