use core::{fmt, iter::FusedIterator};

use hashbrown::hash_table;
use slab::Slab;

use crate::{KeyData, ValueData};

use super::HashSlabMap;

/// An iterator over the full entries of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::iter_full`] method.
/// See its documentation for more.
pub struct IterFull<'a, K, V> {
    iter: hash_table::Iter<'a, KeyData<K>>,
    slab: &'a Slab<ValueData<V>>,
}

impl<'a, K, V> IterFull<'a, K, V> {
    pub(super) fn new(
        iter: hash_table::Iter<'a, KeyData<K>>,
        slab: &'a Slab<ValueData<V>>,
    ) -> Self {
        Self { iter, slab }
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<K, V> Clone for IterFull<'_, K, V> {
    fn clone(&self) -> Self {
        IterFull {
            iter: self.iter.clone(),
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
        self.iter.next().map(|KeyData { key, index, .. }| {
            let ValueData { value, .. } = &self.slab[*index];
            (*index, key, value)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K, V> ExactSizeIterator for IterFull<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
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
    pub fn new(iter_full: IterFull<'a, K, V>) -> Self {
        Self { iter_full }
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<K, V> Clone for Iter<'_, K, V> {
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter_full.size_hint()
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
pub struct IterFullMut<'a, K, V> {
    iter: hash_table::Iter<'a, KeyData<K>>,
    slab: &'a mut Slab<ValueData<V>>,
}

impl<'a, K, V> IterFullMut<'a, K, V> {
    pub(super) fn new(
        iter: hash_table::Iter<'a, KeyData<K>>,
        slab: &'a mut Slab<ValueData<V>>,
    ) -> Self {
        Self { iter, slab }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IterFullMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IterFullMut")
            .field("remaining", &self.len())
            .finish()
    }
}

impl<'a, K, V> Iterator for IterFullMut<'a, K, V> {
    type Item = (usize, &'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        let KeyData { key, index, .. } = self.iter.next()?;
        self.slab.get_mut(*index).map(|ValueData { value, .. }| {
            // We are using raw pointers to access the value in the Slab
            // and ensuring that no other reference to the Slab is active
            let value = unsafe { &mut *(value as *mut V) };
            (*index, key, value)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K, V> ExactSizeIterator for IterFullMut<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// A mutable iterator over entry pairs of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::iter_mut`] method.
/// See its documentation for more.
pub struct IterMut<'a, K, V> {
    iter_full_mut: IterFullMut<'a, K, V>,
}

impl<'a, K, V> IterMut<'a, K, V> {
    pub fn new(iter_full_mut: IterFullMut<'a, K, V>) -> Self {
        Self { iter_full_mut }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IterMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IterMut")
            .field("remaining", &self.iter_full_mut.len())
            .finish()
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_full_mut.next().map(|(_, k, v)| (k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter_full_mut.size_hint()
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
    fn len(&self) -> usize {
        self.iter_full_mut.len()
    }
}

/// An owning iterator over the index-key-value triples of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::into_full_iter`] method
/// (provided by the [`IntoIterator`] trait). See its documentation for more.
pub struct IntoFullIter<K, V> {
    into_iter: hash_table::IntoIter<KeyData<K>>,
    slab: Slab<ValueData<V>>,
}

impl<K, V> IntoFullIter<K, V> {
    pub(super) fn new(
        into_iter: hash_table::IntoIter<KeyData<K>>,
        slab: Slab<ValueData<V>>,
    ) -> Self {
        Self { into_iter, slab }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IntoFullIter<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoFullIter")
            .field("remaining", &self.into_iter.len())
            .finish()
    }
}

impl<K, V> Iterator for IntoFullIter<K, V> {
    type Item = (usize, K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.into_iter.next().map(|KeyData { index, key, .. }| {
            let ValueData { value, .. } = self.slab.remove(index);
            (index, key, value)
        })
    }
}

impl<K, V> ExactSizeIterator for IntoFullIter<K, V> {
    fn len(&self) -> usize {
        self.into_iter.len()
    }
}

impl<K, V> FusedIterator for IntoFullIter<K, V> {}

/// An owning iterator over the entries of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::into_iter`] method
/// (provided by the [`IntoIterator`] trait). See its documentation for more.
pub struct IntoIter<K, V> {
    into_full_iter: IntoFullIter<K, V>,
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IntoIter<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoIter")
            .field("remaining", &self.into_full_iter.len())
            .finish()
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.into_full_iter
            .next()
            .map(|(_, key, value)| (key, value))
    }
}

impl<K, V, S> IntoIterator for HashSlabMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            into_full_iter: self.into_full_iter(),
        }
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.into_full_iter.len()
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
