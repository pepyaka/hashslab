#![doc = include_str!("../README.md")]

use std::hash::{BuildHasher, Hash, Hasher, RandomState};

use hashbrown::Equivalent;
use thiserror::Error;

pub mod map;
pub use map::HashSlabMap;

// Hasher wrapper. First write to set hasher state
#[derive(Debug, Default, Clone)]
struct HashSlabHasher<H> {
    hasher: H,
    state: HashSlabHasherState,
}

impl<H: Hasher> HashSlabHasher<H> {
    fn new(hasher: H) -> Self {
        Self {
            hasher,
            state: HashSlabHasherState::Init,
        }
    }
}

impl<H: Hasher> Hasher for HashSlabHasher<H> {
    fn finish(&self) -> u64 {
        match self.state {
            HashSlabHasherState::Init => {
                unreachable!(".finish() shouldn't be call on `Init` state")
            }
            HashSlabHasherState::Hasher => self.hasher.finish(),
            HashSlabHasherState::Finished(hash_value) => hash_value,
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        match (&self.state, bytes) {
            (HashSlabHasherState::Init, &[]) => {
                self.state = HashSlabHasherState::Hasher;
            }
            (HashSlabHasherState::Init, &[a, b, c, d, e, f, g, h]) => {
                let hash_value = u64::from_ne_bytes([a, b, c, d, e, f, g, h]);
                self.state = HashSlabHasherState::Finished(hash_value);
            }
            (HashSlabHasherState::Init, _) => {
                unreachable!("init write should be or `[]` either `[u8; 8]`")
            }
            (HashSlabHasherState::Hasher, bytes) => {
                self.hasher.write(bytes);
            }
            (HashSlabHasherState::Finished(_), _) => {}
        }
    }
}

#[derive(Debug, Default, Clone)]
enum HashSlabHasherState {
    #[default]
    Init,
    Hasher,
    Finished(u64),
}

// Hasher Builder wrapper
#[derive(Debug, Clone, Default)]
struct HashSlabHasherBuilder<S = RandomState>(S);

impl<S: BuildHasher> BuildHasher for HashSlabHasherBuilder<S> {
    type Hasher = HashSlabHasher<S::Hasher>;

    fn build_hasher(&self) -> Self::Hasher {
        let hasher = self.0.build_hasher();
        HashSlabHasher::new(hasher)
    }
}

// Key wrapper. Can't implement [Equivalnet] without it
#[derive(Debug)]
struct KeyQuery<Q>(Q);

impl<K: Hash> Hash for KeyQuery<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Set `Hasher` state with `[]` initial write
        state.write(&[]);
        self.0.hash(state);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct KeyEntry<K> {
    key: K,
    hash_value: u64,
    index: usize,
}

impl<K> KeyEntry<K> {
    fn new(key: K, hash_value: u64, index: usize) -> Self {
        Self {
            key,
            hash_value,
            index,
        }
    }
}

impl<K: Hash> Hash for KeyEntry<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Set `Finished` state with `[u8; 8]` initial write
        state.write_u64(self.hash_value);
    }
}

impl<'q, Q: ?Sized, K> Equivalent<KeyEntry<K>> for KeyQuery<&'q Q>
where
    Q: Equivalent<K>,
{
    #[inline]
    fn equivalent(&self, entry: &KeyEntry<K>) -> bool {
        self.0.equivalent(&entry.key)
    }
}

// Key getted from slab
#[derive(Debug, Clone)]
struct RawHash {
    hash_value: u64,
    index: usize,
}

impl RawHash {
    fn new(hash_value: u64, index: usize) -> Self {
        Self { hash_value, index }
    }
}

impl Hash for RawHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Set `Finished` state with `[u8; 8]` initial write
        state.write_u64(self.hash_value);
    }
}

// HashMap entry always has uniq index
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

#[derive(Debug, Clone)]
struct EntryBuilder<K> {
    key: K,
    hash_value: u64,
}

impl<K: Hash> EntryBuilder<K> {
    fn new<S: BuildHasher>(key: K, hasher_builder: &HashSlabHasherBuilder<S>) -> Self {
        let mut hasher = hasher_builder.0.build_hasher();
        key.hash(&mut hasher);
        Self {
            key,
            hash_value: hasher.finish(),
        }
    }

    fn value_entry<T>(&self, value: T) -> ValueEntry<T> {
        ValueEntry {
            hash_value: self.hash_value,
            data: value,
        }
    }

    fn key_entry(self, index: usize) -> KeyEntry<K> {
        KeyEntry::new(self.key, self.hash_value, index)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Borrow,
        hash::{DefaultHasher, RandomState},
    };

    use super::*;
    use hashbrown::{Equivalent, HashMap, HashSet};

    #[test]
    fn hashslab_hasher_state_as_wrapper() {
        let mut hasher = HashSlabHasher::new(DefaultHasher::new());
        assert!(matches!(hasher.state, HashSlabHasherState::Init));

        hasher.write(&[]);
        assert!(matches!(hasher.state, HashSlabHasherState::Hasher));

        (()).hash(&mut hasher);
        assert!(matches!(hasher.state, HashSlabHasherState::Hasher));

        let _ = hasher.finish();
        assert!(matches!(hasher.state, HashSlabHasherState::Hasher));
    }

    #[test]
    fn hashslab_hasher_state_direct_write() {
        let mut hasher = HashSlabHasher::new(DefaultHasher::new());
        assert!(matches!(hasher.state, HashSlabHasherState::Init));

        hasher.write_u64(42);
        assert!(matches!(hasher.state, HashSlabHasherState::Finished(42)));

        (()).hash(&mut hasher);
        assert!(matches!(hasher.state, HashSlabHasherState::Finished(42)));

        assert_eq!(42, hasher.finish());
        assert!(matches!(hasher.state, HashSlabHasherState::Finished(42)));
    }

    #[test]
    #[should_panic]
    fn hashslab_hasher_state_unreachable() {
        let mut hasher = HashSlabHasher::new(DefaultHasher::new());
        assert!(matches!(hasher.state, HashSlabHasherState::Init));

        hasher.write(&[1, 2, 3]);
        assert!(matches!(hasher.state, HashSlabHasherState::Finished(42)));
    }

    #[test]
    fn hashslab_hasher_transparency() {
        let mut default_hasher = DefaultHasher::new();
        let mut hashslab_hasher = HashSlabHasher {
            hasher: default_hasher.clone(),
            state: HashSlabHasherState::Hasher,
        };

        (()).hash(&mut default_hasher);
        (()).hash(&mut hashslab_hasher);

        assert_eq!(default_hasher.finish(), hashslab_hasher.finish());
    }

    #[test]
    fn entry_builder() {
        let random_state = RandomState::new();
        let hasher_builder = HashSlabHasherBuilder(random_state.clone());
        let entry_builder = EntryBuilder::new(3u8, &hasher_builder);

        let value_entry = entry_builder.value_entry(());
        let mut hasher = random_state.build_hasher();
        3u8.hash(&mut hasher);
        assert_eq!(value_entry.hash_value, hasher.finish());

        let key_entry = entry_builder.key_entry(555);
        let mut hasher = random_state.build_hasher();
        3u8.hash(&mut hasher);
        assert_eq!(key_entry.hash_value, hasher.finish());
    }

    #[test]
    fn hashmap_with_hasher_builder() {
        let hasher_builder = HashSlabHasherBuilder(RandomState::new());
        let map: HashMap<(), (), _> = HashMap::with_hasher(hasher_builder);
        assert_eq!(map.len(), 0);
    }

    // equivalence: KeyQuery & KeyEntry
    #[test]
    fn key_query_equivalence_well_known_types() {
        let state = <HashSlabHasherBuilder>::default();

        let query = KeyQuery(&());
        let entry = EntryBuilder::new((), &state).key_entry(0);
        assert!(query.equivalent(&entry));

        let query = KeyQuery("hello");
        let entry = EntryBuilder::new(String::from("hello"), &state).key_entry(0);
        assert!(query.equivalent(&entry));

        let query = KeyQuery(&11);
        let entry = EntryBuilder::new(12, &state).key_entry(0);
        assert!(!query.equivalent(&entry));

        let vec = Vec::new();
        let query = KeyQuery(&vec);
        let entry = EntryBuilder::new(vec![42], &state).key_entry(0);
        assert!(!query.equivalent(&entry));
    }

    // equivalence: RawHash & KeyEntry
    #[test]
    fn raw_hash_equivalence_well_known_types() {
        let state = <HashSlabHasherBuilder>::default();
        // RawHash vs KeyEntry
        assert!(RawHash {
            index: 0,
            hash_value: 42
        }
        .equivalent(&EntryBuilder::new((), &state).key_entry(0)));

        assert!(RawHash {
            index: 0,
            hash_value: 42
        }
        .equivalent(&EntryBuilder::new(String::from("A"), &state).key_entry(0)));

        assert!(!RawHash {
            index: 0,
            hash_value: 42
        }
        .equivalent(&EntryBuilder::new((), &state).key_entry(1)));

        assert!(!RawHash {
            index: 0,
            hash_value: 42
        }
        .equivalent(&EntryBuilder::new(String::from("A"), &state).key_entry(1)));
    }

    // equivalence: KeyQuery & KeyEntry
    #[test]
    fn key_query_equivalence_custom_types() {
        #[derive(Debug, PartialEq, Eq, Hash)]
        enum MyQuery {
            One,
            Two,
        }

        #[derive(Debug, PartialEq, Eq, Hash)]
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

        let state = <HashSlabHasherBuilder>::default();

        let query = KeyQuery(&MyQuery::One);
        let entry = EntryBuilder::new(MyKey::One, &state).key_entry(0);
        assert!(query.equivalent(&entry));

        let query = KeyQuery(&MyKey::One);
        let entry = EntryBuilder::new(MyKey::One, &state).key_entry(0);
        assert!(query.equivalent(&entry));

        let query = KeyQuery(&MyQuery::One);
        let entry = EntryBuilder::new(MyKey::Two, &state).key_entry(0);
        assert!(!query.equivalent(&entry));

        let query = KeyQuery(&MyKey::One);
        let entry = EntryBuilder::new(MyKey::Two, &state).key_entry(0);
        assert!(!query.equivalent(&entry));
    }

    #[cfg(test)]
    mod hash_equality {
        use super::*;
        // KeyQuery & KeyEntry hashes should be equal with HashSlabHasher, but unequal for others
        #[cfg(test)]
        mod key_entry_vs_key_query {
            use super::*;

            #[test]
            fn default_hasher() {
                let builder = RandomState::new();

                let key = String::from("key");
                let query = KeyQuery("key");
                let entry =
                    EntryBuilder::new(key, &HashSlabHasherBuilder(builder.clone())).key_entry(666);

                // DefaultHasher
                let mut query_hasher = builder.build_hasher();
                let mut entry_hasher = builder.build_hasher();

                query.hash(&mut query_hasher);
                entry.hash(&mut entry_hasher);

                assert_ne!(query_hasher.finish(), entry_hasher.finish());

                // HashSlabHasher based on RandomHahser
                let builder = HashSlabHasherBuilder(builder);

                let mut query_hasher = builder.build_hasher();
                let mut entry_hasher = builder.build_hasher();

                query.hash(&mut query_hasher);
                entry.hash(&mut entry_hasher);

                assert_eq!(query_hasher.finish(), entry_hasher.finish());
            }

            #[test]
            fn fnv_hasher() {
                let map = HashMap::<(), (), fnv::FnvBuildHasher>::default();
                let builder = map.hasher();

                let key = String::from("key");
                let query = KeyQuery("key");
                let entry =
                    EntryBuilder::new(key, &HashSlabHasherBuilder(builder.clone())).key_entry(666);

                // DefaultHasher
                let mut query_hasher = builder.build_hasher();
                let mut entry_hasher = builder.build_hasher();

                query.hash(&mut query_hasher);
                entry.hash(&mut entry_hasher);

                assert_ne!(query_hasher.finish(), entry_hasher.finish());

                // HashSlabHasher based on RandomHahser
                let builder = HashSlabHasherBuilder(builder.clone());

                let mut query_hasher = builder.build_hasher();
                let mut entry_hasher = builder.build_hasher();

                query.hash(&mut query_hasher);
                entry.hash(&mut entry_hasher);

                assert_eq!(query_hasher.finish(), entry_hasher.finish());
            }
        }

        // RawHash vs KeyEntry should equal with any hasher
        mod key_entry_vs_raw_hash {
            use super::*;

            #[test]
            fn default_hasher() {
                let builder = RandomState::new();

                let entry_builder =
                    EntryBuilder::new(vec![1, 2, 3], &HashSlabHasherBuilder(builder.clone()));
                let raw_hash = RawHash::new(entry_builder.value_entry(()).hash_value, 0);
                let entry = entry_builder.key_entry(666);

                // DefaultHasher
                let mut raw_hash_hasher = builder.build_hasher();
                let mut entry_hasher = builder.build_hasher();

                raw_hash.hash(&mut raw_hash_hasher);
                entry.hash(&mut entry_hasher);

                assert_eq!(raw_hash_hasher.finish(), entry_hasher.finish());

                // HashSlabHasher based on RandomHahser
                let builder = HashSlabHasherBuilder(builder);

                let mut raw_hash_hasher = builder.build_hasher();
                let mut entry_hasher = builder.build_hasher();

                raw_hash.hash(&mut raw_hash_hasher);
                entry.hash(&mut entry_hasher);

                assert_eq!(raw_hash_hasher.finish(), entry_hasher.finish());
            }
            #[test]
            fn fnv_hasher() {
                let map = HashMap::<(), (), fnv::FnvBuildHasher>::default();
                let builder = map.hasher();

                let entry_builder =
                    EntryBuilder::new(vec![1, 2, 3], &HashSlabHasherBuilder(builder.clone()));
                let raw_hash = RawHash::new(entry_builder.value_entry(()).hash_value, 0);
                let entry = entry_builder.key_entry(666);

                // DefaultHasher
                let mut raw_hash_hasher = builder.build_hasher();
                let mut entry_hasher = builder.build_hasher();

                raw_hash.hash(&mut raw_hash_hasher);
                entry.hash(&mut entry_hasher);

                assert_eq!(raw_hash_hasher.finish(), entry_hasher.finish());

                // HashSlabHasher based on RandomHahser
                let builder = HashSlabHasherBuilder(builder.clone());

                let mut raw_hash_hasher = builder.build_hasher();
                let mut entry_hasher = builder.build_hasher();

                raw_hash.hash(&mut raw_hash_hasher);
                entry.hash(&mut entry_hasher);

                assert_eq!(raw_hash_hasher.finish(), entry_hasher.finish());
            }
        }
    }

    #[test]
    fn hashset_contains_search_key() {
        let index = 333;

        let hasher_buider = HashSlabHasherBuilder(RandomState::new());
        let mut hs = HashSet::with_hasher(hasher_buider);

        let state = hs.hasher();

        let eq_key_entry = EntryBuilder::new(String::from("EQ"), state).key_entry(index);
        let ne_key_entry = EntryBuilder::new(String::from("NE"), state).key_entry(index);

        let search_key = KeyQuery("EQ");
        let key_entry = EntryBuilder::new(String::from("EQ"), state).key_entry(index);
        hs.insert(key_entry);

        assert_eq!(Some(&eq_key_entry), hs.get(&search_key));

        assert_ne!(Some(&ne_key_entry), hs.get(&search_key));
    }

    #[test]
    fn hashset_contains_raw_hash() {
        let index = 4444;

        let hasher_buider = HashSlabHasherBuilder(RandomState::new());
        let mut hs = HashSet::with_hasher(hasher_buider);

        let state = hs.hasher();

        let eq_key_entry = EntryBuilder::new(String::from("EQ"), state).key_entry(index);
        let ne_key_entry = EntryBuilder::new(String::from("NE"), state).key_entry(index);

        let builder = EntryBuilder::new(String::from("EQ"), state);
        let raw_hash = {
            let ValueEntry { hash_value, .. } = builder.value_entry(());
            RawHash::new(hash_value, index)
        };

        let key_entry = builder.key_entry(index);
        hs.insert(key_entry);

        assert_eq!(Some(&eq_key_entry), hs.get(&raw_hash));

        assert_ne!(Some(&ne_key_entry), hs.get(&raw_hash));
    }
}
