use radis_lib::storage::Storage;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Mutex, MutexGuard};

pub struct HashMapStorage<K, V> {
    data: Mutex<HashMap<K, V>>,
}

impl<K: Hash, V> HashMapStorage<K, V> {
    pub fn new() -> HashMapStorage<K, V> {
        HashMapStorage {
            data: Mutex::new(HashMap::new()),
        }
    }

    fn storage(&self) -> MutexGuard<HashMap<K, V>> {
        self.data.lock().unwrap()
    }
}

impl<K, V> Storage<K, V> for HashMapStorage<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        self.storage().get(key).cloned()
    }

    fn set(&self, key: K, val: V) {
        self.storage().insert(key, val);
    }

    fn has(&self, key: &K) -> bool {
        self.storage().contains_key(key)
    }

    fn remove(&self, key: &K) {
        self.storage().remove(key);
    }

    fn clear(&self) {
        self.storage().clear();
    }
}
