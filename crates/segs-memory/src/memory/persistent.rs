use std::{any::Any, marker::PhantomData};

use nohash::IntMap;
use serde::Serialize;

pub type PermMap = IntMap<u64, PermValue>;
type Serializer = fn(&Box<dyn Any + 'static + Send + Sync>) -> Vec<u8>;
pub trait SerializableAny: Serialize + Any + 'static + Send + Sync {}
impl<T: Serialize + Any + 'static + Send + Sync> SerializableAny for T {}

pub struct PermEntry<'a, V> {
    map: &'a PermMap,
    key: u64,
    _marker: PhantomData<V>,
}

impl<'a, V> PermEntry<'a, V>
where
    V: 'static + Send + Sync,
{
    pub fn new(map: &'a PermMap, key: u64) -> Self {
        Self {
            map,
            key,
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> Option<PermCell<'_, V>> {
        self.map.get(&self.key)?.value.downcast_ref::<V>().map(PermCell)
    }
}

pub struct PermMutEntry<'a, V> {
    map: &'a mut PermMap,
    key: u64,
    occupied: bool,
    _marker: PhantomData<V>,
}

impl<'a, V> PermMutEntry<'a, V>
where
    V: SerializableAny,
{
    pub fn new(map: &'a mut PermMap, key: u64) -> Self {
        let occupied = map.get(&key).is_some_and(|v| v.is::<V>());
        Self {
            map,
            key,
            occupied,
            _marker: PhantomData,
        }
    }

    pub fn or_insert_with(&mut self, default: impl FnOnce() -> V) -> PermMutCell<'_, V> {
        if !self.occupied {
            self.map.insert(self.key, PermValue::dirty(default()));
            self.occupied = true;
        }
        let PermValue { value, dirty, .. } = self.map.get_mut(&self.key).unwrap();
        PermMutCell {
            borrow: value.downcast_mut::<V>().unwrap(),
            dirty,
        }
    }

    pub fn or_insert(&mut self, default: V) -> PermMutCell<'_, V> {
        self.or_insert_with(|| default)
    }

    pub fn or_default(&mut self) -> PermMutCell<'_, V>
    where
        V: Default,
    {
        self.or_insert_with(V::default)
    }

    pub fn insert(&mut self, value: V) {
        self.map.insert(self.key, PermValue::dirty(value));
        self.occupied = true;
    }
}

#[derive(Debug)]
pub struct PermValue {
    pub value: Box<dyn Any + 'static + Send + Sync>,
    /// Whether the value has been modified since it was loaded from storage.
    /// This is used to determine whether the value needs to be written back to
    /// storage.
    pub dirty: bool,
    pub serializer: Serializer,
}

impl PermValue {
    fn new<V: SerializableAny>(value: V, dirty: bool) -> Self {
        Self {
            value: Box::new(value),
            dirty,
            serializer: |v| postcard::to_allocvec(v.downcast_ref::<V>().unwrap()).unwrap(),
        }
    }

    pub fn dirty<V: SerializableAny>(value: V) -> Self {
        Self::new(value, true)
    }

    pub fn not_dirty<V: SerializableAny>(value: V) -> Self {
        Self::new(value, false)
    }

    fn is<V: 'static>(&self) -> bool {
        self.value.is::<V>()
    }
}

pub struct PermCell<'a, V>(pub &'a V);

impl<'a, V> PermCell<'a, V> {
    pub fn read<O>(&self, reader: impl FnOnce(&V) -> O) -> O {
        reader(self.0)
    }
}

pub struct PermMutCell<'a, V> {
    borrow: &'a mut V,
    dirty: &'a mut bool,
}

impl<'a, V> PermMutCell<'a, V> {
    pub fn read<O>(&self, reader: impl FnOnce(&V) -> O) -> O {
        reader(self.borrow)
    }

    pub fn write<O>(&mut self, writer: impl FnOnce(&mut V) -> O) -> O {
        *self.dirty = true;
        writer(self.borrow)
    }
}
