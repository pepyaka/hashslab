use std::{fmt, hash::Hash, mem};

use hashbrown::hash_table;
use slab::Slab;

use crate::{KeyData, ValueData};

/// A view into a single entry, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`HashSlabMap`].
///
/// [`HashSlabMap`]: struct.HashSlabMap.html
/// [`entry`]: struct.HashSlabMap.html#method.entry
///
/// # Examples
///
/// ```
/// use hashslab::map::{Entry, HashSlabMap, OccupiedEntry};
///
/// let mut map = HashSlabMap::new();
/// map.extend([("a", 10), ("b", 20), ("c", 30)]);
/// assert_eq!(map.len(), 3);
///
/// // Existing key (or_insert)
/// let v = map.entry("b").or_insert(2);
/// assert_eq!(std::mem::replace(v, 2), 20);
/// // Nonexistent key (or_insert)
/// map.entry("d").or_insert(4);
///
/// // Existing key (or_insert_with)
/// let v = map.entry("c").or_insert_with(|| 3);
/// assert_eq!(std::mem::replace(v, 3), 30);
/// // Nonexistent key (or_insert_with)
/// map.entry("e").or_insert_with(|| 5);
///
/// println!("Our HashSlabMap: {:?}", map);
///
/// let mut vec: Vec<_> = map.iter().map(|(&k, &v)| (k, v)).collect();
/// // The `Iter` iterator produces items in arbitrary order, so the
/// // items must be sorted to test them against a sorted array.
/// vec.sort_unstable();
/// assert_eq!(vec, [("a", 10), ("b", 2), ("c", 3), ("d", 4), ("e", 5)]);
/// ```
pub enum Entry<'a, K, V> {
    /// Existing slot with equivalent key.
    Occupied(OccupiedEntry<'a, K, V>),
    /// Vacant slot (no equivalent key in the map).
    Vacant(VacantEntry<'a, K, V>),
}

impl<K, V> Entry<'_, K, V> {
    /// Return the index where the key-value pair exists or will be inserted.
    pub fn index(&self) -> usize {
        match *self {
            Entry::Occupied(ref entry) => entry.index(),
            Entry::Vacant(ref entry) => entry.index(),
        }
    }
    /// Gets a reference to the entry's key, either within the map if occupied,
    /// or else the new key that was used to find the entry.
    pub fn key(&self) -> &K {
        match *self {
            Entry::Occupied(ref entry) => entry.key(),
            Entry::Vacant(ref entry) => entry.key(),
        }
    }

    /// Modifies the entry if it is occupied.
    pub fn and_modify<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        if let Entry::Occupied(entry) = &mut self {
            f(entry.get_mut());
        }
        self
    }
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Hash,
{
    /// Sets the value of the entry (after inserting if vacant), and returns an `OccupiedEntry`.
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V> {
        match self {
            Entry::Occupied(mut entry) => {
                entry.insert(value);
                entry
            }
            Entry::Vacant(entry) => entry.insert_entry(value),
        }
    }

    /// Inserts the given default value in the entry if it is vacant and returns a mutable
    /// reference to it. Otherwise a mutable reference to an already existent value is returned.
    ///
    /// Computes in **O(1)** time (amortized average).
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Inserts the result of the `call` function in the entry if it is vacant and returns a mutable
    /// reference to it. Otherwise a mutable reference to an already existent value is returned.
    ///
    /// Computes in **O(1)** time (amortized average).
    pub fn or_insert_with<F>(self, call: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(call()),
        }
    }

    /// Inserts the result of the `call` function with a reference to the entry's key if it is
    /// vacant, and returns a mutable reference to the new value. Otherwise a mutable reference to
    /// an already existent value is returned.
    ///
    /// Computes in **O(1)** time (amortized average).
    pub fn or_insert_with_key<F>(self, call: F) -> &'a mut V
    where
        F: FnOnce(&K) -> V,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let value = call(entry.key());
                entry.insert(value)
            }
        }
    }

    /// Inserts a default-constructed value in the entry if it is vacant and returns a mutable
    /// reference to it. Otherwise a mutable reference to an already existent value is returned.
    ///
    /// Computes in **O(1)** time (amortized average).
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(V::default()),
        }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Entry<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tuple = f.debug_tuple("Entry");
        match self {
            Entry::Vacant(v) => tuple.field(v),
            Entry::Occupied(o) => tuple.field(o),
        };
        tuple.finish()
    }
}

/// A view into an occupied entry in an [`HashSlabMap`][crate::HashSlabMap].
/// It is part of the [`Entry`] enum.
pub struct OccupiedEntry<'a, K, V> {
    inner: hash_table::OccupiedEntry<'a, KeyData<K>>,
    slab: &'a mut Slab<ValueData<V>>,
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    pub(super) fn new(
        inner: hash_table::OccupiedEntry<'a, KeyData<K>>,
        slab: &'a mut Slab<ValueData<V>>,
    ) -> Self {
        Self { inner, slab }
    }

    /// Return the index of the key-value pair
    #[inline]
    pub fn index(&self) -> usize {
        self.inner.get().index
    }

    // #[inline]
    // fn into_ref_mut(self) -> RefMut<'a, K, V> {
    //     RefMut::new(self.index.into_table(), self.entries)
    // }

    /// Gets a reference to the entry's key in the map.
    ///
    /// Note that this is not the key that was used to find the entry. There may be an observable
    /// difference if the key type has any distinguishing features outside of `Hash` and `Eq`, like
    /// extra fields or the memory address of an allocation.
    pub fn key(&self) -> &K {
        &self.inner.get().key
    }

    /// Gets a reference to the entry's value in the map.
    pub fn get(&self) -> &V {
        let KeyData { index, .. } = self.inner.get();
        &self.slab[*index].value
    }

    /// Gets a mutable reference to the entry's value in the map.
    ///
    /// If you need a reference which may outlive the destruction of the
    /// [`Entry`] value, see [`into_mut`][Self::into_mut].
    pub fn get_mut(&mut self) -> &mut V {
        let KeyData { index, .. } = self.inner.get();
        &mut self.slab[*index].value
    }

    /// Converts into a mutable reference to the entry's value in the map,
    /// with a lifetime bound to the map itself.
    pub fn into_mut(self) -> &'a mut V {
        let KeyData { index, .. } = self.inner.get();
        &mut self.slab[*index].value
    }

    /// Sets the value of the entry to `value`, and returns the entry's old value.
    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.get_mut(), value)
    }

    /// Remove the key, value pair stored in the map for this entry, and return the value.
    pub fn remove(self) -> V {
        self.remove_entry().1
    }

    /// Remove and return the key, value pair stored in the map for this entry
    pub fn remove_entry(self) -> (K, V) {
        let (KeyData { key, index, .. }, _) = self.inner.remove();
        let ValueData { value, .. } = self.slab.remove(index);
        (key, value)
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for OccupiedEntry<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("key", self.key())
            .field("value", self.get())
            .finish()
    }
}

// impl<'a, K, V> From<IndexedEntry<'a, K, V>> for OccupiedEntry<'a, K, V> {
//     fn from(other: IndexedEntry<'a, K, V>) -> Self {
//         let IndexedEntry {
//             map: RefMut { indices, entries },
//             index,
//         } = other;
//         let hash = entries[index].hash;
//         Self {
//             entries,
//             index: indices
//                 .find_entry(hash.get(), move |&i| i == index)
//                 .expect("index not found"),
//         }
//     }
// }

/// A view into a vacant entry in an [`HashSlabMap`][crate::HashSlabMap].
/// It is part of the [`Entry`] enum.
pub struct VacantEntry<'a, K, V> {
    inner: hash_table::VacantEntry<'a, KeyData<K>>,
    slab: &'a mut Slab<ValueData<V>>,
    key: K,
    hash: u64,
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    pub(super) fn new(
        inner: hash_table::VacantEntry<'a, KeyData<K>>,
        slab: &'a mut Slab<ValueData<V>>,
        key: K,
        hash: u64,
    ) -> Self {
        Self {
            inner,
            slab,
            key,
            hash,
        }
    }

    /// Return the index where a key-value pair may be inserted.
    pub fn index(&self) -> usize {
        self.slab.vacant_key()
    }

    /// Gets a reference to the key that was used to find the entry.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Takes ownership of the key, leaving the entry vacant.
    pub fn into_key(self) -> K {
        self.key
    }

    /// Inserts the entry's key and the given value into the map, and returns a mutable reference
    /// to the value.
    pub fn insert(self, value: V) -> &'a mut V
    where
        K: Hash,
    {
        let (inner, slab) = self.table_entry_insert(value);
        let index = inner.get().index;
        &mut slab[index].value
    }

    /// Sets the value of the entry with the [`VacantEntry`]'s key, and returns an [`OccupiedEntry`].
    ///
    /// # Examples
    ///
    /// ```
    /// use hashslab::HashSlabMap;
    /// use hashslab::map::Entry;
    ///
    /// let mut map: HashSlabMap<&str, u32> = HashSlabMap::new();
    ///
    /// if let Entry::Vacant(v) = map.entry("poneyland") {
    ///     let o = v.insert_entry(37);
    ///     assert_eq!(o.get(), &37);
    /// }
    /// ```
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V>
    where
        K: Hash,
    {
        let (inner, slab) = self.table_entry_insert(value);
        OccupiedEntry { inner, slab }
    }
}

// Private functions
impl<'a, K, V> VacantEntry<'a, K, V> {
    fn table_entry_insert(
        self,
        value: V,
    ) -> (
        hash_table::OccupiedEntry<'a, KeyData<K>>,
        &'a mut Slab<ValueData<V>>,
    )
    where
        K: Hash,
    {
        let index = self.slab.insert(ValueData::new(value, self.hash));
        let inner = self.inner.insert(KeyData::new(self.key, index));
        (inner, self.slab)
    }
}

impl<K: fmt::Debug, V> fmt::Debug for VacantEntry<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VacantEntry").field(self.key()).finish()
    }
}

// /// A view into an occupied entry in an [`HashSlabMap`][crate::HashSlabMap] obtained by index.
// ///
// /// This `struct` is created from the [`get_index_entry`][crate::HashSlabMap::get_index_entry] method.
// pub struct IndexedEntry<'a, K, V> {
//     map: RefMut<'a, K, V>,
//     // We have a mutable reference to the map, which keeps the index
//     // valid and pointing to the correct entry.
//     index: usize,
// }

// impl<'a, K, V> IndexedEntry<'a, K, V> {
//     pub(crate) fn new(map: &'a mut HashSlabMapCore<K, V>, index: usize) -> Self {
//         Self {
//             map: map.borrow_mut(),
//             index,
//         }
//     }

//     /// Return the index of the key-value pair
//     #[inline]
//     pub fn index(&self) -> usize {
//         self.index
//     }

//     /// Gets a reference to the entry's key in the map.
//     pub fn key(&self) -> &K {
//         &self.map.entries[self.index].key
//     }

//     pub(crate) fn key_mut(&mut self) -> &mut K {
//         &mut self.map.entries[self.index].key
//     }

//     /// Gets a reference to the entry's value in the map.
//     pub fn get(&self) -> &V {
//         &self.map.entries[self.index].value
//     }

//     /// Gets a mutable reference to the entry's value in the map.
//     ///
//     /// If you need a reference which may outlive the destruction of the
//     /// `IndexedEntry` value, see [`into_mut`][Self::into_mut].
//     pub fn get_mut(&mut self) -> &mut V {
//         &mut self.map.entries[self.index].value
//     }

//     /// Sets the value of the entry to `value`, and returns the entry's old value.
//     pub fn insert(&mut self, value: V) -> V {
//         mem::replace(self.get_mut(), value)
//     }

//     /// Converts into a mutable reference to the entry's value in the map,
//     /// with a lifetime bound to the map itself.
//     pub fn into_mut(self) -> &'a mut V {
//         &mut self.map.entries[self.index].value
//     }

//     /// Remove and return the key, value pair stored in the map for this entry
//     ///
//     /// Like [`Vec::swap_remove`][crate::Vec::swap_remove], the pair is removed by swapping it with
//     /// the last element of the map and popping it off.
//     /// **This perturbs the position of what used to be the last element!**
//     ///
//     /// Computes in **O(1)** time (average).
//     pub fn swap_remove_entry(mut self) -> (K, V) {
//         self.map.swap_remove_index(self.index).unwrap()
//     }

//     /// Remove and return the key, value pair stored in the map for this entry
//     ///
//     /// Like [`Vec::remove`][crate::Vec::remove], the pair is removed by shifting all of the
//     /// elements that follow it, preserving their relative order.
//     /// **This perturbs the index of all of those elements!**
//     ///
//     /// Computes in **O(n)** time (average).
//     pub fn shift_remove_entry(mut self) -> (K, V) {
//         self.map.shift_remove_index(self.index).unwrap()
//     }

//     /// Remove the key, value pair stored in the map for this entry, and return the value.
//     ///
//     /// Like [`Vec::swap_remove`][crate::Vec::swap_remove], the pair is removed by swapping it with
//     /// the last element of the map and popping it off.
//     /// **This perturbs the position of what used to be the last element!**
//     ///
//     /// Computes in **O(1)** time (average).
//     pub fn swap_remove(self) -> V {
//         self.swap_remove_entry().1
//     }

//     /// Remove the key, value pair stored in the map for this entry, and return the value.
//     ///
//     /// Like [`Vec::remove`][crate::Vec::remove], the pair is removed by shifting all of the
//     /// elements that follow it, preserving their relative order.
//     /// **This perturbs the index of all of those elements!**
//     ///
//     /// Computes in **O(n)** time (average).
//     pub fn shift_remove(self) -> V {
//         self.shift_remove_entry().1
//     }

//     /// Moves the position of the entry to a new index
//     /// by shifting all other entries in-between.
//     ///
//     /// This is equivalent to [`HashSlabMap::move_index`][`crate::HashSlabMap::move_index`]
//     /// coming `from` the current [`.index()`][Self::index].
//     ///
//     /// * If `self.index() < to`, the other pairs will shift down while the targeted pair moves up.
//     /// * If `self.index() > to`, the other pairs will shift up while the targeted pair moves down.
//     ///
//     /// ***Panics*** if `to` is out of bounds.
//     ///
//     /// Computes in **O(n)** time (average).
//     pub fn move_index(mut self, to: usize) {
//         self.map.move_index(self.index, to);
//     }

//     /// Swaps the position of entry with another.
//     ///
//     /// This is equivalent to [`HashSlabMap::swap_indices`][`crate::HashSlabMap::swap_indices`]
//     /// with the current [`.index()`][Self::index] as one of the two being swapped.
//     ///
//     /// ***Panics*** if the `other` index is out of bounds.
//     ///
//     /// Computes in **O(1)** time (average).
//     pub fn swap_indices(mut self, other: usize) {
//         self.map.swap_indices(self.index, other);
//     }
// }

// impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IndexedEntry<'_, K, V> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("IndexedEntry")
//             .field("index", &self.index)
//             .field("key", self.key())
//             .field("value", self.get())
//             .finish()
//     }
// }

// impl<'a, K, V> From<OccupiedEntry<'a, K, V>> for IndexedEntry<'a, K, V> {
//     fn from(other: OccupiedEntry<'a, K, V>) -> Self {
//         Self {
//             index: other.index(),
//             map: other.into_ref_mut(),
//         }
//     }
// }
