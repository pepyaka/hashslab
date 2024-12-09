use core::{fmt, iter::FusedIterator};

use super::iter::{IntoIter, IterFull, IterFullMut};

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
impl<K, V> Clone for Values<'_, K, V> {
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
pub struct ValuesMut<'a, K, V> {
    iter_full_mut: IterFullMut<'a, K, V>,
}

impl<'a, K, V> ValuesMut<'a, K, V> {
    pub fn new(iter_full_mut: IterFullMut<'a, K, V>) -> Self {
        Self { iter_full_mut }
    }
}

impl<K, V: fmt::Debug> fmt::Debug for ValuesMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValuesMut")
            .field("remaining", &self.len())
            .finish()
    }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_full_mut.next().map(|(_, _, value)| value)
    }
}

impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V> {
    fn len(&self) -> usize {
        self.iter_full_mut.len()
    }
}

impl<K, V> FusedIterator for ValuesMut<'_, K, V> {}

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
