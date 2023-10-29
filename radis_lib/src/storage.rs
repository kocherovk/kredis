pub trait Storage<K, V> {
    fn get(&self, key: &K) -> Option<V>;
    fn set(&self, key: K, val: V);
    fn has(&self, key: &K) -> bool;
    fn remove(&self, key: &K);
    fn clear(&self);
}
