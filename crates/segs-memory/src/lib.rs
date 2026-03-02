mod memory;
mod storage;

use std::{
    path::Path,
    sync::{OnceLock, RwLock},
};

use egui::{Context, Id, Ui};

use crate::memory::{Memory, Persistent, Temporary};

static MEMORY: OnceLock<MemoryControl> = OnceLock::new();

/// Initializes the memory system by creating a MemoryControl instance and
/// storing it in a global OnceLock. This function should be called once at
/// startup before any calls to mem
pub fn init_memory(folder_path: impl AsRef<Path>) -> storage::StorageResult<()> {
    let memory_control = MemoryControl::init(folder_path)?;
    MEMORY
        .set(memory_control)
        // SAFETY: We require that init_memory is called exactly once at startup before any calls to mem(), and that it
        // is never modified after initialization. This is guaranteed by the design of the library, as init_memory is
        // only called once at startup and then accessed immutably.
        .expect("MemoryControl has already been initialized");
    Ok(())
}

pub trait MemoryExt {
    /// Provides access to the MemoryControl instance, which manages both
    /// temporary and persistent in-memory values. This method allows you to
    /// read and write values in memory using the MemoryControl instance.
    fn mem(&self) -> &MemoryControl;
}

impl MemoryExt for Context {
    fn mem(&self) -> &MemoryControl {
        // SAFETY: We require that MemoryControl is initialized before any calls to
        // mem(), and that it is never modified after initialization. This is guaranteed
        // by the design of the library, as MemoryControl is only initialized once at
        // startup and then accessed immutably.
        MEMORY.get().unwrap()
    }
}

impl MemoryExt for Ui {
    fn mem(&self) -> &MemoryControl {
        self.ctx().mem()
    }
}

#[derive(Debug)]
pub struct MemoryControl(RwLock<Memory>);

impl MemoryControl {
    pub fn init(folder_path: impl AsRef<Path>) -> storage::StorageResult<Self> {
        let memory = Memory::init(folder_path)?;
        Ok(Self(RwLock::new(memory)))
    }
}

/// Temporary in-memory management.
impl MemoryControl {
    pub fn insert_temp<V: Temporary>(&self, id: Id, value: V) {
        let mut memory = self.0.write().unwrap();
        memory.temp_mut(id).insert(value);
    }

    pub fn get_temp<V>(&self, id: Id) -> Option<V>
    where
        V: Temporary + Clone,
    {
        let memory = self.0.read().unwrap();
        memory.temp(id).get().map(|v| v.read(|v: &V| v.clone()))
    }

    pub fn get_temp_or_insert<V>(&self, id: Id, default: V) -> V
    where
        V: Temporary + Clone,
    {
        let mut memory = self.0.write().unwrap();
        memory.temp_mut(id).or_insert(default).read(|v: &V| v.clone())
    }

    pub fn get_temp_or_insert_with<V>(&self, id: Id, default: impl FnOnce() -> V) -> V
    where
        V: Temporary + Clone,
    {
        let mut memory = self.0.write().unwrap();
        memory.temp_mut(id).or_insert_with(default).read(|v: &V| v.clone())
    }

    pub fn get_temp_or_default<V>(&self, id: Id) -> V
    where
        V: Temporary + Default + Clone,
    {
        let mut memory = self.0.write().unwrap();
        memory.temp_mut(id).or_default().read(|v: &V| v.clone())
    }

    pub fn remove_temp<V>(&self, id: Id) -> Option<V>
    where
        V: Temporary + Clone,
    {
        let mut memory = self.0.write().unwrap();
        memory.temp_mut(id).remove()
    }

    pub fn read_temp_or_insert<V: Temporary, O>(&self, id: Id, default: V, reader: impl FnOnce(&V) -> O) -> O {
        let mut memory = self.0.write().unwrap();
        memory.temp_mut(id).or_insert(default).read(reader)
    }

    pub fn read_temp_or_default<V, O>(&self, id: Id, writer: impl FnOnce(&mut V) -> O) -> O
    where
        V: Temporary + Default,
    {
        let mut memory = self.0.write().unwrap();
        memory.temp_mut(id).or_default().write(writer)
    }

    pub fn read_temp_or_insert_with<V: Temporary, O>(
        &self,
        id: Id,
        default: impl FnOnce() -> V,
        writer: impl FnOnce(&mut V) -> O,
    ) -> O {
        let mut memory = self.0.write().unwrap();
        memory.temp_mut(id).or_insert_with(default).write(writer)
    }

    pub fn write_temp_mut_or_insert<V: Temporary, O>(&self, id: Id, default: V, writer: impl FnOnce(&mut V) -> O) -> O {
        let mut memory = self.0.write().unwrap();
        memory.temp_mut(id).or_insert(default).write(writer)
    }

    pub fn write_temp_mut_or_default<V, O>(&self, id: Id, writer: impl FnOnce(&mut V) -> O) -> O
    where
        V: Temporary + Default,
    {
        let mut memory = self.0.write().unwrap();
        memory.temp_mut(id).or_default().write(writer)
    }

    pub fn write_temp_mut_or_insert_with<V: Temporary, O>(
        &self,
        id: Id,
        default: impl FnOnce() -> V,
        writer: impl FnOnce(&mut V) -> O,
    ) -> O {
        let mut memory = self.0.write().unwrap();
        memory.temp_mut(id).or_insert_with(default).write(writer)
    }
}

/// Persistent in-memory management.
impl MemoryControl {
    pub fn insert_perm<V: Persistent>(&self, id: Id, value: V) {
        let mut memory = self.0.write().unwrap();
        memory.perm_mut(id).insert(value);
    }

    pub fn get_perm<V: Persistent>(&self, id: Id) -> Option<V> {
        let mut memory = self.0.write().unwrap();
        memory.perm(id).get().map(|v| v.read(|v: &V| v.clone()))
    }

    pub fn get_perm_or_insert<V: Persistent>(&self, id: Id, default: V) -> V {
        let mut memory = self.0.write().unwrap();
        memory.perm_mut(id).or_insert(default).read(|v: &V| v.clone())
    }

    pub fn get_perm_or_insert_with<V: Persistent>(&self, id: Id, default: impl FnOnce() -> V) -> V {
        let mut memory = self.0.write().unwrap();
        memory.perm_mut(id).or_insert_with(default).read(|v: &V| v.clone())
    }

    pub fn get_perm_or_default<V>(&self, id: Id) -> V
    where
        V: Persistent + Default,
    {
        let mut memory = self.0.write().unwrap();
        memory.perm_mut(id).or_default().read(|v: &V| v.clone())
    }

    pub fn read_perm_or_insert<V: Persistent, O>(&self, id: Id, default: V, reader: impl FnOnce(&V) -> O) -> O {
        let mut memory = self.0.write().unwrap();
        memory.perm_mut(id).or_insert(default).read(reader)
    }

    pub fn read_perm_or_default<V, O>(&self, id: Id, writer: impl FnOnce(&mut V) -> O) -> O
    where
        V: Persistent + Default,
    {
        let mut memory = self.0.write().unwrap();
        memory.perm_mut(id).or_default().write(writer)
    }

    pub fn read_perm_or_insert_with<V: Persistent, O>(
        &self,
        id: Id,
        default: impl FnOnce() -> V,
        writer: impl FnOnce(&mut V) -> O,
    ) -> O {
        let mut memory = self.0.write().unwrap();
        memory.perm_mut(id).or_insert_with(default).write(writer)
    }

    pub fn write_perm_mut_or_insert<V: Persistent, O>(
        &self,
        id: Id,
        default: V,
        writer: impl FnOnce(&mut V) -> O,
    ) -> O {
        let mut memory = self.0.write().unwrap();
        memory.perm_mut(id).or_insert(default).write(writer)
    }

    pub fn write_perm_mut_or_default<V, O>(&self, id: Id, writer: impl FnOnce(&mut V) -> O) -> O
    where
        V: Persistent + Default,
    {
        let mut memory = self.0.write().unwrap();
        memory.perm_mut(id).or_default().write(writer)
    }

    pub fn write_perm_mut_or_insert_with<V: Persistent, O>(
        &self,
        id: Id,
        default: impl FnOnce() -> V,
        writer: impl FnOnce(&mut V) -> O,
    ) -> O {
        let mut memory = self.0.write().unwrap();
        memory.perm_mut(id).or_insert_with(default).write(writer)
    }

    pub fn sync_persistence(&self) -> storage::StorageResult<()> {
        let mut memory = self.0.write().unwrap();
        memory.flush_dirty()
    }
}
