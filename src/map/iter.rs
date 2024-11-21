use core::fmt;
use std::{
    hash::{BuildHasher, Hash},
    iter::FusedIterator,
};

use hashbrown::{hash_set, HashSet};
use slab::Slab;

use crate::{DirectAssignmentHasherBuilder, KeyEntry, RawHash, ValueEntry};

use super::HashSlabMap;

/// An iterator over the full entries of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::iter_full`] method.
/// See its documentation for more.
pub struct IterFull<'a, K, V> {
    pub(super) hs_iter: hash_set::Iter<'a, KeyEntry<K>>,
    pub(super) slab: &'a Slab<ValueEntry<V>>,
}

impl<'a, K, V> IterFull<'a, K, V> {
    pub(super) fn new(
        hs_iter: hash_set::Iter<'a, KeyEntry<K>>,
        slab: &'a Slab<ValueEntry<V>>,
    ) -> Self {
        Self { hs_iter, slab }
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<'a, K, V> Clone for IterFull<'a, K, V> {
    fn clone(&self) -> Self {
        IterFull {
            hs_iter: self.hs_iter.clone(),
            slab: self.slab,
        }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IterFull<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.clone().map(|(i, k, v)| (i, (k, v))))
            .finish()
    }
}

impl<'a, K, V> Iterator for IterFull<'a, K, V> {
    type Item = (usize, &'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let KeyEntry { key, index } = self.hs_iter.next()?;
        self.slab
            .get(*index)
            .map(|ValueEntry { data, .. }| (*index, key, data))
    }
}

impl<K, V> ExactSizeIterator for IterFull<'_, K, V> {
    fn len(&self) -> usize {
        self.hs_iter.len()
    }
}

impl<K, V> FusedIterator for IterFull<'_, K, V> {}

/// An iterator over the entries of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::iter`] method.
/// See its documentation for more.
pub struct Iter<'a, K, V> {
    iter_full: IterFull<'a, K, V>,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub(super) fn new(
        hs_iter: hash_set::Iter<'a, KeyEntry<K>>,
        slab: &'a Slab<ValueEntry<V>>,
    ) -> Self {
        Self {
            iter_full: IterFull { hs_iter, slab },
        }
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<'a, K, V> Clone for Iter<'a, K, V> {
    fn clone(&self) -> Self {
        Iter {
            iter_full: self.iter_full.clone(),
        }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Iter<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.clone()).finish()
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_full.next().map(|(_, k, v)| (k, v))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    fn len(&self) -> usize {
        self.iter_full.len()
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

/// A mutable iterator over entry triples of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::iter_full_mut`] method.
/// See its documentation for more.
pub struct IterFullMut<'a, K, V, S> {
    hs: &'a HashSet<KeyEntry<K>, DirectAssignmentHasherBuilder<S>>,
    slab_iter_mut: slab::IterMut<'a, ValueEntry<V>>,
}

impl<'a, K, V, S> IterFullMut<'a, K, V, S> {
    pub(super) fn new(
        hs: &'a HashSet<KeyEntry<K>, DirectAssignmentHasherBuilder<S>>,
        slab_iter_mut: slab::IterMut<'a, ValueEntry<V>>,
    ) -> Self {
        Self { hs, slab_iter_mut }
    }

    fn len(&self) -> usize {
        self.slab_iter_mut.len()
    }
}

impl<K: fmt::Debug, V: fmt::Debug, S> fmt::Debug for IterFullMut<'_, K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IterFullMut")
            .field("remaining", &self.len())
            .finish()
    }
}

impl<'a, K, V, S> Iterator for IterFullMut<'a, K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    type Item = (usize, &'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        let (index, ValueEntry { hash_value, data }) = self.slab_iter_mut.next()?;
        self.hs
            .get(&RawHash {
                index,
                value: *hash_value,
            })
            .map(|KeyEntry { index, key }| (*index, key, data))
    }
}

/// A mutable iterator over entry pairs of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::iter_mut`] method.
/// See its documentation for more.
pub struct IterMut<'a, K, V, S> {
    iter_full_mut: IterFullMut<'a, K, V, S>,
}

impl<'a, K, V, S> IterMut<'a, K, V, S> {
    pub fn new(iter_full_mut: IterFullMut<'a, K, V, S>) -> Self {
        Self { iter_full_mut }
    }
}

impl<K: fmt::Debug, V: fmt::Debug, S> fmt::Debug for IterMut<'_, K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IterMut")
            .field("remaining", &self.iter_full_mut.len())
            .finish()
    }
}

impl<'a, K, V, S> Iterator for IterMut<'a, K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_full_mut.next().map(|(_, k, v)| (k, v))
    }
}

/// An owning iterator over the entries of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::into_iter`] method
/// (provided by the [`IntoIterator`] trait). See its documentation for more.
pub struct IntoIter<K, V> {
    hs_into_iter: hash_set::IntoIter<KeyEntry<K>>,
    slab: Slab<ValueEntry<V>>,
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IntoIter<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoIter")
            .field("remaining", &self.hs_into_iter.len())
            .finish()
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.hs_into_iter
            .next()
            .map(|KeyEntry { index, key }| (key, self.slab.remove(index).data))
    }
}

impl<K, V, S> IntoIterator for HashSlabMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            hs_into_iter: self.hs.into_iter(),
            slab: self.slab,
        }
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.slab.len()
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> {}

impl<'a, K, V, S> IntoIterator for &'a HashSlabMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
