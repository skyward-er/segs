mod persistent;
mod temporary;

use std::{any::Any, path::Path};

use egui::{Id, ahash::HashSet};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    memory::{
        persistent::{PermEntry, PermMap, PermMutEntry, PermValue},
        temporary::{TempEntry, TempMap},
    },
    storage::{Storage, StorageResult},
};

pub trait Temporary: Any + 'static + Send + Sync {}
impl<T: Any + 'static + Send + Sync> Temporary for T {}
pub trait Persistent: Any + 'static + Send + Sync + PartialEq + Serialize + DeserializeOwned + Clone {}
impl<T: Any + 'static + Send + Sync + PartialEq + Serialize + DeserializeOwned + Clone> Persistent for T {}

#[derive(Debug)]
pub struct Memory {
    /// Temporary in-memory values that are not persisted to storage. These are
    /// useful for storing transient state that is only relevant during a single
    /// execution of the program, such as UI state or cached computations.
    temp_map: TempMap,

    /// Persistent in-memory values that are backed by storage. These values are
    /// loaded from storage on initialization and written back to storage when
    /// modified. They are useful for storing state that needs to persist across
    /// executions of the program, such as user settings or cached data that is
    /// expensive to compute.
    perm_map: PermMap,

    /// The list of keys that are currently stored in the storage backend. This
    /// is used to determine which keys need to be loaded into memory on
    /// initialization and which keys need to be removed from storage when
    /// they are deleted from memory.
    stored_keys: Dirty<HashSet<u64>>,

    /// The storage backend that handles loading and saving persistent values.
    storage: Storage,
}

impl Memory {
    pub fn init(folder_path: impl AsRef<Path>) -> StorageResult<Self> {
        let storage = Storage::init(folder_path)?;
        let stored_keys = Self::retrieve_all_stored(&storage)?;
        Ok(Self {
            temp_map: TempMap::default(),
            perm_map: PermMap::default(),
            stored_keys: Dirty::new_clean(stored_keys.into_iter().collect()),
            storage,
        })
    }

    fn retrieve_all_stored(storage: &Storage) -> StorageResult<Vec<u64>> {
        storage
            .get("metadata", b"persistent_keys")
            // If we fail to retrieve the keys, we return an empty list. This way we can handle the case where the
            // storage is empty or the keys are not present without causing errors.
            .map(|keys: Option<Vec<u64>>| keys.unwrap_or_default())
    }
}

/// Access to IdMapped in-memory values, both temporary and persistent.
impl Memory {
    pub fn perm<V: Persistent>(&mut self, id: Id) -> PermEntry<'_, V> {
        if self.is_archived_but_not_loaded(id.value()) {
            self.load_or_drop::<V>(id.value())
                .expect("Failed to load value from storage"); // FIXME log if error
        }
        PermEntry::new(&self.perm_map, id.value())
    }

    pub fn perm_mut<V: Persistent>(&mut self, id: Id) -> PermMutEntry<'_, V> {
        if self.is_archived_but_not_loaded(id.value()) {
            self.load_or_drop::<V>(id.value())
                .expect("Failed to load value from storage"); // FIXME log if error
        }
        PermMutEntry::new(&mut self.perm_map, id.value())
    }

    pub fn temp<V: Temporary>(&self, id: Id) -> TempEntry<&TempMap, V> {
        TempEntry::<&TempMap, V>::new(&self.temp_map, id.value())
    }

    pub fn temp_mut<V: Temporary>(&mut self, id: Id) -> TempEntry<&mut TempMap, V> {
        TempEntry::<&mut TempMap, V>::new(&mut self.temp_map, id.value())
    }
}

/// Handle persistent state using storage layer.
impl Memory {
    pub fn flush_dirty(&mut self) -> StorageResult<()> {
        // 1. collect all dirty entries and serialize them. This way we limit the
        //    mutable borrow and collect all the data we need before we start writing to
        //    storage.
        let mut to_archive = vec![];
        for (key, cell) in self.perm_map.iter_mut() {
            if cell.is_dirty() {
                let PermValue { value, serializer, .. } = cell.as_ref();
                to_archive.push(ArchivedValue::new(*key, serializer(value)));
                cell.clear_dirty();
            }
        }
        // 2. write all the archived values to storage.
        for archived in to_archive {
            self.store_raw(archived.key, &archived.bytes)?;
        }
        // 3. update the list of stored keys in storage. This way we can keep track of
        //    which keys are currently stored and which are not.
        if self.stored_keys.is_dirty() {
            self.storage.insert(
                "metadata",
                b"persistent_keys",
                &self.perm_map.keys().copied().collect::<Vec<u64>>(),
            )?;
            self.stored_keys.clear_dirty();
        }
        // 4. we may flush, but we avoid doing so to do not impact rendering
        //    performance. Instead, we rely on the fact that the storage layer will
        //    flush periodically and on drop.
        Ok(())
    }

    /// Loads the value for the given key from storage and inserts it into the
    /// persistent map if it exists. Otherswise, it removes the key from storage
    /// to ensure consistency between memory and storage.
    fn load_or_drop<T: Persistent>(&mut self, key: u64) -> StorageResult<()> {
        if let Some(value) = self.load_persistent::<T>(key)? {
            self.perm_map.insert(key, Dirty::new_clean(PermValue::new(value)));
        } else {
            self.remove_persistent(key)?;
        }
        Ok(())
    }

    fn is_archived_but_not_loaded(&self, key: u64) -> bool {
        self.stored_keys.as_ref().contains(&key) && !self.perm_map.contains_key(&key)
    }

    fn store_raw(&mut self, key: u64, value: &[u8]) -> StorageResult<()> {
        self.stored_keys.update_and_mark(|h| h.insert(key));
        self.storage.insert_raw("persistent", key.to_be_bytes(), value)
    }

    fn load_persistent<T: DeserializeOwned>(&self, key: u64) -> StorageResult<Option<T>> {
        self.storage.get("persistent", key.to_be_bytes())
    }

    fn remove_persistent(&mut self, key: u64) -> StorageResult<()> {
        self.stored_keys.update_and_mark(|h| h.remove(&key));
        self.storage.remove("persistent", key.to_be_bytes())
    }
}

#[derive(Debug)]
pub struct Dirty<T: PartialEq> {
    inner: T,
    dirty: bool,
}

impl<T: PartialEq> Dirty<T> {
    fn new_clean(inner: T) -> Self {
        Self { inner, dirty: false }
    }

    fn new_dirty(inner: T) -> Self {
        Self { inner, dirty: true }
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn replace(&mut self, new_inner: T) {
        if self.inner != new_inner {
            self.inner = new_inner;
            self.dirty = true;
        }
    }

    fn update_and_mark(&mut self, f: impl FnOnce(&mut T) -> bool) {
        if f(&mut self.inner) {
            self.dirty = true;
        }
    }

    fn map<U: Eq>(self, f: impl FnOnce(T) -> U) -> Dirty<U> {
        Dirty {
            inner: f(self.inner),
            dirty: self.dirty,
        }
    }
}

impl<T: PartialEq + Clone> Dirty<T> {
    fn update_and_check<O>(&mut self, f: impl FnOnce(&mut T) -> O) -> O {
        let old_inner = self.inner.clone();
        let result = f(&mut self.inner);
        if self.inner != old_inner {
            self.dirty = true;
        }
        result
    }
}

impl<T: PartialEq> AsRef<T> for Dirty<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T: PartialEq> AsMut<T> for Dirty<T> {
    fn as_mut(&mut self) -> &mut T {
        self.dirty = true;
        &mut self.inner
    }
}

struct ArchivedValue {
    key: u64,
    bytes: Vec<u8>,
}

impl ArchivedValue {
    fn new(key: u64, bytes: Vec<u8>) -> Self {
        Self { key, bytes }
    }
}
