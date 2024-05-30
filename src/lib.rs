//! [`ExpiringMap`] is a wrapper around [`AHashMap`] that allows the specification
//! of TTLs on entries. It does not support iteration.
//!
//! ```rust
//! use std::time::Duration;
//! use expiringmap::ExpiringMap;
//! let mut map = ExpiringMap::new();
//! map.insert("key", "value", Duration::from_millis(50));
//! std::thread::sleep(Duration::from_millis(60));
//! assert!(map.get(&"key").is_none());
//! ```
#![allow(clippy::must_use_candidate)]

use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
    time::{Duration, Instant},
};

#[cfg(test)]
mod test;

type ExpiringMapInner<K, V> = HashMap<K, ExpiryValue<V>>;

/// A struct to contain a value and its expiry information
#[derive(Debug, Clone)]
pub struct ExpiryValue<T> {
    inserted: Instant,
    ttl: Duration,
    value: T,
}

impl<T> Deref for ExpiryValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> ExpiryValue<T> {
    /// When this value was inserted
    pub const fn inserted(&self) -> Instant {
        self.inserted
    }

    /// How long this entry will live
    pub const fn ttl(&self) -> Duration {
        self.ttl
    }

    /// How long is left before this entry is deleted
    pub fn remaining(&self) -> Duration {
        self.ttl.saturating_sub(self.inserted.elapsed())
    }

    /// Take ownership of the internal value
    pub fn value(self) -> T {
        self.value
    }

    /// If this entry is expired and should be deleted
    pub fn expired(&self) -> bool {
        self.remaining().is_zero()
    }

    /// if this entry has not expired, and should be kept
    pub fn not_expired(&self) -> bool {
        !self.expired()
    }
}

/// A wrapper around [`AHashMap`] which adds TTLs
#[derive(Debug)]
pub struct ExpiringMap<K, V> {
    last_size: usize,
    inner: ExpiringMapInner<K, V>,
}

#[derive(Debug)]
/// A set version of [`ExpiringMap`]. Sets `V` to [`()`](https://doc.rust-lang.org/stable/std/primitive.unit.html)
pub struct ExpiringSet<K>(ExpiringMap<K, ()>);

impl<K> Deref for ExpiringSet<K> {
    type Target = ExpiringMap<K, ()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K> DerefMut for ExpiringSet<K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K: PartialEq + Eq + Hash, V> ExpiringMap<K, V> {
    /// the minimum size to set `last_size` to so we don't go bananas with vacuums
    const MINIMUM_VACUUM_SIZE: usize = 8;

    /// Create a new [`ExpiringMap`]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Create a new [`ExpiringMap`] with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: ExpiringMapInner::with_capacity(capacity),
            last_size: Self::MINIMUM_VACUUM_SIZE,
        }
    }

    /// Shrinks the hashmap based on entries that should no longer be contained.
    /// This is O(n).
    pub fn vacuum(&mut self) {
        // keep all the items in the set where it has been
        // less than ttl since they were added
        self.inner.retain(|_, expiry| expiry.not_expired());
        if self.inner.len() > Self::MINIMUM_VACUUM_SIZE {
            self.last_size = self.inner.len();
        } else {
            self.last_size = Self::MINIMUM_VACUUM_SIZE;
        }
    }

    /// execute a vacuum if the map has grown by more than 1.5 times
    pub fn vacuum_if_needed(&mut self) {
        if (self.last_size * 3) / 2 < self.inner.len() {
            self.vacuum();
        }
    }

    /// If the value exists and has not expired, return its expiry data
    pub fn get_meta(&self, key: &K) -> Option<&ExpiryValue<V>> {
        let val = self.inner.get(key);
        if val.is_some_and(ExpiryValue::expired) {
            return None;
        }
        val
    }

    /// If the value exists and has not expired, return it
    pub fn get(&self, key: &K) -> Option<&V> {
        // get meta checks expiry for us
        self.get_meta(key).map(|v| &v.value)
    }

    /// If a key exists for this value, get both the key and value if it is not expired
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        if let Some((k, v)) = self.inner.get_key_value(key) {
            if v.expired() {
                None
            } else {
                Some((k, &v.value))
            }
        } else {
            None
        }
    }

    /// Get a mutable reference to the value pointed to by a key, if it is not expired
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(v) = self.inner.get_mut(key) {
            if v.expired() {
                None
            } else {
                Some(&mut v.value)
            }
        } else {
            None
        }
    }

    /// Insert a value into the map, returning the old value if it has not expired and existed
    pub fn insert(&mut self, key: K, value: V, ttl: Duration) -> Option<ExpiryValue<V>> {
        self.vacuum_if_needed();
        let entry = ExpiryValue {
            inserted: Instant::now(),
            ttl,
            value,
        };
        self.inner
            .insert(key, entry)
            .filter(ExpiryValue::not_expired)
    }

    /// If this key exists and is not expired, returns true
    pub fn contains_key(&self, key: &K) -> bool {
        self.get_meta(key).is_some_and(ExpiryValue::not_expired)
    }

    /// Remove an item from the map. If it exists and has not expired, return true
    pub fn remove(&mut self, key: &K) -> bool {
        self.inner
            .remove(key)
            .as_ref()
            .is_some_and(ExpiryValue::not_expired)
    }

    /// Return the size the map was last time it was vacuumed
    pub const fn last_size(&self) -> usize {
        self.last_size
    }

    /// Return the number of items within the map
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Return true if the [`Self::len`] is 0
    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }

    /// Return the capacity of the internal map
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Reserve at least a certain capacity on the internal map
    pub fn reserve(&mut self, addtional: usize) {
        self.inner.reserve(addtional);
    }

    /// Shrink the internal map to the minimum allowable size
    /// in accordance with the resize policy
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }

    /// Shrink the internal map to the minimum of the
    /// minimum allowable size and the `min_capacity`
    /// in accordance with the resize policy
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity);
    }
}

impl<K: PartialEq + Eq + Hash> ExpiringSet<K> {
    /// Create a new [`ExpiringSet`]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Create a new [`ExpiringSet`] with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self(ExpiringMap::with_capacity(capacity))
    }

    /// Returns true if the set contains this value
    pub fn insert(&mut self, key: K, ttl: Duration) -> bool {
        self.vacuum_if_needed();
        let entry = ExpiryValue {
            inserted: Instant::now(),
            ttl,
            value: (),
        };
        self.inner
            .insert(key, entry)
            .filter(ExpiryValue::not_expired)
            .is_some()
    }

    /// Returns true if the set contains this value
    pub fn contains(&self, key: &K) -> bool {
        // contains_key checks expiry for us
        self.0.contains_key(key)
    }

    /// If it exists and has not expired, remove and return the value at this key
    pub fn take(&mut self, key: &K) -> Option<K> {
        self.0
            .inner
            .remove_entry(key)
            .filter(|(_, v)| v.not_expired())
            .map(|v| v.0)
    }
}

impl<K: PartialEq + Eq + Hash, V> Default for ExpiringMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialEq + Eq + Hash> Default for ExpiringSet<K> {
    fn default() -> Self {
        Self::new()
    }
}
