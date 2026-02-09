use std::{any::Any, marker::PhantomData};

use nohash::IntMap;
use serde::Serialize;

use crate::memory::Dirty;

pub type PermMap = IntMap<u64, Dirty<PermValue>>;
type CloneAnyType = Box<dyn CloneAny>;
type Serializer = fn(&CloneAnyType) -> Vec<u8>;
type Comparator = fn(&CloneAnyType, &CloneAnyType) -> bool;
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
        (*self.map.get(&self.key)?.as_ref().value)
            .as_any()
            .downcast_ref::<V>()
            .map(PermCell)
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
    V: SerializableAny + PartialEq + Clone,
{
    pub fn new(map: &'a mut PermMap, key: u64) -> Self {
        let occupied = map.get(&key).is_some_and(|v| v.as_ref().is::<V>());
        Self {
            map,
            key,
            occupied,
            _marker: PhantomData,
        }
    }

    pub fn or_insert_with(&mut self, default: impl FnOnce() -> V) -> PermMutCell<'_, V> {
        if !self.occupied {
            self.map.insert(self.key, Dirty::new_dirty(PermValue::new(default())));
            self.occupied = true;
        }
        PermMutCell::new(self.map.get_mut(&self.key).unwrap())
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

    /// Inserts the value if it doesn't exist, or replaces it if it's different
    /// from the existing value. Marks the entry as dirty if the value
    /// replaced was different or if the entry was newly inserted.
    pub fn insert(&mut self, value: V) {
        use std::collections::hash_map::Entry;
        match self.map.entry(self.key) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().replace(PermValue::new(value));
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(Dirty::new_dirty(PermValue::new(value)));
            }
        }
        self.occupied = true;
    }
}

#[derive(Clone)]
pub struct PermValue {
    pub value: Box<dyn CloneAny>,
    pub serializer: Serializer,
    pub comparator: Comparator,
}

impl PartialEq for PermValue {
    fn eq(&self, other: &Self) -> bool {
        (self.comparator)(&self.value, &other.value)
    }
}

impl PermValue {
    pub fn new<V: SerializableAny + PartialEq + Clone>(value: V) -> Self {
        Self {
            value: Box::new(value),
            serializer: |v| postcard::to_allocvec((**v).as_any().downcast_ref::<V>().unwrap()).unwrap(),
            comparator: |a, b| {
                (**a).as_any().downcast_ref::<V>().unwrap() == (**b).as_any().downcast_ref::<V>().unwrap()
            },
        }
    }

    fn is<V: 'static>(&self) -> bool {
        (*self.value).as_any().is::<V>()
    }
}

pub struct PermCell<'a, V>(pub &'a V);

impl<'a, V> PermCell<'a, V> {
    pub fn read<O>(&self, reader: impl FnOnce(&V) -> O) -> O {
        reader(self.0)
    }
}

pub struct PermMutCell<'a, V: PartialEq> {
    value: &'a mut Dirty<PermValue>,
    _marker: PhantomData<V>,
}

impl<'a, V: PartialEq + 'static> PermMutCell<'a, V> {
    fn new(value: &'a mut Dirty<PermValue>) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    pub fn read<O>(&self, reader: impl FnOnce(&V) -> O) -> O {
        // SAFETY: This cannot unwrap to None because Self::new can only be called with
        // a PermValue that contains a V, and the value is never replaced with a
        // different type.
        reader((*self.value.as_ref().value).as_any().downcast_ref::<V>().unwrap())
    }

    pub fn write<O>(&mut self, writer: impl FnOnce(&mut V) -> O) -> O {
        self.value.update_and_check(|v| {
            // SAFETY: This cannot unwrap to None because Self::new can only be called with
            // a PermValue that contains a V, and the value is never replaced with a
            // different type.
            let v = (*v.value).as_any_mut().downcast_mut::<V>().unwrap();
            writer(v)
        })
    }
}

pub trait CloneAny: Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn CloneAny>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> CloneAny for T
where
    T: Any + Clone + Send + Sync + 'static,
{
    fn clone_box(&self) -> Box<dyn CloneAny> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Clone for Box<dyn CloneAny> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

impl std::fmt::Debug for PermValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PermValue").finish_non_exhaustive()
    }
}
