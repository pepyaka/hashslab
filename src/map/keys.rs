use core::fmt;
use std::iter::FusedIterator;

use hashbrown::hash_map;

use crate::KeyEntry;

/// An iterator over the index-key pairs of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::full_keys`] method.
/// See its documentation for more.
pub struct FullKeys<'a, K, V> {
    pub(super) keys: hash_map::Keys<'a, KeyEntry<K>, V>,
}

impl<'a, K, V> FullKeys<'a, K, V> {
    pub(crate) fn new(keys: hash_map::Keys<'a, KeyEntry<K>, V>) -> Self {
        Self { keys }
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<'a, K, V> Clone for FullKeys<'a, K, V> {
    fn clone(&self) -> Self {
        FullKeys {
            keys: self.keys.clone(),
        }
    }
}

impl<K: fmt::Debug, V> fmt::Debug for FullKeys<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V> Iterator for FullKeys<'a, K, V> {
    type Item = (usize, &'a K);

    fn next(&mut self) -> Option<Self::Item> {
        self.keys
            .next()
            .map(|KeyEntry { index, key, .. }| (*index, key))
    }
}

impl<K, V> ExactSizeIterator for FullKeys<'_, K, V> {
    fn len(&self) -> usize {
        self.keys.len()
    }
}

impl<K, V> FusedIterator for FullKeys<'_, K, V> {}

/// An iterator over the keys of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::keys`] method.
/// See its documentation for more.
pub struct Keys<'a, K, V> {
    pub(super) full_keys: FullKeys<'a, K, V>,
}

impl<'a, K, V> Keys<'a, K, V> {
    pub fn new(full_keys: FullKeys<'a, K, V>) -> Self {
        Self { full_keys }
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<'a, K, V> Clone for Keys<'a, K, V> {
    fn clone(&self) -> Self {
        Keys {
            full_keys: self.full_keys.clone(),
        }
    }
}

impl<K: fmt::Debug, V> fmt::Debug for Keys<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.full_keys.next().map(|(_, k)| k)
    }
}

impl<K, V> ExactSizeIterator for Keys<'_, K, V> {
    fn len(&self) -> usize {
        self.full_keys.len()
    }
}

impl<K, V> FusedIterator for Keys<'_, K, V> {}

/// An owning iterator over the keys of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::into_keys`] method.
/// See its documentation for more.
pub struct IntoKeys<K, V> {
    into_keys: hash_map::IntoKeys<KeyEntry<K>, V>,
}

impl<K, V> IntoKeys<K, V> {
    pub(crate) fn new(into_keys: hash_map::IntoKeys<KeyEntry<K>, V>) -> Self {
        Self { into_keys }
    }
}

impl<K: fmt::Debug, V> fmt::Debug for IntoKeys<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoKeys")
            .field("remaining", &self.len())
            .finish()
    }
}

impl<K, V> Iterator for IntoKeys<K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.into_keys.next().map(|KeyEntry { key, .. }| key)
    }
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V> {
    fn len(&self) -> usize {
        self.into_keys.len()
    }
}

impl<K, V> FusedIterator for IntoKeys<K, V> {}

/// An iterator over the indexes ([`usize`] keys) of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::indices`] method.
/// See its documentation for more.
#[derive(Debug, Clone)]
pub struct Indices<'a, K, V> {
    pub(super) keys: hash_map::Keys<'a, KeyEntry<K>, V>,
}

impl<'a, K, V> Indices<'a, K, V> {
    pub(crate) fn new(keys: hash_map::Keys<'a, KeyEntry<K>, V>) -> Self {
        Self { keys }
    }
}

impl<'a, K, V> Iterator for Indices<'a, K, V> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next().map(|KeyEntry { index, .. }| *index)
    }
}

impl<K, V> ExactSizeIterator for Indices<'_, K, V> {
    fn len(&self) -> usize {
        self.keys.len()
    }
}

impl<K, V> FusedIterator for Indices<'_, K, V> {}
