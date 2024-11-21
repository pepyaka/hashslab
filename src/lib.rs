#![doc = include_str!("../README.md")]

use std::hash::{BuildHasher, Hash, Hasher, RandomState};

use hashbrown::Equivalent;
use thiserror::Error;

pub mod map;
pub use map::HashSlabMap;

// Hasher wrapper
#[derive(Debug, Default, Clone)]
struct DirectAssignmentHasher<H> {
    hasher: H,
    finish: Option<u64>,
}

impl<H: Hasher> DirectAssignmentHasher<H> {
    fn new(hasher: H) -> Self {
        Self {
            hasher,
            finish: None,
        }
    }
}

impl<H: Hasher> Hasher for DirectAssignmentHasher<H> {
    fn finish(&self) -> u64 {
        self.finish.unwrap_or_else(|| self.hasher.finish())
    }

    fn write(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes);
    }

    // This is hack point
    fn write_u64(&mut self, i: u64) {
        self.finish = Some(i);
    }
}

// Hasher Builder wrapper => [RandomState] wrapper
#[derive(Debug, Default, Clone)]
struct DirectAssignmentHasherBuilder<S = RandomState>(S);

impl<S> DirectAssignmentHasherBuilder<S> {
    fn new(hasher_builder: S) -> Self {
        Self(hasher_builder)
    }
}

impl<S: BuildHasher> BuildHasher for DirectAssignmentHasherBuilder<S> {
    type Hasher = DirectAssignmentHasher<S::Hasher>;

    fn build_hasher(&self) -> Self::Hasher {
        let hasher = self.0.build_hasher();
        DirectAssignmentHasher::new(hasher)
    }
}

// Key wrapper. Can't implement [Equivalnet] without it
#[derive(Debug, Hash)]
struct Query<K>(K);

#[derive(Debug, PartialEq, Eq, Clone)]
struct KeyEntry<K> {
    index: usize,
    key: K,
}

impl<K: Hash> Hash for KeyEntry<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

// Key getted from slab
#[derive(Debug, Clone)]
struct RawHash {
    index: usize,
    value: u64,
}

impl Hash for RawHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.value);
    }
}

impl<'q, Q: ?Sized, K> Equivalent<KeyEntry<K>> for Query<&'q Q>
where
    Q: Equivalent<K>,
{
    #[inline]
    fn equivalent(&self, entry: &KeyEntry<K>) -> bool {
        self.0.equivalent(&entry.key)
    }
}

impl<K> Equivalent<KeyEntry<K>> for RawHash {
    #[inline]
    fn equivalent(&self, entry: &KeyEntry<K>) -> bool {
        self.index == entry.index
    }
}

#[derive(Debug, Clone)]
struct ValueEntry<T> {
    hash_value: u64,
    data: T,
}

/// The error type for [`try_reserve`][HashSlabMap::try_reserve] methods.
#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum TryReserveError {
    #[error("Error due to the computed capacity exceeding the collection's maximum (usually `isize::MAX` bytes)")]
    HashSetCapacityOverflow,

    #[error("The memory allocator returned an error. The layout of allocation request that failed: {layout:?}")]
    HashSetAllocError { layout: std::alloc::Layout },

    #[error(
        "sum of current ({capacity}) and additional ({additional}) capacity exceeds isize::MAX"
    )]
    Slab { capacity: usize, additional: usize },
}

impl From<hashbrown::TryReserveError> for TryReserveError {
    fn from(err: hashbrown::TryReserveError) -> Self {
        match err {
            hashbrown::TryReserveError::CapacityOverflow => Self::HashSetCapacityOverflow,
            hashbrown::TryReserveError::AllocError { layout } => Self::HashSetAllocError { layout },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Borrow, hash::DefaultHasher};

    use super::*;
    use hashbrown::{Equivalent, HashMap, HashSet};

    #[test]
    fn hasher_transparency() {
        let mut default_hasher = DefaultHasher::new();
        let mut direct_assignment_hasher = DirectAssignmentHasher::new(default_hasher.clone());

        (()).hash(&mut default_hasher);
        (()).hash(&mut direct_assignment_hasher);

        assert_eq!(default_hasher.finish(), direct_assignment_hasher.finish());
    }

    #[test]
    fn hasher_direct_assignment() {
        let mut hasher = DirectAssignmentHasher::new(DefaultHasher::new());

        (()).hash(&mut hasher);
        hasher.write_u64(42);

        assert_eq!(42, hasher.finish());
    }

    #[test]
    fn hashmap_with_hasher_builder() {
        let hasher_builder = DirectAssignmentHasherBuilder::<RandomState>::default();
        let mut map = HashMap::with_hasher(hasher_builder);
        map.insert(String::from("A"), 42);

        assert_eq!(Some(&42), map.get("A"));
    }

    #[test]
    fn query_equivalence_well_known_types() {
        // Query vs HsEntry
        assert!(Query(&()).equivalent(&KeyEntry { index: 0, key: () }));

        assert!(Query("hello").equivalent(&KeyEntry {
            index: 0,
            key: String::from("hello")
        }));

        assert!(!Query(&11).equivalent(&KeyEntry { index: 0, key: 12 }));

        assert!(!Query([42].as_slice()).equivalent(&KeyEntry {
            index: 0,
            key: Vec::new()
        }));
    }

    #[test]
    fn raw_hash_equivalence_well_known_types() {
        // RawHash vs HsEntry
        assert!(RawHash {
            index: 0,
            value: 42
        }
        .equivalent(&KeyEntry { index: 0, key: () }));

        assert!(RawHash {
            index: 0,
            value: 42
        }
        .equivalent(&KeyEntry {
            index: 0,
            key: String::from("A")
        }));

        assert!(!RawHash {
            index: 0,
            value: 42
        }
        .equivalent(&KeyEntry { index: 1, key: () }));

        assert!(!RawHash {
            index: 0,
            value: 42
        }
        .equivalent(&KeyEntry {
            index: 1,
            key: String::from("A")
        }));
    }

    #[test]
    fn query_keys_equivalence_custom_types() {
        #[derive(Debug, PartialEq, Eq)]
        enum MyQuery {
            One,
            Two,
        }

        #[derive(Debug, PartialEq, Eq)]
        enum MyKey {
            One,
            Two,
        }

        impl Borrow<MyQuery> for MyKey {
            fn borrow(&self) -> &MyQuery {
                match self {
                    MyKey::One => &MyQuery::One,
                    MyKey::Two => &MyQuery::Two,
                }
            }
        }

        // Query vs HsEntry
        assert!(Query(&MyQuery::One).equivalent(&KeyEntry {
            index: 0,
            key: MyKey::One
        }));

        assert!(Query(&MyKey::One).equivalent(&KeyEntry {
            index: 0,
            key: MyKey::One
        }));

        assert!(!Query(&MyQuery::One).equivalent(&KeyEntry {
            index: 0,
            key: MyKey::Two
        }));

        assert!(!Query(&MyKey::One).equivalent(&KeyEntry {
            index: 0,
            key: MyKey::Two
        }));
    }

    #[test]
    fn hash_equality_hs_entry_vs_search_key() {
        // Query vs HsEntry should be equal with any hasher
        let hasher = DefaultHasher::new();
        let mut hs_entry_hasher = hasher.clone();
        let mut search_key_hasher = hasher.clone();

        let hs_entry = KeyEntry { index: 0, key: () };
        hs_entry.hash(&mut hs_entry_hasher);

        let search_key = Query(&());
        search_key.hash(&mut search_key_hasher);

        assert_eq!(search_key_hasher.finish(), hs_entry_hasher.finish());

        let hasher = DirectAssignmentHasher::new(hasher);
        let mut hs_entry_hasher = hasher.clone();
        let mut search_key_hasher = hasher.clone();

        let hs_entry = KeyEntry { index: 0, key: () };
        hs_entry.hash(&mut hs_entry_hasher);

        let search_key = Query(&());
        search_key.hash(&mut search_key_hasher);

        assert_eq!(hs_entry_hasher.finish(), search_key_hasher.finish());
    }

    #[test]
    fn hash_equality_hs_entry_vs_raw_hash() {
        // RawHash vs HsEntry should NOT be equal with default hasher
        let hasher = DefaultHasher::new();
        let mut hs_entry_hasher = hasher.clone();
        let mut raw_hash_hasher = hasher.clone();

        let hs_entry = KeyEntry { index: 0, key: () };
        hs_entry.hash(&mut hs_entry_hasher);
        let hs_entry_hash_value = hs_entry_hasher.finish();

        let raw_hash = RawHash {
            index: 42,
            value: hs_entry_hash_value,
        };
        raw_hash.hash(&mut raw_hash_hasher);

        assert_ne!(hs_entry_hash_value, raw_hash_hasher.finish());

        // And should be equal with custom hasher
        let hasher = DirectAssignmentHasher::new(hasher);
        let mut hs_entry_hasher = hasher.clone();
        let mut raw_hash_hasher = hasher.clone();

        let hs_entry = KeyEntry { index: 0, key: () };
        hs_entry.hash(&mut hs_entry_hasher);
        let hs_entry_hash_value = hs_entry_hasher.finish();

        let raw_hash = RawHash {
            index: 42,
            value: hs_entry_hash_value,
        };
        raw_hash.hash(&mut raw_hash_hasher);

        assert_eq!(hs_entry_hash_value, raw_hash_hasher.finish());
    }

    #[test]
    fn hashset_contains_search_key() {
        let hasher_buider = DirectAssignmentHasherBuilder::new(RandomState::new());
        let mut hs = HashSet::with_hasher(hasher_buider);

        let value = String::from("EQ");

        let hs_entry = KeyEntry {
            index: 0,
            key: value.clone(),
        };
        hs.insert(hs_entry);

        let search_key = Query(value.as_str());

        assert_eq!(
            Some(&KeyEntry {
                index: 0,
                key: String::from("EQ"),
            }),
            hs.get(&search_key)
        );

        assert_ne!(
            Some(&KeyEntry {
                index: 0,
                key: String::from("NE"),
            }),
            hs.get(&search_key)
        );
    }

    #[test]
    fn hashset_contains_raw_hash() {
        let value = String::from("EQ");

        let hasher_buider = DirectAssignmentHasherBuilder::new(RandomState::new());
        let mut hs = HashSet::with_hasher(hasher_buider);
        let hs_entry = KeyEntry {
            index: 0,
            key: value.clone(),
        };
        hs.insert(hs_entry);

        let mut hasher = hs.hasher().build_hasher();
        value.hash(&mut hasher);
        let hash_value = hasher.finish();
        let raw_hash = RawHash {
            index: 0,
            value: hash_value,
        };

        assert_eq!(
            Some(&KeyEntry {
                index: 0,
                key: String::from("EQ")
            }),
            hs.get(&raw_hash)
        );

        assert_ne!(
            Some(&KeyEntry {
                index: 0,
                key: String::from("NE"),
            }),
            hs.get(&raw_hash)
        );
    }
}
