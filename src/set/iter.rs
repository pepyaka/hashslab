use core::{
    fmt,
    hash::{BuildHasher, Hash},
    iter::{Chain, FusedIterator},
};

use crate::map;

use super::HashSlabSet;

impl<'a, T, S> IntoIterator for &'a HashSlabSet<T, S> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S> IntoIterator for HashSlabSet<T, S> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.map.into_keys())
    }
}

/// An iterator over the items of an [`HashSlabSet`].
///
/// This `struct` is created by the [`HashSlabSet::iter`] method.
/// See its documentation for more.
pub struct Iter<'a, T> {
    keys: map::Keys<'a, T>,
}

impl<'a, T> Iter<'a, T> {
    pub fn new(keys: map::Keys<'a, T>) -> Self {
        Self { keys }
    }
}

impl<T: fmt::Debug> fmt::Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<T> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        Iter {
            keys: self.keys.clone(),
        }
    }
}

impl<K> Default for Iter<'_, K> {
    fn default() -> Self {
        Iter {
            keys: Default::default(),
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next()
    }
}

// impl<T> DoubleEndedIterator for Iter<'_, T> {
//     double_ended_iterator_methods!(Bucket::key_ref);
// }

impl<T> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        self.keys.len()
    }
}

impl<T> FusedIterator for Iter<'_, T> {}

/// An iterator over the index-value entries of an [`HashSlabSet`].
///
/// This `struct` is created by the [`HashSlabSet::iter_full`] method.
/// See its documentation for more.
pub struct IterFull<'a, K> {
    full_keys: map::FullKeys<'a, K>,
}

impl<'a, K> IterFull<'a, K> {
    pub fn new(full_keys: map::FullKeys<'a, K>) -> Self {
        Self { full_keys }
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<K> Clone for IterFull<'_, K> {
    fn clone(&self) -> Self {
        IterFull {
            full_keys: self.full_keys.clone(),
        }
    }
}

impl<K: fmt::Debug> fmt::Debug for IterFull<'_, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.clone()).finish()
    }
}

impl<'a, K> Iterator for IterFull<'a, K> {
    type Item = (usize, &'a K);

    fn next(&mut self) -> Option<Self::Item> {
        self.full_keys.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.full_keys.size_hint()
    }
}

impl<K> ExactSizeIterator for IterFull<'_, K> {
    fn len(&self) -> usize {
        self.full_keys.len()
    }
}

impl<K> FusedIterator for IterFull<'_, K> {}

/// An owning iterator over the items of an [`HashSlabSet`].
///
/// This `struct` is created by the [`HashSlabSet::into_iter`] method
/// (provided by the [`IntoIterator`] trait). See its documentation for more.
pub struct IntoIter<T> {
    into_keys: map::IntoKeys<T>,
}

impl<T> IntoIter<T> {
    pub fn new(into_keys: map::IntoKeys<T>) -> Self {
        Self { into_keys }
    }
}

impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoIter")
            .field("remaining", &self.len())
            .finish()
    }
}

impl<T> Default for IntoIter<T> {
    fn default() -> Self {
        Self {
            into_keys: Default::default(),
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.into_keys.next()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.into_keys.len()
    }
}

impl<T> FusedIterator for IntoIter<T> {}

/// A draining iterator over the items of an [`HashSlabSet`].
///
/// This `struct` is created by the [`HashSlabSet::drain`] method.
/// See its documentation for more.
pub struct Drain<'a, T> {
    drain: map::Drain<'a, T, ()>,
}

impl<'a, T> Drain<'a, T> {
    pub fn new(drain: map::Drain<'a, T, ()>) -> Self {
        Self { drain }
    }
}

impl<T> Iterator for Drain<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next().map(|(x, _)| x)
    }
}

impl<T> ExactSizeIterator for Drain<'_, T> {
    fn len(&self) -> usize {
        self.drain.len()
    }
}

impl<T> FusedIterator for Drain<'_, T> {}

impl<T: fmt::Debug> fmt::Debug for Drain<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Drain")
            .field("remaining", &self.len())
            .finish()
    }
}

/// A lazy iterator producing elements in the difference of [`HashSlabSet`]s.
///
/// This `struct` is created by the [`HashSlabSet::difference`] method.
/// See its documentation for more.
pub struct Difference<'a, T, S> {
    iter: Iter<'a, T>,
    other: &'a HashSlabSet<T, S>,
}

impl<'a, T, S> Difference<'a, T, S> {
    pub fn new(iter: Iter<'a, T>, other: &'a HashSlabSet<T, S>) -> Self {
        Self { iter, other }
    }
}

impl<T, S> Clone for Difference<'_, T, S> {
    fn clone(&self) -> Self {
        Difference {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<T, S> fmt::Debug for Difference<'_, T, S>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, T, S> Iterator for Difference<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.by_ref().find(|&item| !self.other.contains(item))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

impl<T, S> FusedIterator for Difference<'_, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
}

/// A lazy iterator producing elements in the intersection of [`HashSlabSet`]s.
///
/// This `struct` is created by the [`HashSlabSet::intersection`] method.
/// See its documentation for more.
pub struct Intersection<'a, T, S> {
    iter: Iter<'a, T>,
    other: &'a HashSlabSet<T, S>,
}

impl<'a, T, S> Intersection<'a, T, S> {
    pub(super) fn new<S1>(set: &'a HashSlabSet<T, S1>, other: &'a HashSlabSet<T, S>) -> Self {
        Self {
            iter: set.iter(),
            other,
        }
    }
}

impl<'a, T, S> Iterator for Intersection<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.by_ref().find(|&item| self.other.contains(item))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

impl<T, S> FusedIterator for Intersection<'_, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
}

impl<T, S> Clone for Intersection<'_, T, S> {
    fn clone(&self) -> Self {
        Intersection {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<T, S> fmt::Debug for Intersection<'_, T, S>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// A lazy iterator producing elements in the symmetric difference of [`HashSlabSet`]s.
///
/// This `struct` is created by the [`HashSlabSet::symmetric_difference`] method.
/// See its documentation for more.
pub struct SymmetricDifference<'a, T, S1, S2> {
    chain: Chain<Difference<'a, T, S2>, Difference<'a, T, S1>>,
}

impl<'a, T, S1, S2> SymmetricDifference<'a, T, S1, S2>
where
    T: Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
{
    pub(super) fn new(set1: &'a HashSlabSet<T, S1>, set2: &'a HashSlabSet<T, S2>) -> Self {
        let diff1 = set1.difference(set2);
        let diff2 = set2.difference(set1);
        Self {
            chain: diff1.chain(diff2),
        }
    }
}

impl<'a, T, S1, S2> Iterator for SymmetricDifference<'a, T, S1, S2>
where
    T: Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.chain.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chain.size_hint()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.chain.fold(init, f)
    }
}

impl<T, S1, S2> FusedIterator for SymmetricDifference<'_, T, S1, S2>
where
    T: Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
{
}

impl<T, S1, S2> Clone for SymmetricDifference<'_, T, S1, S2> {
    fn clone(&self) -> Self {
        SymmetricDifference {
            chain: self.chain.clone(),
        }
    }
}

impl<T, S1, S2> fmt::Debug for SymmetricDifference<'_, T, S1, S2>
where
    T: fmt::Debug + Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// A lazy iterator producing elements in the union of [`HashSlabSet`]s.
///
/// This `struct` is created by the [`HashSlabSet::union`] method.
/// See its documentation for more.
pub struct Union<'a, T, S> {
    chain: Chain<Iter<'a, T>, Difference<'a, T, S>>,
}

impl<'a, T, S> Union<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    pub(super) fn new<S2>(set1: &'a HashSlabSet<T, S>, set2: &'a HashSlabSet<T, S2>) -> Self
    where
        S2: BuildHasher,
    {
        Self {
            chain: set1.iter().chain(set2.difference(set1)),
        }
    }
}

impl<'a, T, S> Iterator for Union<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.chain.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chain.size_hint()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.chain.fold(init, f)
    }
}

impl<T, S> FusedIterator for Union<'_, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
}

impl<T, S> Clone for Union<'_, T, S> {
    fn clone(&self) -> Self {
        Union {
            chain: self.chain.clone(),
        }
    }
}

impl<T, S> fmt::Debug for Union<'_, T, S>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
