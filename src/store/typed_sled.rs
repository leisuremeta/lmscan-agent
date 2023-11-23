use std::{marker::PhantomData, result::Result};

use serde::{Deserialize, Serialize};
use sled::{Db, Error};

use crate::library::common::{from_ivec, into_byte_vec};

pub struct TypedSled<K, V> {
    pub db: Db,
    _marker: PhantomData<(K, V)>,
}

impl<
        K: Serialize + for<'a> Deserialize<'a> + Clone,
        V: Serialize + for<'a> Deserialize<'a> + Default,
    > TypedSled<K, V>
{
    pub fn new(db: Db) -> Self {
        Self {
            db,
            _marker: PhantomData,
        }
    }

    pub fn insert(&self, key: K, value: V) {
        let serialized_key = into_byte_vec(&key);
        let serialized_value = into_byte_vec(&value);
        self.db.insert(serialized_key, serialized_value).unwrap();
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let key = into_byte_vec(key);
        self.db
            .get(&key)
            .unwrap()
            .map(|raw_val| from_ivec(&raw_val))
    }

    pub fn entry(&self, key: &K) -> Option<(K, V)> {
        Self::get(&self, key).map(|deserialized_val| (key.clone(), deserialized_val))
    }

    pub fn remove(&self, key: &K) {
        self.db.remove(into_byte_vec(key)).unwrap();
    }

    pub fn flush(&self) -> Result<usize, Error> {
        self.db.flush()
    }

    pub fn contains(&self, key: &K) -> bool {
        self.db.contains_key(into_byte_vec(key)).unwrap()
    }
}
