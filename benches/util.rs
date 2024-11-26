use std::{collections::HashMap, sync::LazyLock};

use hashslab::HashSlabMap;
use indexmap::IndexMap;
use rand::{rngs::SmallRng, seq::SliceRandom as _, SeedableRng as _};

/// Use a consistently seeded Rng for benchmark stability
pub fn small_rng() -> SmallRng {
    let seed = u64::from_le_bytes(*b"indexmap");
    SmallRng::seed_from_u64(seed)
}

pub fn shuffled_keys<I>(iter: I) -> Vec<I::Item>
where
    I: IntoIterator,
{
    let mut v = Vec::from_iter(iter);
    let mut rng = small_rng();
    v.shuffle(&mut rng);
    v
}

pub const LOOKUP_MAP_SIZE: u32 = 100_000_u32;

pub static SHUFFLED_KEYS: LazyLock<Vec<u32>> = LazyLock::new(|| shuffled_keys(0..LOOKUP_MAP_SIZE));

pub static SHUFFLED_HASHMAP: LazyLock<HashMap<u32, u32>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(SHUFFLED_KEYS.len());
    let keys = &*SHUFFLED_KEYS;
    for &key in keys {
        map.insert(key, key);
    }
    map
});

pub static SHUFFLED_INDEXMAP: LazyLock<IndexMap<u32, u32>> = LazyLock::new(|| {
    let mut map = IndexMap::with_capacity(SHUFFLED_KEYS.len());
    let keys = &*SHUFFLED_KEYS;
    for &key in keys {
        map.insert(key, key);
    }
    map
});

pub static SHUFFLED_HASHSLABMAP: LazyLock<HashSlabMap<u32, u32>> = LazyLock::new(|| {
    let mut map = HashSlabMap::with_capacity(SHUFFLED_KEYS.len());
    let keys = &*SHUFFLED_KEYS;
    for &key in keys {
        map.insert(key, key);
    }
    map
});
