//! A hash set implemented using [`HashSlabMap`]
use core::{
    fmt,
    hash::{BuildHasher, Hash},
    mem,
    ops::{BitAnd, BitOr, BitXor, Index, Sub},
};

#[cfg(feature = "std")]
use std::hash::RandomState;

use hashbrown::{hash_table, Equivalent};

use crate::{map::make_hasher, HashSlabMap, KeyData, TryReserveError, ValueData};

mod iter;
pub use iter::{
    Difference, Drain, Intersection, IntoIter, Iter, IterFull, SymmetricDifference, Union,
};

// mod mutable;
// mod slice;

#[cfg(test)]
mod tests;

/// A hashslab set implemented as a `HashSlabMap` where the value is `()`.
///
/// # Examples
///
/// ```
/// use hashslab::HashSlabSet;
/// // Type inference lets us omit an explicit type signature (which
/// // would be `HashSlabSet<String>` in this example).
/// let mut books = HashSlabSet::new();
///
/// // Add some books.
/// books.insert("A Dance With Dragons".to_string());
/// books.insert("To Kill a Mockingbird".to_string());
/// books.insert("The Odyssey".to_string());
/// books.insert("The Great Gatsby".to_string());
///
/// // Check for a specific one.
/// if !books.contains("The Winds of Winter") {
///     println!("We have {} books, but The Winds of Winter ain't one.",
///              books.len());
/// }
///
/// // Remove a book.
/// books.remove("The Odyssey");
///
/// // Iterate over everything.
/// for book in &books {
///     println!("{}", book);
/// }
/// ```
///
/// The easiest way to use `HashSlabSet` with a custom type is to derive
/// [`Eq`] and [`Hash`]. We must also derive [`PartialEq`]. This will in the
/// future be implied by [`Eq`].
///
/// ```
/// use hashslab::HashSlabSet;
/// #[derive(Hash, Eq, PartialEq, Debug)]
/// struct Viking {
///     name: String,
///     power: usize,
/// }
///
/// let mut vikings = HashSlabSet::new();
///
/// vikings.insert(Viking { name: "Einar".to_string(), power: 9 });
/// vikings.insert(Viking { name: "Einar".to_string(), power: 9 });
/// vikings.insert(Viking { name: "Olaf".to_string(), power: 4 });
/// vikings.insert(Viking { name: "Harald".to_string(), power: 8 });
///
/// // Use derived implementation to print the vikings.
/// for x in &vikings {
///     println!("{:?}", x);
/// }
/// ```
///
/// A `HashSlabSet` with fixed list of elements can be initialized from an array:
///
/// ```
/// use hashslab::HashSlabSet;
///
/// let viking_names: HashSlabSet<&'static str> =
///     [ "Einar", "Olaf", "Harald" ].into_iter().collect();
/// // use the values stored in the set
/// ```
///
/// A `HashSlabSet` can get value by index
/// ```
/// # use hashslab::HashSlabSet;
/// let set = HashSlabSet::from(['A','B']);
///
/// assert_eq!(Some(&'A'), set.get_index(0));
/// ```
///
/// [`Cell`]: https://doc.rust-lang.org/std/cell/struct.Cell.html
/// [`Eq`]: https://doc.rust-lang.org/std/cmp/trait.Eq.html
/// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
/// [`HashMap`]: struct.HashMap.html
/// [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
/// [`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
#[cfg(feature = "std")]
pub struct HashSlabSet<T, S = RandomState> {
    pub(crate) map: HashSlabMap<T, (), S>,
}

#[cfg(not(feature = "std"))]
pub struct HashSlabSet<T, S> {
    pub(crate) map: HashSlabMap<T, (), S>,
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T> HashSlabSet<T> {
    /// Creates an empty `HashSlabSet`.
    ///
    /// The hash set is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let set: HashSlabSet<i32> = HashSlabSet::new();
    /// assert!(set.capacity() >= 0);
    /// ```
    pub fn new() -> Self {
        HashSlabSet {
            map: HashSlabMap::new(),
        }
    }

    /// Creates an empty `HashSlabSet` with the specified capacity.
    ///
    /// The hash set will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash set will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let set: HashSlabSet<i32> = HashSlabSet::with_capacity(10);
    /// assert!(set.capacity() >= 10);
    /// ```
    pub fn with_capacity(n: usize) -> Self {
        HashSlabSet {
            map: HashSlabMap::with_capacity(n),
        }
    }
}

impl<T, S> HashSlabSet<T, S> {
    /// Create a new set with capacity for `n` elements.
    /// (Does not allocate if `n` is zero.)
    ///
    /// Computes in **O(n)** time.
    pub fn with_capacity_and_hasher(n: usize, hash_builder: S) -> Self {
        HashSlabSet {
            map: HashSlabMap::with_capacity_and_hasher(n, hash_builder),
        }
    }

    /// Create a new set with `hash_builder`.
    ///
    /// This function is `const`, so it
    /// can be called in `static` contexts.
    pub const fn with_hasher(hash_builder: S) -> Self {
        HashSlabSet {
            map: HashSlabMap::with_hasher(hash_builder),
        }
    }

    /// Return the number of elements the set can hold without reallocating.
    ///
    /// This number is a lower bound; the set might be able to hold more,
    /// but is guaranteed to be able to hold at least this many.
    ///
    /// Computes in **O(1)** time.
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    /// Return a reference to the set's `BuildHasher`.
    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }

    /// Return the number of elements in the set.
    ///
    /// Computes in **O(1)** time.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the set contains no elements.
    ///
    /// Computes in **O(1)** time.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// An iterator visiting all elements in arbitrary order.
    /// The iterator element type is `&'a T`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let mut set = HashSlabSet::new();
    /// set.insert("a");
    /// set.insert("b");
    ///
    /// // Will print in an arbitrary order.
    /// for x in set.iter() {
    ///     println!("{}", x);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self.map.keys())
    }

    /// An iterator visiting all index-value pairs in arbitrary order.
    /// The iterator element type is `(usize, &'a T)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let mut set = HashSlabSet::new();
    /// set.insert("a");
    /// set.insert("b");
    ///
    /// // Will print in an arbitrary order.
    /// for (i, x) in set.iter_full() {
    ///     println!("{}: {}", i, x);
    /// }
    /// ```
    pub fn iter_full(&self) -> IterFull<'_, T> {
        IterFull::new(self.map.full_keys())
    }

    /// Clears the set, returning all elements in an iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let mut set: HashSlabSet<_> = [1, 2, 3].into_iter().collect();
    /// assert!(!set.is_empty());
    ///
    /// // print 1, 2, 3 in an arbitrary order
    /// for i in set.drain() {
    ///     println!("{}", i);
    /// }
    ///
    /// assert!(set.is_empty());
    /// ```
    pub fn drain(&mut self) -> Drain<'_, T> {
        Drain::new(self.map.drain())
    }

    /// Remove all elements in the set, while preserving its capacity.
    ///
    /// Computes in **O(n)** time.
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl<T, S> HashSlabSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    /// Shrink the capacity of the set as much as possible.
    ///
    /// Computes in **O(n)** time.
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
    }

    /// Reserve capacity for `additional` more values.
    ///
    /// Computes in **O(n)** time.
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional);
    }

    /// Try to reserve capacity for `additional` more values.
    ///
    /// Computes in **O(n)** time.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.map.try_reserve(additional)
    }

    /// Insert the value into the set.
    ///
    /// If an equivalent item already exists in the set, it returns
    /// `false` leaving the original value in the set and without
    /// altering its insertion order. Otherwise, it inserts the new
    /// item and returns `true`.
    ///
    /// Computes in **O(1)** time (amortized average).
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    /// Insert the value into the set, and get its index.
    ///
    /// If an equivalent item already exists in the set, it returns
    /// the index of the existing item and `false`, leaving the
    /// original value in the set and without altering its insertion
    /// order. Otherwise, it inserts the new item and returns the index
    /// of the inserted item and `true`.
    ///
    /// Computes in **O(1)** time (amortized average).
    pub fn insert_full(&mut self, value: T) -> (usize, bool) {
        let (index, existing) = self.map.insert_full(value, ());
        (index, existing.is_none())
    }

    /// Adds a value to the set, replacing the existing value, if any, that is
    /// equal to the given one, without altering its insertion order. Returns
    /// the replaced value.
    ///
    /// Computes in **O(1)** time (average).
    pub fn replace(&mut self, value: T) -> Option<T> {
        self.replace_full(value).1
    }

    /// Adds a value to the set, replacing the existing value, if any, that is equal to the given one.
    /// Returns the index of the item and its replaced value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let mut set = HashSlabSet::new();
    /// set.insert(Vec::<i32>::new());
    ///
    /// assert_eq!(set.get(&[][..]).unwrap().capacity(), 0);
    /// assert_eq!(set.replace_full(Vec::with_capacity(10)), (0, Some(vec![])));
    /// assert_eq!(set.get(&[][..]).unwrap().capacity(), 10);
    /// ```
    pub fn replace_full(&mut self, value: T) -> (usize, Option<T>) {
        let hash = self.map.builder.hash_one(&value);
        match self.map.table.entry(
            hash,
            |KeyData { key, .. }| key == &value,
            make_hasher(&self.map.builder),
        ) {
            hash_table::Entry::Occupied(mut occupied_entry) => {
                let KeyData { ref mut key, index } = occupied_entry.get_mut();
                (*index, Some(mem::replace(key, value)))
            }
            hash_table::Entry::Vacant(vacant_entry) => {
                let index = self.map.slab.insert(ValueData::new((), hash));
                vacant_entry.insert(KeyData::new(value, index));
                (index, None)
            }
        }
    }

    /// Visits the values representing the difference,
    /// i.e., the values that are in `self` but not in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let a: HashSlabSet<_> = [1, 2, 3].into_iter().collect();
    /// let b: HashSlabSet<_> = [4, 2, 3, 4].into_iter().collect();
    ///
    /// // Can be seen as `a - b`.
    /// for x in a.difference(&b) {
    ///     println!("{}", x); // Print 1
    /// }
    ///
    /// let diff: HashSlabSet<_> = a.difference(&b).collect();
    /// assert_eq!(diff, [1].iter().collect::<HashSlabSet<_>>());
    ///
    /// // Note that difference is not symmetric,
    /// // and `b - a` means something else:
    /// let diff: HashSlabSet<_> = b.difference(&a).collect();
    /// assert_eq!(diff, [4].iter().collect::<HashSlabSet<_>>());
    /// ```
    pub fn difference<'a, S2>(&'a self, other: &'a HashSlabSet<T, S2>) -> Difference<'a, T, S2> {
        Difference::new(self.iter(), other)
    }

    /// Visits the values representing the symmetric difference,
    /// i.e., the values that are in `self` or in `other` but not in both.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let a: HashSlabSet<_> = [1, 2, 3].into_iter().collect();
    /// let b: HashSlabSet<_> = [4, 2, 3, 4].into_iter().collect();
    ///
    /// // Print 1, 4 in arbitrary order.
    /// for x in a.symmetric_difference(&b) {
    ///     println!("{}", x);
    /// }
    ///
    /// let diff1: HashSlabSet<_> = a.symmetric_difference(&b).collect();
    /// let diff2: HashSlabSet<_> = b.symmetric_difference(&a).collect();
    ///
    /// assert_eq!(diff1, diff2);
    /// assert_eq!(diff1, [1, 4].iter().collect::<HashSlabSet<_>>());
    /// ```
    pub fn symmetric_difference<'a, S2>(
        &'a self,
        other: &'a HashSlabSet<T, S2>,
    ) -> SymmetricDifference<'a, T, S, S2>
    where
        S2: BuildHasher,
    {
        SymmetricDifference::new(self, other)
    }

    /// Visits the values representing the intersection,
    /// i.e., the values that are both in `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let a: HashSlabSet<_> = [1, 2, 3].into_iter().collect();
    /// let b: HashSlabSet<_> = [4, 2, 3, 4].into_iter().collect();
    ///
    /// // Print 2, 3 in arbitrary order.
    /// for x in a.intersection(&b) {
    ///     println!("{}", x);
    /// }
    ///
    /// let intersection: HashSlabSet<_> = a.intersection(&b).collect();
    /// assert_eq!(intersection, [2, 3].iter().collect::<HashSlabSet<_>>());
    /// ```
    pub fn intersection<'a, S2>(&'a self, other: &'a HashSlabSet<T, S2>) -> Intersection<'a, T, S2>
    where
        S2: BuildHasher,
    {
        Intersection::new(self, other)
    }

    /// Visits the values representing the union,
    /// i.e., all the values in `self` or `other`, without duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let a: HashSlabSet<_> = [1, 2, 3].into_iter().collect();
    /// let b: HashSlabSet<_> = [4, 2, 3, 4].into_iter().collect();
    ///
    /// // Print 1, 2, 3, 4 in arbitrary order.
    /// for x in a.union(&b) {
    ///     println!("{}", x);
    /// }
    ///
    /// let union: HashSlabSet<_> = a.union(&b).collect();
    /// assert_eq!(union, [1, 2, 3, 4].iter().collect::<HashSlabSet<_>>());
    /// ```
    pub fn union<'a, S2>(&'a self, other: &'a HashSlabSet<T, S2>) -> Union<'a, T, S>
    where
        S2: BuildHasher,
    {
        Union::new(self, other)
    }

    /// Moves all values from `other` into `self`, leaving `other` empty.
    ///
    /// This is equivalent to calling [`insert`][Self::insert] for each value
    /// from `other` in order, which means that values that already exist
    /// in `self` are unchanged in their current position.
    ///
    /// See also [`union`][Self::union] to iterate the combined values by
    /// reference, without modifying `self` or `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashslab::HashSlabSet;
    ///
    /// let mut a = HashSlabSet::from([1, 2, 3]);
    /// let mut b = HashSlabSet::from([3, 4, 5]);
    /// let old_capacity = b.capacity();
    ///
    /// a.append(&mut b);
    ///
    /// assert_eq!(a.len(), 5);
    /// assert_eq!(b.len(), 0);
    /// assert_eq!(b.capacity(), old_capacity);
    ///
    /// let mut sorted: Vec<_> = a.into_iter().collect();
    /// sorted.sort();
    /// assert_eq!(sorted, vec![1, 2, 3, 4, 5]);
    /// ```
    pub fn append<S2>(&mut self, other: &mut HashSlabSet<T, S2>) {
        self.map.append(&mut other.map);
    }

    /// Removes a value from the set. Returns whether the value was
    /// present in the set.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    ///
    /// let mut set = HashSlabSet::new();
    ///
    /// set.insert(2);
    /// assert_eq!(set.remove(&2), true);
    /// assert_eq!(set.remove(&2), false);
    /// ```
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.remove_full(value).is_some()
    }

    /// Remove the value from the set return it and the index it had.
    /// Return `None` if value was not in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let mut set = HashSlabSet::new();
    ///
    /// set.insert("A");
    /// assert_eq!(set.remove_full(&"A"), Some((0, "A")));
    /// assert_eq!(set.remove_full(&"A"), None);
    /// ```
    pub fn remove_full<Q>(&mut self, value: &Q) -> Option<(usize, T)>
    where
        Q: ?Sized + Hash + Equivalent<T>,
    {
        self.map.remove_full(value).map(|(i, x, ())| (i, x))
    }

    /// Remove the value by index
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let mut set = HashSlabSet::new();
    ///
    /// set.insert("A");
    /// assert_eq!(set.remove_index(0), Some("A"));
    /// assert_eq!(set.remove_index(0), None);
    /// ```
    pub fn remove_index(&mut self, index: usize) -> Option<T> {
        self.map.remove_index(index).map(|(x, ())| x)
    }

    /// Returns `true` if the set contains a value.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let set: HashSlabSet<_> = [1, 2, 3].into_iter().collect();
    /// assert_eq!(set.contains(&1), true);
    /// assert_eq!(set.contains(&4), false);
    /// ```
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.map.contains_key(value)
    }

    /// Returns a reference to the value in the set, if any, that is equal to the given value.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let set: HashSlabSet<_> = [1, 2, 3].into_iter().collect();
    /// assert_eq!(set.get(&2), Some(&2));
    /// assert_eq!(set.get(&4), None);
    /// ```
    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        Q: ?Sized + Hash + Equivalent<T>,
    {
        self.map.get_key_value(value).map(|(x, &())| x)
    }

    /// Return item index and value
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let set: HashSlabSet<_> = [1, 2, 3].into_iter().collect();
    /// assert_eq!(set.get_full(&2), Some((1, &2)));
    /// assert_eq!(set.get_full(&4), None);
    /// ```
    pub fn get_full<Q>(&self, value: &Q) -> Option<(usize, &T)>
    where
        Q: ?Sized + Hash + Equivalent<T>,
    {
        self.map.get_full(value).map(|(i, x, &())| (i, x))
    }

    /// Return item index, if it exists in the set
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let set: HashSlabSet<_> = [1, 2, 3].into_iter().collect();
    /// assert_eq!(set.get_index_of(&2), Some(1));
    /// assert_eq!(set.get_index_of(&4), None);
    /// ```
    pub fn get_index_of<Q>(&self, value: &Q) -> Option<usize>
    where
        Q: ?Sized + Hash + Equivalent<T>,
    {
        self.map.get_index_of(value)
    }

    /// Removes and returns the value in the set, if any, that is equal to the given one.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    ///
    /// let mut set: HashSlabSet<_> = [1, 2, 3].into_iter().collect();
    /// assert_eq!(set.take(&2), Some(2));
    /// assert_eq!(set.take(&2), None);
    /// ```
    pub fn take<Q>(&mut self, value: &Q) -> Option<T>
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.map.remove_entry(value).map(|(k, _)| k)
    }

    /// Returns `true` if the set is a subset of another,
    /// i.e., `other` contains at least all the values in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let sup: HashSlabSet<_> = [1, 2, 3].into_iter().collect();
    /// let mut set = HashSlabSet::new();
    ///
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(2);
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(4);
    /// assert_eq!(set.is_subset(&sup), false);
    /// ```
    pub fn is_subset<S2>(&self, other: &HashSlabSet<T, S2>) -> bool
    where
        S2: BuildHasher,
    {
        self.len() <= other.len() && self.iter().all(move |value| other.contains(value))
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let a: HashSlabSet<_> = [1, 2, 3].into_iter().collect();
    /// let mut b = HashSlabSet::new();
    ///
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(4);
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(1);
    /// assert_eq!(a.is_disjoint(&b), false);
    /// ```
    pub fn is_disjoint<S2>(&self, other: &HashSlabSet<T, S2>) -> bool
    where
        S2: BuildHasher,
    {
        if self.len() <= other.len() {
            self.iter().all(move |value| !other.contains(value))
        } else {
            other.iter().all(move |value| !self.contains(value))
        }
    }

    /// Returns `true` if the set is a superset of another,
    /// i.e., `self` contains at least all the values in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let sub: HashSlabSet<_> = [1, 2].into_iter().collect();
    /// let mut set = HashSlabSet::new();
    ///
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(0);
    /// set.insert(1);
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(2);
    /// assert_eq!(set.is_superset(&sub), true);
    /// ```
    pub fn is_superset<S2>(&self, other: &HashSlabSet<T, S2>) -> bool
    where
        S2: BuildHasher,
    {
        other.is_subset(self)
    }
}

impl<T, S> HashSlabSet<T, S> {
    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&e)` returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashslab::HashSlabSet;
    ///
    /// let xs = [1,2,3,4,5,6];
    /// let mut set: HashSlabSet<i32> = xs.into_iter().collect();
    /// set.retain(|&k| k % 2 == 0);
    /// assert_eq!(set.len(), 3);
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.map.retain(|k, _| f(k));
    }

    /// Get a value by index
    ///
    /// # Examples
    ///
    /// ```
    /// # use hashslab::HashSlabSet;
    /// let mut set = HashSlabSet::new();
    /// set.insert("A");
    /// assert_eq!(set.get_index(0), Some(&"A"));
    /// assert_eq!(set.get_index(1), None);
    /// ```
    pub fn get_index(&self, index: usize) -> Option<&T> {
        self.map.get_index(index).map(|(k, _)| k)
    }
}

impl<T, S> fmt::Debug for HashSlabSet<T, S>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T, S> Clone for HashSlabSet<T, S>
where
    T: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        HashSlabSet {
            map: self.map.clone(),
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.map.clone_from(&other.map);
    }
}

/// Access [`HashSlabSet`] values at indexed positions.
///
/// # Examples
///
/// ```
/// use hashslab::HashSlabSet;
///
/// let mut set = HashSlabSet::new();
/// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
///     set.insert(word.to_string());
/// }
/// assert_eq!(set[0], "Lorem");
/// assert_eq!(set[1], "ipsum");
/// ```
///
/// ```should_panic
/// use hashslab::HashSlabSet;
///
/// let mut set = HashSlabSet::new();
/// set.insert("foo");
/// println!("{:?}", set[10]); // panics!
/// ```
impl<T, S> Index<usize> for HashSlabSet<T, S> {
    type Output = T;

    /// Returns a reference to the value at the supplied `index`.
    ///
    /// ***Panics*** if `index` is out of bounds.
    fn index(&self, index: usize) -> &T {
        self.get_index(index)
            .expect("HashSlabSet: index out of bounds")
    }
}

impl<T, S> FromIterator<T> for HashSlabSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher + Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iterable: I) -> Self {
        let iter = iterable.into_iter().map(|x| (x, ()));
        HashSlabSet {
            map: HashSlabMap::from_iter(iter),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T, const N: usize> From<[T; N]> for HashSlabSet<T, RandomState>
where
    T: Eq + Hash,
{
    /// # Examples
    ///
    /// ```
    /// use hashslab::HashSlabSet;
    ///
    /// let set1 = HashSlabSet::from([1, 2, 3, 4]);
    /// let set2: HashSlabSet<_> = [1, 2, 3, 4].into();
    /// assert_eq!(set1, set2);
    /// ```
    fn from(arr: [T; N]) -> Self {
        Self::from_iter(arr)
    }
}

impl<T, S> Extend<T> for HashSlabSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iterable: I) {
        let iter = iterable.into_iter().map(|x| (x, ()));
        self.map.extend(iter);
    }
}

impl<'a, T, S> Extend<&'a T> for HashSlabSet<T, S>
where
    T: Hash + Eq + Copy + 'a,
    S: BuildHasher,
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iterable: I) {
        let iter = iterable.into_iter().copied();
        self.extend(iter);
    }
}

impl<T, S> Default for HashSlabSet<T, S>
where
    S: Default,
{
    /// Return an empty [`HashSlabSet`]
    fn default() -> Self {
        HashSlabSet {
            map: HashSlabMap::default(),
        }
    }
}

impl<T, S1, S2> PartialEq<HashSlabSet<T, S2>> for HashSlabSet<T, S1>
where
    T: Hash + Eq,
    S1: BuildHasher,
    S2: BuildHasher,
{
    fn eq(&self, other: &HashSlabSet<T, S2>) -> bool {
        self.len() == other.len() && self.is_subset(other)
    }
}

impl<T, S> Eq for HashSlabSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
}

impl<T, S1, S2> BitAnd<&HashSlabSet<T, S2>> for &HashSlabSet<T, S1>
where
    T: Eq + Hash + Clone,
    S1: BuildHasher + Default,
    S2: BuildHasher,
{
    type Output = HashSlabSet<T, S1>;

    /// Returns the set intersection, cloned into a new set.
    ///
    /// Values are collected in the same order that they appear in `self`.
    fn bitand(self, other: &HashSlabSet<T, S2>) -> Self::Output {
        self.intersection(other).cloned().collect()
    }
}

impl<T, S1, S2> BitOr<&HashSlabSet<T, S2>> for &HashSlabSet<T, S1>
where
    T: Eq + Hash + Clone,
    S1: BuildHasher + Default,
    S2: BuildHasher,
{
    type Output = HashSlabSet<T, S1>;

    /// Returns the set union, cloned into a new set.
    ///
    /// Values from `self` are collected in their original order, followed by
    /// values that are unique to `other` in their original order.
    fn bitor(self, other: &HashSlabSet<T, S2>) -> Self::Output {
        self.union(other).cloned().collect()
    }
}

impl<T, S1, S2> BitXor<&HashSlabSet<T, S2>> for &HashSlabSet<T, S1>
where
    T: Eq + Hash + Clone,
    S1: BuildHasher + Default,
    S2: BuildHasher,
{
    type Output = HashSlabSet<T, S1>;

    /// Returns the set symmetric-difference, cloned into a new set.
    ///
    /// Values from `self` are collected in their original order, followed by
    /// values from `other` in their original order.
    fn bitxor(self, other: &HashSlabSet<T, S2>) -> Self::Output {
        self.symmetric_difference(other).cloned().collect()
    }
}

impl<T, S1, S2> Sub<&HashSlabSet<T, S2>> for &HashSlabSet<T, S1>
where
    T: Eq + Hash + Clone,
    S1: BuildHasher + Default,
    S2: BuildHasher,
{
    type Output = HashSlabSet<T, S1>;

    /// Returns the set difference, cloned into a new set.
    ///
    /// Values are collected in the same order that they appear in `self`.
    fn sub(self, other: &HashSlabSet<T, S2>) -> Self::Output {
        self.difference(other).cloned().collect()
    }
}

impl<T, S> From<HashSlabMap<T, (), S>> for HashSlabSet<T, S> {
    fn from(map: HashSlabMap<T, (), S>) -> Self {
        Self { map }
    }
}
