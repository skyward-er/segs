mod persistent;
mod temporary;

use std::path::Path;

use egui::{Id, ahash::HashSet};
use serde::de::DeserializeOwned;

use crate::{
    memory::{
        persistent::{PermEntry, PermMap, PermMutEntry, PermValue, SerializableAny},
        temporary::{TempEntry, TempMap},
    },
    storage::{Storage, StorageResult},
};

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
    stored_keys: HashSet<u64>,
    dirty_keys: bool,

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
            stored_keys: stored_keys.into_iter().collect(),
            dirty_keys: false,
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
    pub fn perm<V>(&mut self, id: Id) -> PermEntry<'_, V>
    where
        V: SerializableAny + DeserializeOwned,
    {
        if self.is_archived_but_not_loaded(id.value()) {
            self.load_or_drop::<V>(id.value())
                .expect("Failed to load value from storage"); // FIXME log if error
        }
        PermEntry::new(&self.perm_map, id.value())
    }

    pub fn perm_mut<V>(&mut self, id: Id) -> PermMutEntry<'_, V>
    where
        V: SerializableAny + DeserializeOwned,
    {
        if self.is_archived_but_not_loaded(id.value()) {
            self.load_or_drop::<V>(id.value())
                .expect("Failed to load value from storage"); // FIXME log if error
        }
        PermMutEntry::new(&mut self.perm_map, id.value())
    }

    pub fn temp<V>(&self, id: Id) -> TempEntry<&TempMap, V>
    where
        V: 'static + Send + Sync,
    {
        TempEntry::<&TempMap, V>::new(&self.temp_map, id.value())
    }

    pub fn temp_mut<V>(&mut self, id: Id) -> TempEntry<&mut TempMap, V>
    where
        V: 'static + Send + Sync,
    {
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
        for (key, value) in self.perm_map.iter_mut() {
            let PermValue {
                value,
                dirty,
                serializer,
            } = value;
            if *dirty {
                to_archive.push(ArchivedValue::new(*key, serializer(value)));
                *dirty = false;
            }
        }
        // 2. write all the archived values to storage.
        for archived in to_archive {
            self.store_raw(archived.key, &archived.bytes)?;
        }
        // 3. update the list of stored keys in storage. This way we can keep track of
        //    which keys are currently stored and which are not.
        if self.dirty_keys {
            self.storage.insert(
                "metadata",
                b"persistent_keys",
                &self.perm_map.keys().copied().collect::<Vec<u64>>(),
            )?;
            self.dirty_keys = false;
        }
        // 4. we may flush, but we avoid doing so to do not impact rendering
        //    performance. Instead, we rely on the fact that the storage layer will
        //    flush periodically and on drop.
        Ok(())
    }

    /// Loads the value for the given key from storage and inserts it into the
    /// persistent map if it exists. Otherswise, it removes the key from storage
    /// to ensure consistency between memory and storage.
    fn load_or_drop<T>(&mut self, key: u64) -> StorageResult<()>
    where
        T: SerializableAny + DeserializeOwned,
    {
        if let Some(value) = self.load_persistent::<T>(key)? {
            self.perm_map.insert(key, PermValue::not_dirty(value));
        } else {
            self.remove_persistent(key)?;
        }
        Ok(())
    }

    fn is_archived_but_not_loaded(&self, key: u64) -> bool {
        self.stored_keys.contains(&key) && !self.perm_map.contains_key(&key)
    }

    fn store_raw(&mut self, key: u64, value: &[u8]) -> StorageResult<()> {
        if self.stored_keys.insert(key) {
            self.dirty_keys = true;
        }
        self.storage.insert_raw("persistent", key.to_be_bytes(), value)
    }

    fn load_persistent<T: DeserializeOwned>(&self, key: u64) -> StorageResult<Option<T>> {
        self.storage.get("persistent", key.to_be_bytes())
    }

    fn remove_persistent(&mut self, key: u64) -> StorageResult<()> {
        if self.stored_keys.remove(&key) {
            self.dirty_keys = true;
        }
        self.storage.remove("persistent", key.to_be_bytes())
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
