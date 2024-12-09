use core::{fmt, iter::FusedIterator};

use hashbrown::hash_table;
use slab::Slab;

use crate::{KeyData, ValueData};

/// A draining iterator over the index-key-value triples of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::drain`] method.
/// See its documentation for more.
pub struct DrainFull<'a, K, V> {
    drain: hash_table::Drain<'a, KeyData<K>>,
    slab: &'a mut Slab<ValueData<V>>,
}

impl<'a, K, V> DrainFull<'a, K, V> {
    pub(super) fn new(
        drain: hash_table::Drain<'a, KeyData<K>>,
        slab: &'a mut Slab<ValueData<V>>,
    ) -> Self {
        Self { drain, slab }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for DrainFull<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Drain")
            .field("remaining", &self.len())
            .finish()
    }
}

impl<K, V> Iterator for DrainFull<'_, K, V> {
    type Item = (usize, K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next().map(|KeyData { key, index, .. }| {
            let ValueData { value, .. } = self.slab.remove(index);
            (index, key, value)
        })
    }
}

impl<K, V> ExactSizeIterator for DrainFull<'_, K, V> {
    fn len(&self) -> usize {
        self.drain.len()
    }
}

impl<K, V> FusedIterator for DrainFull<'_, K, V> {}

/// A draining iterator over the key-value entries of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::drain`] method.
/// See its documentation for more.
pub struct Drain<'a, K, V> {
    pub(super) drain_full: DrainFull<'a, K, V>,
}

impl<'a, K, V> Drain<'a, K, V> {
    pub(super) fn new(drain_full: DrainFull<'a, K, V>) -> Self {
        Self { drain_full }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Drain<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DrainFull")
            .field("remaining", &self.len())
            .finish()
    }
}

impl<K, V> Iterator for Drain<'_, K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.drain_full.next().map(|(_, k, v)| (k, v))
    }
}

impl<K, V> ExactSizeIterator for Drain<'_, K, V> {
    fn len(&self) -> usize {
        self.drain_full.len()
    }
}

impl<K, V> FusedIterator for Drain<'_, K, V> {}
