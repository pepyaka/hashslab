use core::fmt;
use std::iter::FusedIterator;

use crate::ValueEntry;

use super::iter::{IntoIter, IterFull};

/// An iterator over the values of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::values`] method.
/// See its documentation for more.
pub struct Values<'a, K, V> {
    iter_full: IterFull<'a, K, V>,
}

impl<'a, K, V> Values<'a, K, V> {
    pub fn new(iter_full: IterFull<'a, K, V>) -> Self {
        Self { iter_full }
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<'a, K, V> Clone for Values<'a, K, V> {
    fn clone(&self) -> Self {
        Values {
            iter_full: self.iter_full.clone(),
        }
    }
}

impl<K, V: fmt::Debug> fmt::Debug for Values<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_full.next().map(|(_, _, v)| v)
    }
}

impl<K, V> ExactSizeIterator for Values<'_, K, V> {
    fn len(&self) -> usize {
        self.iter_full.len()
    }
}

impl<K, V> FusedIterator for Values<'_, K, V> {}

/// A mutable iterator over the values of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::values_mut`] method.
/// See its documentation for more.
pub struct ValuesMut<'a, V> {
    slab_iter_mut: slab::IterMut<'a, ValueEntry<V>>,
}

impl<'a, V> ValuesMut<'a, V> {
    pub(super) fn new(slab_iter_mut: slab::IterMut<'a, ValueEntry<V>>) -> Self {
        Self { slab_iter_mut }
    }
}

impl<V: fmt::Debug> fmt::Debug for ValuesMut<'_, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValuesMut")
            .field("remaining", &self.slab_iter_mut.len())
            .finish()
    }
}

impl<'a, V> Iterator for ValuesMut<'a, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.slab_iter_mut
            .next()
            .map(|(_, ValueEntry { data, .. })| data)
    }
}

impl<V> ExactSizeIterator for ValuesMut<'_, V> {
    fn len(&self) -> usize {
        self.slab_iter_mut.len()
    }
}

impl<V> FusedIterator for ValuesMut<'_, V> {}

/// An owning iterator over the values of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::into_values`] method.
/// See its documentation for more.
pub struct IntoValues<K, V> {
    into_iter: IntoIter<K, V>,
}

impl<K, V> IntoValues<K, V> {
    pub fn new(into_iter: IntoIter<K, V>) -> Self {
        Self { into_iter }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IntoValues<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoValues")
            .field("remaining", &self.into_iter.len())
            .finish()
    }
}

impl<K, V> Iterator for IntoValues<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.into_iter.next().map(|(_, v)| v)
    }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V> {
    fn len(&self) -> usize {
        self.into_iter.len()
    }
}

impl<K, V> FusedIterator for IntoValues<K, V> {}
