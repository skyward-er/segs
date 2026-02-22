use std::{any::Any, marker::PhantomData};

use nohash::IntMap;

pub type TempMap = IntMap<u64, Box<dyn Any + 'static + Send + Sync>>;

pub struct TempEntry<M, V> {
    map: M,
    key: u64,
    occupied: bool,
    _marker: PhantomData<V>,
}

impl<'a, V> TempEntry<&'a TempMap, V>
where
    V: 'static + Send + Sync,
{
    pub fn new(map: &'a TempMap, key: u64) -> Self {
        let occupied = map.get(&key).is_some_and(|v| v.is::<V>());
        Self {
            map,
            key,
            occupied,
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> Option<TempCell<&V>> {
        self.map.get(&self.key)?.downcast_ref::<V>().map(TempCell)
    }
}

impl<'a, V> TempEntry<&'a mut TempMap, V>
where
    V: 'static + Send + Sync,
{
    pub fn new(map: &'a mut TempMap, key: u64) -> Self {
        let occupied = map.get(&key).is_some_and(|v| v.is::<V>());
        Self {
            map,
            key,
            occupied,
            _marker: PhantomData,
        }
    }

    pub fn or_insert_with(&mut self, default: impl FnOnce() -> V) -> TempCell<&mut V> {
        if !self.occupied {
            self.map.insert(self.key, Box::new(default()));
            self.occupied = true;
        }
        TempCell(self.map.get_mut(&self.key).unwrap().downcast_mut::<V>().unwrap())
    }

    pub fn or_insert(&mut self, default: V) -> TempCell<&mut V> {
        self.or_insert_with(|| default)
    }

    pub fn or_default(&mut self) -> TempCell<&mut V>
    where
        V: Default,
    {
        self.or_insert_with(V::default)
    }

    pub fn insert(&mut self, value: V) {
        self.map.insert(self.key, Box::new(value));
        self.occupied = true;
    }

    pub fn remove(&mut self) -> Option<V> {
        self.occupied = false;
        self.map.remove(&self.key)?.downcast::<V>().ok().map(|v| *v)
    }
}

pub struct TempCell<V>(V);

impl<V> TempCell<&V> {
    pub fn read<O>(&self, reader: impl FnOnce(&V) -> O) -> O {
        reader(self.0)
    }
}

impl<V> TempCell<&mut V> {
    pub fn read<O>(&self, reader: impl FnOnce(&V) -> O) -> O {
        reader(self.0)
    }

    pub fn write<O>(&mut self, writer: impl FnOnce(&mut V) -> O) -> O {
        writer(self.0)
    }
}
