use std::{fmt::Debug, path::Path};

use fjall::{Database, Keyspace, KeyspaceCreateOptions, UserKey};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Db(#[from] fjall::Error),

    #[error("Encoding error: {0}")]
    Encode(#[from] postcard::Error),
}

pub type StorageResult<T> = Result<T, StorageError>;

pub struct Storage {
    db: Database,
}

impl Storage {
    /// Initializes the storage system by opening a database at the given folder
    /// path.
    pub fn init(folder_path: impl AsRef<Path>) -> StorageResult<Self> {
        let db = Database::builder(folder_path).open()?;
        Ok(Self { db })
    }

    /// Inserts bytes into the specified keyspace under the given key.
    pub fn insert_raw(&self, keyspace: &str, key: impl Into<UserKey>, value: &[u8]) -> StorageResult<()> {
        Ok(self.keyspace(keyspace)?.insert(key, value)?)
    }

    /// Inserts any type that implements Serialize
    pub fn insert<T>(&self, keyspace: &str, key: impl Into<UserKey>, value: &T) -> StorageResult<()>
    where
        T: Serialize,
    {
        let keyspace = self.keyspace(keyspace)?;

        // Serialize the value into a byte vector using postcard
        let bytes = postcard::to_allocvec(value)?;

        keyspace.insert(key, bytes)?;
        Ok(())
    }

    /// Retrieves and deserializes into an owned T
    pub fn get<T>(&self, keyspace: &str, key: impl AsRef<[u8]>) -> StorageResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let keyspace = self.keyspace(keyspace)?;

        // Fjall's get returns an Option<MultiVector>, which derefs to &[u8]
        let Some(bytes) = keyspace.get(key)? else {
            return Ok(None);
        };

        let value = postcard::from_bytes(&bytes)?;
        Ok(Some(value))
    }

    /// Removes a key from the specified keyspace.
    pub fn remove(&self, keyspace: &str, key: impl Into<UserKey>) -> StorageResult<()> {
        let keyspace = self.keyspace(keyspace)?;
        keyspace.remove(key)?;
        Ok(())
    }

    // Persists any pending changes to disk.
    pub fn flush(&self) -> StorageResult<()> {
        self.db.persist(fjall::PersistMode::SyncData)?;
        Ok(())
    }

    #[inline]
    fn keyspace(&self, name: &str) -> StorageResult<Keyspace> {
        Ok(self.db.keyspace(name, KeyspaceCreateOptions::default)?)
    }
}

impl Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage").finish()
    }
}
