use core::{fmt, iter::FusedIterator};

use hashbrown::hash_table;

use crate::KeyData;

/// An iterator over the index-key pairs of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::full_keys`] method.
/// See its documentation for more.
pub struct FullKeys<'a, K> {
    iter: hash_table::Iter<'a, KeyData<K>>,
}

impl<'a, K> FullKeys<'a, K> {
    pub(super) fn new(iter: hash_table::Iter<'a, KeyData<K>>) -> Self {
        Self { iter }
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<K> Clone for FullKeys<'_, K> {
    fn clone(&self) -> Self {
        FullKeys {
            iter: self.iter.clone(),
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
        self.iter
            .next()
            .map(|KeyData { index, key, .. }| (*index, key))
    }
}

impl<K> ExactSizeIterator for FullKeys<'_, K> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K> FusedIterator for FullKeys<'_, K> {}

/// An iterator over the keys of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::keys`] method.
/// See its documentation for more.
pub struct Keys<'a, K> {
    pub(super) full_keys: FullKeys<'a, K>,
}

impl<'a, K> Keys<'a, K> {
    pub fn new(full_keys: FullKeys<'a, K>) -> Self {
        Self { full_keys }
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<K> Clone for Keys<'_, K> {
    fn clone(&self) -> Self {
        Keys {
            full_keys: self.full_keys.clone(),
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
        self.full_keys.next().map(|(_, k)| k)
    }
}

impl<K> ExactSizeIterator for Keys<'_, K> {
    fn len(&self) -> usize {
        self.full_keys.len()
    }
}

impl<K> FusedIterator for Keys<'_, K> {}

/// An owning iterator over the keys of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::into_keys`] method.
/// See its documentation for more.
pub struct IntoKeys<K> {
    into_iter: hash_table::IntoIter<KeyData<K>>,
}

impl<K> IntoKeys<K> {
    pub(super) fn new(into_iter: hash_table::IntoIter<KeyData<K>>) -> Self {
        Self { into_iter }
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
        self.into_iter.next().map(|KeyData { key, .. }| key)
    }
}

impl<K> ExactSizeIterator for IntoKeys<K> {
    fn len(&self) -> usize {
        self.into_iter.len()
    }
}

impl<K> FusedIterator for IntoKeys<K> {}

/// An iterator over the indexes ([`usize`] keys) of an [`HashSlabMap`].
///
/// This `struct` is created by the [`HashSlabMap::indices`] method.
/// See its documentation for more.
#[derive(Debug, Clone)]
pub struct Indices<'a, K> {
    iter: hash_table::Iter<'a, KeyData<K>>,
}

impl<'a, K> Indices<'a, K> {
    pub(super) fn new(iter: hash_table::Iter<'a, KeyData<K>>) -> Self {
        Self { iter }
    }
}

impl<K> Iterator for Indices<'_, K> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|KeyData { index, .. }| *index)
    }
}

impl<K> ExactSizeIterator for Indices<'_, K> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K> FusedIterator for Indices<'_, K> {}
