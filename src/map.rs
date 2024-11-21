use std::{
    fmt,
    hash::{BuildHasher, Hash, Hasher, RandomState},
    isize, mem,
    ops::{Index, IndexMut},
};

use hashbrown::{Equivalent, HashSet};
use slab::Slab;

use crate::TryReserveError;

use super::{DirectAssignmentHasherBuilder, KeyEntry, Query, RawHash, ValueEntry};

mod keys;
use keys::{FullKeys, Indices, IntoKeys, Keys};

mod values;
use values::{IntoValues, Values, ValuesMut};

mod iter;
use iter::{Iter, IterFull, IterFullMut, IterMut};

mod drain;
use drain::{Drain, DrainFull};

#[cfg(test)]
mod tests;

pub struct HashSlabMap<K, V, S = RandomState> {
    hs: HashSet<KeyEntry<K>, DirectAssignmentHasherBuilder<S>>,
    slab: Slab<ValueEntry<V>>,
}

impl<K, V> HashSlabMap<K, V> {
    /// Create a new map. (Does not allocate.)
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Create a new map with capacity for `n` entries. (Does not allocate if `n` is zero.)
    #[inline]
    pub fn with_capacity(n: usize) -> Self {
        Self::with_capacity_and_hasher(n, <_>::default())
    }
}

impl<K, V, S> HashSlabMap<K, V, S> {
    /// Create a new map with `hash_builder` and capacity for `n` entries.
    #[inline]
    pub fn with_capacity_and_hasher(n: usize, hash_builder: S) -> Self {
        let hasher_buider = DirectAssignmentHasherBuilder::new(hash_builder);
        Self {
            hs: HashSet::with_capacity_and_hasher(n, hasher_buider),
            slab: Slab::with_capacity(n),
        }
    }

    /// Create a new map with `hash_builder`.
    pub fn with_hasher(hash_builder: S) -> Self {
        Self::with_capacity_and_hasher(0, hash_builder)
    }

    /// Return the number of values the hashslab can store without reallocating.
    pub fn capacity(&self) -> usize {
        self.hs.capacity().min(self.slab.capacity())
    }

    /// Returns a reference to the hashset's [`BuildHasher`].
    ///
    /// [`BuildHasher`]: https://doc.rust-lang.org/std/hash/trait.BuildHasher.html
    pub fn hasher(&self) -> &S {
        &self.hs.hasher().0
    }

    /// Return the number of key-value pairs in the map.
    #[inline]
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.hs.len(), self.slab.len());
        self.hs.len()
    }

    /// Returns true if the map contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// An iterator over the index-key-value triples in arbitrary order.
    pub fn iter_full(&self) -> IterFull<'_, K, V> {
        IterFull::new(self.hs.iter(), &self.slab)
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter::new(self.hs.iter(), &self.slab)
    }

    /// An iterator visiting all index-key-value triple in arbitrary order, with mutable references to the values.
    pub fn iter_full_mut(&mut self) -> IterFullMut<'_, K, V, S>
    where
        K: Clone,
    {
        IterFullMut::new(&self.hs, self.slab.iter_mut())
    }

    /// An iterator visiting all key-value pairs in arbitrary order, with mutable references to the values.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V, S>
    where
        K: Clone,
    {
        IterMut::new(self.iter_full_mut())
    }

    /// An iterator visiting index-key pairs in arbitrary order. The iterator element type is [`(usize, &'a K)`].
    pub fn full_keys(&self) -> FullKeys<'_, K> {
        FullKeys::new(self.hs.iter())
    }

    /// An iterator visiting all keys in arbitrary order. The iterator element type is [`&'a K`].
    pub fn keys(&self) -> Keys<'_, K> {
        Keys::new(self.hs.iter())
    }

    /// Return an owning iterator over the keys of the map, in their order
    pub fn into_keys(self) -> IntoKeys<K> {
        IntoKeys::new(self.hs.into_iter())
    }

    /// Return an iterator over the values of the map, in their order
    pub fn values(&self) -> Values<'_, K, V> {
        Values::new(self.iter_full())
    }

    /// Return an iterator that allows modifying each value.
    ///
    /// This function should generally be *avoided* as it use [`slab::IterMut`], which is not efficient.
    pub fn values_mut(&mut self) -> ValuesMut<'_, V> {
        ValuesMut::new(self.slab.iter_mut())
    }

    /// Return an owning iterator over the values of the map, in their order
    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues::new(self.into_iter())
    }

    /// An iterator over indices in arbitrary order. The iterator element type is [`usize`].
    pub fn indices(&self) -> Indices<'_, K> {
        Indices::new(self.hs.iter())
    }

    /// Remove all entries in the map, while preserving its capacity.
    pub fn clear(&mut self) {
        self.hs.clear();
        self.slab.clear();
    }

    /// Clears the map, returning all index-key-value triples as an iterator. Keeps the allocated memory for reuse.
    pub fn drain_full(&mut self) -> DrainFull<'_, K, V> {
        DrainFull::new(self.hs.drain(), &mut self.slab)
    }

    /// Clears the map, returning all key-value pairs as an iterator. Keeps the allocated memory for reuse.
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        Drain::new(self.drain_full())
    }
}

impl<K, V, S> HashSlabMap<K, V, S> where S: BuildHasher {}

impl<K, V, S> HashSlabMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Reserve capacity for `additional` more key-value pairs.
    pub fn reserve(&mut self, additional: usize) {
        self.hs.reserve(additional);
        self.slab.reserve(additional);
    }

    /// Try to reserve capacity for `additional` more key-value pairs.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.hs.try_reserve(additional)?;
        let capacity = self.slab.capacity();
        if (capacity + additional) <= isize::MAX as usize {
            self.slab.reserve(additional);
            Ok(())
        } else {
            Err(TryReserveError::Slab {
                capacity,
                additional,
            })
        }
    }

    /// Shrink the capacity of the map as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.hs.shrink_to_fit();
        self.slab.shrink_to_fit();
    }

    /// Insert a key-value pair in the map.
    ///
    /// If an equivalent key already exists in the map: the key remains and
    /// retains in its place in the order, its corresponding value is updated
    /// with `value`, and the older value is returned inside `Some(_)`.
    ///
    /// If no equivalent key existed in the map: the new key-value pair is
    /// inserted, last in order, and `None` is returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert_full(key, value).1
    }

    /// Insert a key-value pair in the map, and get their index.
    ///
    /// If an equivalent key already exists in the map: the key remains and
    /// retains in its place in the order, its corresponding value is updated
    /// with `value`, and the older value is returned inside `(index, Some(_))`.
    ///
    /// If no equivalent key existed in the map: the new key-value pair is
    /// inserted, last in order, and `(index, None)` is returned.
    pub fn insert_full(&mut self, key: K, value: V) -> (usize, Option<V>) {
        if let Some(entry) = self.hs.get(&Query(&key)) {
            let index = entry.index;
            let ValueEntry { data, .. } = self
                .slab
                .get_mut(index)
                .expect("slab should has entry with existing hashset entry");
            let mut value = value;
            mem::swap(data, &mut value);
            (index, Some(value))
        } else {
            let mut hasher = self.hs.hasher().build_hasher();
            key.hash(&mut hasher);
            let hash_value = hasher.finish();
            let index = self.slab.insert(ValueEntry {
                hash_value,
                data: value,
            });
            let hs_entry = KeyEntry { index, key };
            self.hs.insert(hs_entry);
            (index, None)
        }
    }

    /// Return item index, key and value
    pub fn get_full<Q>(&self, key: &Q) -> Option<(usize, &K, &V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        let KeyEntry { index, key } = self.hs.get(&Query(key))?;
        self.slab
            .get(*index)
            .map(|ValueEntry { data, .. }| (*index, key, data))
    }

    /// Return references to the key-value pair stored for `key`, if it is present, else `None`.
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.get_full(key).map(|(_, key, data)| (key, data))
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.get_full(key).map(|(_, _, data)| data)
    }

    /// Get a key-value pair by index
    pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
        let ValueEntry { hash_value, data } = self.slab.get(index)?;
        self.hs
            .get(&RawHash {
                value: *hash_value,
                index,
            })
            .map(|KeyEntry { key, .. }| (key, data))
    }

    /// Get a value by index.
    pub fn get_index_value(&self, index: usize) -> Option<&V> {
        self.slab.get(index).map(|ValueEntry { data, .. }| data)
    }

    /// Return item index, if it exists in the map
    pub fn get_index_of<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.hs
            .get(&Query(key))
            .map(|KeyEntry { index, .. }| *index)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: ?Sized + Hash + Equivalent<K>,
    {
        let KeyEntry { index, .. } = self.hs.get(&Query(key))?;
        self.slab
            .get_mut(*index)
            .map(|ValueEntry { data, .. }| data)
    }

    pub fn get_index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        let ValueEntry { hash_value, data } = self.slab.get_mut(index)?;
        self.hs
            .get(&RawHash {
                index,
                value: *hash_value,
            })
            .map(|KeyEntry { key, .. }| (key, data))
    }

    /// Remove the key-value pair equivalent to `key` and return its value.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: ?Sized + Hash + Equivalent<K>,
    {
        self.remove_entry(key).map(|(_, v)| v)
    }

    /// Remove and return the key-value pair equivalent to `key`.
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        Q: ?Sized + Hash + Equivalent<K>,
    {
        self.remove_full(key).map(|(_, k, v)| (k, v))
    }

    /// Remove the key-value pair equivalent to key and return it and the index it had.
    pub fn remove_full<Q>(&mut self, key: &Q) -> Option<(usize, K, V)>
    where
        Q: ?Sized + Hash + Equivalent<K>,
    {
        let KeyEntry { index, key } = self.hs.take(&Query(key))?;
        self.slab
            .try_remove(index)
            .map(|ValueEntry { data, .. }| (index, key, data))
    }

    /// Remove the key-value pair by index
    pub fn remove_index(&mut self, index: usize) -> Option<(K, V)> {
        let ValueEntry { data, hash_value } = self.slab.try_remove(index)?;
        self.hs
            .take(&RawHash {
                index,
                value: hash_value,
            })
            .map(|KeyEntry { key, .. }| (key, data))
    }
}

// https://github.com/rust-lang/rust/issues/26925
impl<K: Clone, V: Clone, S: Clone> Clone for HashSlabMap<K, V, S> {
    fn clone(&self) -> Self {
        Self {
            hs: self.hs.clone(),
            slab: self.slab.clone(),
        }
    }
}

impl<K, V> fmt::Debug for HashSlabMap<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.iter_full().map(|(i, k, v)| (i, (k, v))))
            .finish()
    }
}

impl<K, V, S> Default for HashSlabMap<K, V, S>
where
    S: Default,
{
    fn default() -> Self {
        Self::with_capacity_and_hasher(0, S::default())
    }
}

/// Access [`HashSlabMap`] values corresponding to a key.
///
/// # Examples
///
/// ```
/// use hashslab::HashSlabMap;
///
/// let mut map = HashSlabMap::new();
/// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
///     map.insert(word.to_lowercase(), word.to_uppercase());
/// }
/// assert_eq!(map["lorem"], "LOREM");
/// assert_eq!(map["ipsum"], "IPSUM");
/// ```
///
/// ```should_panic
/// use hashslab::HashSlabMap;
///
/// let mut map = HashSlabMap::new();
/// map.insert("foo", 1);
/// println!("{:?}", map["bar"]); // panics!
/// ```
impl<K, V, Q: ?Sized, S> Index<&Q> for HashSlabMap<K, V, S>
where
    K: Hash + Eq,
    Q: Hash + Equivalent<K>,
    S: BuildHasher,
{
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied `key`.
    ///
    /// ***Panics*** if `key` is not present in the map.
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("HashSlabMap: key not found")
    }
}

/// Access [`HashSlabMap`] values at indexed positions.
///
/// # Examples
///
/// ```
/// use hashslab::HashSlabMap;
///
/// let mut map = HashSlabMap::new();
/// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
///     map.insert(word.to_lowercase(), word.to_uppercase());
/// }
/// assert_eq!(map[0], "LOREM");
/// assert_eq!(map[1], "IPSUM");
/// ```
///
/// ```should_panic
/// use hashslab::HashSlabMap;
///
/// let mut map = HashSlabMap::new();
/// map.insert("foo", 1);
/// println!("{:?}", map[10]); // panics!
/// ```
impl<K, V, S> Index<usize> for HashSlabMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    type Output = V;

    /// Returns a reference to the value at the supplied `index`.
    ///
    /// ***Panics*** if `index` is out of bounds.
    fn index(&self, index: usize) -> &V {
        self.get_index(index)
            .expect("HashSlabMap: index out of bounds")
            .1
    }
}

/// Access [`HashSlabMap`] values corresponding to a key.
///
/// Mutable indexing allows changing / updating values of key-value
/// pairs that are already present.
///
/// You can **not** insert new pairs with index syntax, use `.insert()`.
///
/// # Examples
///
/// ```
/// use hashslab::HashSlabMap;
///
/// let mut map = HashSlabMap::new();
/// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
///     map.insert(word.to_lowercase(), word.to_string());
/// }
/// let lorem = &mut map["lorem"];
/// assert_eq!(lorem, "Lorem");
/// lorem.retain(char::is_lowercase);
/// assert_eq!(map["lorem"], "orem");
/// ```
///
/// ```should_panic
/// use hashslab::HashSlabMap;
///
/// let mut map = HashSlabMap::new();
/// map.insert("foo", 1);
/// map["bar"] = 1; // panics!
/// ```
impl<K, V, Q: ?Sized, S> IndexMut<&Q> for HashSlabMap<K, V, S>
where
    K: Hash + Eq,
    Q: Hash + Equivalent<K>,
    S: BuildHasher,
{
    /// Returns a mutable reference to the value corresponding to the supplied `key`.
    ///
    /// ***Panics*** if `key` is not present in the map.
    fn index_mut(&mut self, key: &Q) -> &mut V {
        self.get_mut(key).expect("HashSlabMap: key not found")
    }
}

/// Access [`HashSlabMap`] values at indexed positions.
///
/// Mutable indexing allows changing / updating indexed values
/// that are already present.
///
/// You can **not** insert new values with index syntax -- use [`.insert()`][HashSlabMap::insert].
///
/// # Examples
///
/// ```
/// use hashslab::HashSlabMap;
///
/// let mut map = HashSlabMap::new();
/// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
///     map.insert(word.to_lowercase(), word.to_string());
/// }
/// let lorem = &mut map[0];
/// assert_eq!(lorem, "Lorem");
/// lorem.retain(char::is_lowercase);
/// assert_eq!(map["lorem"], "orem");
/// ```
///
/// ```should_panic
/// use hashslab::HashSlabMap;
///
/// let mut map = HashSlabMap::new();
/// map.insert("foo", 1);
/// map[10] = 1; // panics!
/// ```
impl<K, V, S> IndexMut<usize> for HashSlabMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Returns a mutable reference to the value at the supplied `index`.
    ///
    /// ***Panics*** if `index` is out of bounds.
    fn index_mut(&mut self, index: usize) -> &mut V {
        self.get_index_mut(index)
            .expect("HashSlabMap: index out of bounds")
            .1
    }
}

impl<K, V, S> Extend<(K, V)> for HashSlabMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Extend the map with all key-value pairs in the iterable.
    ///
    /// This is equivalent to calling [`insert`][HashSlabMap::insert] for each of
    /// them in order, which means that for keys that already existed
    /// in the map, their value is updated but it keeps the existing order.
    ///
    /// New keys are inserted in the order they appear in the sequence. If
    /// equivalents of a key occur more than once, the last corresponding value
    /// prevails.
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iterable: I) {
        // (Note: this is a copy of `std`/`hashbrown`'s reservation logic.)
        // Keys may be already present or show multiple times in the iterator.
        // Reserve the entire hint lower bound if the map is empty.
        // Otherwise reserve half the hint (rounded up), so the map
        // will only resize twice in the worst case.
        let iter = iterable.into_iter();
        let reserve = if self.is_empty() {
            iter.size_hint().0
        } else {
            (iter.size_hint().0 + 1) / 2
        };
        self.reserve(reserve);
        iter.for_each(move |(k, v)| {
            self.insert(k, v);
        });
    }
}

impl<'a, K, V, S> Extend<(&'a K, &'a V)> for HashSlabMap<K, V, S>
where
    K: Hash + Eq + Copy,
    V: Copy,
    S: BuildHasher,
{
    /// Extend the map with all key-value pairs in the iterable.
    ///
    /// See the first extend method for more details.
    fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iterable: I) {
        self.extend(iterable.into_iter().map(|(&key, &value)| (key, value)));
    }
}

impl<K, V, S> FromIterator<(K, V)> for HashSlabMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher + Default,
{
    /// Create an `HashSlabMap` from the sequence of key-value pairs in the
    /// iterable.
    ///
    /// `from_iter` uses the same logic as `extend`. See
    /// [`extend`][HashSlabMap::extend] for more details.
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iterable: I) -> Self {
        let iter = iterable.into_iter();
        let (low, _) = iter.size_hint();
        let mut map = Self::with_capacity_and_hasher(low, <_>::default());
        map.extend(iter);
        map
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for HashSlabMap<K, V, RandomState>
where
    K: Hash + Eq,
{
    /// # Examples
    ///
    /// ```
    /// use hashslab::HashSlabMap;
    ///
    /// let map1 = HashSlabMap::from([(1, 2), (3, 4)]);
    /// let map2: HashSlabMap<_, _> = [(1, 2), (3, 4)].into();
    /// assert_eq!(map1, map2);
    /// ```
    fn from(arr: [(K, V); N]) -> Self {
        Self::from_iter(arr)
    }
}

impl<K, V1, S1, V2, S2> PartialEq<HashSlabMap<K, V2, S2>> for HashSlabMap<K, V1, S1>
where
    K: Hash + Eq,
    V1: PartialEq<V2>,
    S1: BuildHasher,
    S2: BuildHasher,
{
    fn eq(&self, other: &HashSlabMap<K, V2, S2>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V, S> Eq for HashSlabMap<K, V, S>
where
    K: Eq + Hash,
    V: Eq,
    S: BuildHasher,
{
}
