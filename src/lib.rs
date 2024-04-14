use std::{
    hash::Hash,
    time::{Duration, Instant},
};

use ahash::AHashMap;

#[derive(Debug, Clone)]
pub struct ExpiryValue<T> {
    pub inserted: Instant,
    pub ttl: Duration,
    pub value: T,
}

impl<T> ExpiryValue<T> {
    fn expired(&self) -> bool {
        self.inserted.elapsed() > self.ttl
    }
}

type ExpiringMapInner<K, V> = AHashMap<K, ExpiryValue<V>>;
pub type ExpiringSet<K> = ExpiringMap<K, ()>;

#[derive(Debug)]
pub struct ExpiringMap<K, V> {
    last_size: usize,
    inner: ExpiringMapInner<K, V>,
}

impl<K: PartialEq + Eq + Hash, V> ExpiringMap<K, V> {
    pub fn vacuum(&mut self) {
        // keep all the items in the set where it has been
        // less than ttl since they were added
        self.inner
            .retain(|_, expiry| expiry.inserted.elapsed() < expiry.ttl);
        // make the map as small as possible to decrease memory usage
        self.inner.shrink_to_fit();
    }

    pub fn vacuum_if_needed(&mut self) {
        if (self.last_size * 3) / 2 < self.inner.len() {
            self.vacuum();
            self.last_size = self.inner.len();
        }
    }

    pub fn insert(&mut self, key: K, value: V, ttl: Duration) {
        self.vacuum_if_needed();
        let entry = ExpiryValue {
            inserted: Instant::now(),
            ttl,
            value,
        };
        self.inner.insert(key, entry);
    }

    pub fn get_meta(&self, key: &K) -> Option<&ExpiryValue<V>> {
        let val = self.inner.get(key);
        if val.is_some_and(ExpiryValue::expired) {
            return None;
        }
        val
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some(v) = self.get_meta(key) {
            Some(&v.value)
        } else {
            None
        }
    }

    pub fn last_size(&self) -> usize {
        self.last_size
    }
}

impl<K: PartialEq + Eq + Hash> ExpiringMap<K, ()> {
    pub fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
}
