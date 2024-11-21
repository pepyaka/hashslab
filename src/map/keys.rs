use core::fmt;
use std::iter::FusedIterator;

use hashbrown::hash_set;

use crate::KeyEntry;

/// An iterator over the index-key pairs of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::full_keys`] method.
/// See its documentation for more.
pub struct FullKeys<'a, K> {
    pub(super) hs_iter: hash_set::Iter<'a, KeyEntry<K>>,
}

impl<'a, K> FullKeys<'a, K> {
    pub(super) fn new(hs_iter: hash_set::Iter<'a, KeyEntry<K>>) -> Self {
        Self { hs_iter }
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<'a, K> Clone for FullKeys<'a, K> {
    fn clone(&self) -> Self {
        FullKeys {
            hs_iter: self.hs_iter.clone(),
        }
    }
}

impl<K: fmt::Debug> fmt::Debug for FullKeys<'_, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K> Iterator for FullKeys<'a, K> {
    type Item = (usize, &'a K);

    fn next(&mut self) -> Option<Self::Item> {
        self.hs_iter
            .next()
            .map(|KeyEntry { index, key }| (*index, key))
    }
}

impl<K> ExactSizeIterator for FullKeys<'_, K> {
    fn len(&self) -> usize {
        self.hs_iter.len()
    }
}

impl<K> FusedIterator for FullKeys<'_, K> {}

/// An iterator over the keys of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::keys`] method.
/// See its documentation for more.
pub struct Keys<'a, K> {
    pub(super) hs_iter: hash_set::Iter<'a, KeyEntry<K>>,
}

impl<'a, K> Keys<'a, K> {
    pub(super) fn new(hs_iter: hash_set::Iter<'a, KeyEntry<K>>) -> Self {
        Self { hs_iter }
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<'a, K> Clone for Keys<'a, K> {
    fn clone(&self) -> Self {
        Keys {
            hs_iter: self.hs_iter.clone(),
        }
    }
}

impl<K: fmt::Debug> fmt::Debug for Keys<'_, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K> Iterator for Keys<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.hs_iter.next().map(|KeyEntry { key, .. }| key)
    }
}

impl<K> ExactSizeIterator for Keys<'_, K> {
    fn len(&self) -> usize {
        self.hs_iter.len()
    }
}

impl<K> FusedIterator for Keys<'_, K> {}

/// An owning iterator over the keys of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::into_keys`] method.
/// See its documentation for more.
pub struct IntoKeys<K> {
    hs_into_iter: hash_set::IntoIter<KeyEntry<K>>,
}

impl<K> IntoKeys<K> {
    pub(super) fn new(hs_into_iter: hash_set::IntoIter<KeyEntry<K>>) -> Self {
        Self { hs_into_iter }
    }
}

impl<K: fmt::Debug> fmt::Debug for IntoKeys<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoKeys")
            .field("remaining", &self.len())
            .finish()
    }
}

impl<K> Iterator for IntoKeys<K> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.hs_into_iter.next().map(|KeyEntry { key, .. }| key)
    }
}

impl<K> ExactSizeIterator for IntoKeys<K> {
    fn len(&self) -> usize {
        self.hs_into_iter.len()
    }
}

impl<K> FusedIterator for IntoKeys<K> {}

/// An iterator over the indexes ([`usize`] keys) of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::indices`] method.
/// See its documentation for more.
#[derive(Debug, Clone)]
pub struct Indices<'a, K> {
    pub(super) hs_iter: hash_set::Iter<'a, KeyEntry<K>>,
}

impl<'a, K> Indices<'a, K> {
    pub(super) fn new(hs_iter: hash_set::Iter<'a, KeyEntry<K>>) -> Self {
        Self { hs_iter }
    }
}

impl<'a, K> Iterator for Indices<'a, K> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.hs_iter.next().map(|KeyEntry { index, .. }| *index)
    }
}

impl<K> ExactSizeIterator for Indices<'_, K> {
    fn len(&self) -> usize {
        self.hs_iter.len()
    }
}

impl<K> FusedIterator for Indices<'_, K> {}
