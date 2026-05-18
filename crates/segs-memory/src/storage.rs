use std::{fmt::Debug, path::Path};

use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Encoding error: {0}")]
    Encode(#[from] postcard::Error),

    #[error("Storage backend error: {0}")]
    Backend(String),
}

pub type StorageResult<T> = Result<T, StorageError>;

// ===== Native (fjall) implementation =====

#[cfg(not(target_arch = "wasm32"))]
use fjall::{Database, Keyspace, KeyspaceCreateOptions};

#[cfg(not(target_arch = "wasm32"))]
impl From<fjall::Error> for StorageError {
    fn from(e: fjall::Error) -> Self {
        StorageError::Backend(e.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Storage {
    db: Database,
}

#[cfg(not(target_arch = "wasm32"))]
impl Storage {
    /// Initializes the storage system by opening a database at the given folder
    /// path.
    pub fn init(folder_path: impl AsRef<Path>) -> StorageResult<Self> {
        let db = Database::builder(folder_path).open()?;
        Ok(Self { db })
    }

    /// Inserts bytes into the specified keyspace under the given key.
    pub fn insert_raw(&self, keyspace: &str, key: impl Into<fjall::UserKey>, value: &[u8]) -> StorageResult<()> {
        Ok(self.keyspace(keyspace)?.insert(key, value)?)
    }

    /// Inserts any type that implements Serialize
    pub fn insert<T>(&self, keyspace: &str, key: impl Into<fjall::UserKey>, value: &T) -> StorageResult<()>
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
    pub fn remove(&self, keyspace: &str, key: impl Into<fjall::UserKey>) -> StorageResult<()> {
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

#[cfg(not(target_arch = "wasm32"))]
impl Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage").finish()
    }
}

// ===== Web (localStorage) implementation =====
//
// Keys are stored as "segs:{keyspace}:{key_hex}" in localStorage.
// Values are postcard-serialized bytes encoded as base64 strings, since
// localStorage only accepts strings.

#[cfg(target_arch = "wasm32")]
use base64::{Engine as _, engine::general_purpose::STANDARD};

#[cfg(target_arch = "wasm32")]
impl From<base64::DecodeError> for StorageError {
    fn from(e: base64::DecodeError) -> Self {
        StorageError::Backend(e.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
pub struct Storage {
    inner: web_sys::Storage,
}

#[cfg(target_arch = "wasm32")]
impl Storage {
    pub fn init(_folder_path: impl AsRef<Path>) -> StorageResult<Self> {
        let inner = web_sys::window()
            .ok_or_else(|| StorageError::Backend("No window".into()))?
            .local_storage()
            .map_err(|e| StorageError::Backend(format!("{e:?}")))?
            .ok_or_else(|| StorageError::Backend("localStorage not available".into()))?;
        Ok(Self { inner })
    }

    pub fn insert_raw(&self, keyspace: &str, key: impl AsRef<[u8]>, value: &[u8]) -> StorageResult<()> {
        self.inner
            .set_item(&flat_key(keyspace, key.as_ref()), &STANDARD.encode(value))
            .map_err(|e| StorageError::Backend(format!("{e:?}")))?;
        Ok(())
    }

    pub fn insert<T>(&self, keyspace: &str, key: impl AsRef<[u8]>, value: &T) -> StorageResult<()>
    where
        T: Serialize,
    {
        self.insert_raw(keyspace, key, &postcard::to_allocvec(value)?)
    }

    pub fn get<T>(&self, keyspace: &str, key: impl AsRef<[u8]>) -> StorageResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let Some(encoded) = self
            .inner
            .get_item(&flat_key(keyspace, key.as_ref()))
            .map_err(|e| StorageError::Backend(format!("{e:?}")))?
        else {
            return Ok(None);
        };
        let bytes = STANDARD.decode(encoded)?;
        Ok(Some(postcard::from_bytes(&bytes)?))
    }

    pub fn remove(&self, keyspace: &str, key: impl AsRef<[u8]>) -> StorageResult<()> {
        self.inner
            .remove_item(&flat_key(keyspace, key.as_ref()))
            .map_err(|e| StorageError::Backend(format!("{e:?}")))?;
        Ok(())
    }

    pub fn flush(&self) -> StorageResult<()> {
        Ok(()) // localStorage writes are immediately durable
    }
}

#[cfg(target_arch = "wasm32")]
impl Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage").finish()
    }
}

#[cfg(target_arch = "wasm32")]
fn flat_key(keyspace: &str, key: &[u8]) -> String {
    let hex = key.iter().fold(String::with_capacity(key.len() * 2), |mut s, b| {
        use std::fmt::Write;
        write!(s, "{b:02x}").unwrap();
        s
    });
    format!("segs:{keyspace}:{hex}")
}
