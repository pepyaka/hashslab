use std::{collections::HashMap, hash::Hash, sync::LazyLock};

use criterion::{
    black_box, criterion_group, criterion_main, measurement::Measurement, BenchmarkGroup,
    BenchmarkId, Criterion,
};
use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};

use hashslab::HashSlabMap;
use indexmap::IndexMap;

/// Use a consistently seeded Rng for benchmark stability
fn small_rng() -> SmallRng {
    let seed = u64::from_le_bytes(*b"indexmap");
    SmallRng::seed_from_u64(seed)
}

/*
    Init
*/
fn bench_new(c: &mut Criterion) {
    let mut group = c.benchmark_group("new");
    group.bench_function("hashmap", |b| b.iter(|| HashMap::<String, String>::new()));
    group.bench_function("indexmap", |b| b.iter(|| IndexMap::<String, String>::new()));
    group.bench_function("hashslabmap", |b| {
        b.iter(|| HashSlabMap::<String, String>::new())
    });
}

fn bench_with_capacity(c: &mut Criterion) {
    let mut group = c.benchmark_group("with_capacity");
    for cap in [1, 100, 10_000].iter() {
        group.bench_with_input(BenchmarkId::new("hashmap", cap), cap, |b, i| {
            b.iter(|| HashMap::<String, String>::with_capacity(*i))
        });
        group.bench_with_input(BenchmarkId::new("indexmap", cap), cap, |b, i| {
            b.iter(|| IndexMap::<String, String>::with_capacity(*i))
        });
        group.bench_with_input(BenchmarkId::new("hashslabmap", cap), cap, |b, i| {
            b.iter(|| HashSlabMap::<String, String>::with_capacity(*i))
        });
    }
}

/*
    Grow
*/
// Test grow/resize without preallocation
fn bench_grow(c: &mut Criterion) {
    let mut group = c.benchmark_group("grow");
    for grow_size in [1, 100, 10_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("hashmap", grow_size),
            grow_size,
            |b, grow_size| {
                b.iter(|| {
                    let mut map = HashMap::new();
                    for x in 0..*grow_size {
                        map.insert(x, x);
                    }
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("indexmap", grow_size),
            grow_size,
            |b, grow_size| {
                b.iter(|| {
                    let mut map = IndexMap::new();
                    for x in 0..*grow_size {
                        map.insert(x, x);
                    }
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("hashslabmap", grow_size),
            grow_size,
            |b, grow_size| {
                b.iter(|| {
                    let mut map = HashSlabMap::new();
                    for x in 0..*grow_size {
                        map.insert(x, x);
                    }
                })
            },
        );
    }
}

/*
    Insert
*/
fn insert<'a, M, K, V>(group: &mut BenchmarkGroup<'a, M>, name: &str, list: Vec<(K, V)>)
where
    M: Measurement,
    K: Hash + Eq,
{
    let len = list.len();
    group.bench_with_input(BenchmarkId::new("hashmap", name), &list, |b, list| {
        b.iter(|| {
            let mut map = HashMap::with_capacity(len);
            for (k, v) in list {
                map.insert(k, v);
            }
        })
    });
    group.bench_with_input(BenchmarkId::new("indexmap", name), &list, |b, list| {
        b.iter(|| {
            let mut map = IndexMap::with_capacity(len);
            for (k, v) in list {
                map.insert(k, v);
            }
        })
    });
    group.bench_with_input(BenchmarkId::new("hashslabmap", name), &list, |b, list| {
        b.iter(|| {
            let mut map = HashSlabMap::with_capacity(len);
            for (k, v) in list {
                map.insert(k, v);
            }
        })
    });
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");

    let list: Vec<(usize, ())> = (0..10_000).map(|n| (n, ())).collect();
    insert(&mut group, "usize", list);

    let list: Vec<(String, ())> = (0..10_000).map(|n| (n.to_string(), ())).collect();
    insert(&mut group, "string", list.clone());

    let list: Vec<(&str, ())> = list.iter().map(|(s, _)| (s.as_str(), ())).collect();
    insert(&mut group, "str", list);

    let list: Vec<(usize, [u64; 10])> = (0..10_000).map(|n| (n, Default::default())).collect();
    insert(&mut group, "bigint", list);
}

/*
    Key lookup sequential
*/
const LOOKUP_SEQ_SIZE: usize = 10_000;

fn bench_lookup_key_exist(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_key_exist");
    group.bench_function("hashmap", |b| {
        let mut map = HashMap::with_capacity(LOOKUP_SEQ_SIZE);
        let keys = shuffled_keys(0..LOOKUP_SEQ_SIZE);
        for &key in &keys {
            map.insert(key, 1);
        }
        b.iter(|| {
            for key in 5000..LOOKUP_SEQ_SIZE {
                map.get(&key);
            }
        });
    });
    group.bench_function("indexmap", |b| {
        let mut map = IndexMap::with_capacity(LOOKUP_SEQ_SIZE);
        let keys = shuffled_keys(0..LOOKUP_SEQ_SIZE);
        for &key in &keys {
            map.insert(key, 1);
        }
        b.iter(|| {
            for key in 5000..LOOKUP_SEQ_SIZE {
                map.get(&key);
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let mut map = HashSlabMap::with_capacity(LOOKUP_SEQ_SIZE);
        let keys = shuffled_keys(0..LOOKUP_SEQ_SIZE);
        for &key in &keys {
            map.insert(key, 1);
        }
        b.iter(|| {
            for key in 5000..LOOKUP_SEQ_SIZE {
                map.get(&key);
            }
        });
    });
}

fn bench_lookup_key_noexist(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_key_noexist");
    group.bench_function("hashmap", |b| {
        let mut map = HashMap::with_capacity(LOOKUP_SEQ_SIZE);
        let keys = shuffled_keys(0..LOOKUP_SEQ_SIZE);
        for &key in &keys {
            map.insert(key, 1);
        }
        b.iter(|| {
            for key in LOOKUP_SEQ_SIZE..15000 {
                map.get(&key);
            }
        });
    });
    group.bench_function("indexmap", |b| {
        let mut map = IndexMap::with_capacity(LOOKUP_SEQ_SIZE);
        let keys = shuffled_keys(0..LOOKUP_SEQ_SIZE);
        for &key in &keys {
            map.insert(key, 1);
        }
        b.iter(|| {
            for key in LOOKUP_SEQ_SIZE..15000 {
                map.get(&key);
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let mut map = HashSlabMap::with_capacity(LOOKUP_SEQ_SIZE);
        let keys = shuffled_keys(0..LOOKUP_SEQ_SIZE);
        for &key in &keys {
            map.insert(key, 1);
        }
        b.iter(|| {
            for key in LOOKUP_SEQ_SIZE..15000 {
                map.get(&key);
            }
        });
    });
}

/*
    Key lookup random
*/
fn shuffled_keys<I>(iter: I) -> Vec<I::Item>
where
    I: IntoIterator,
{
    let mut v = Vec::from_iter(iter);
    let mut rng = small_rng();
    v.shuffle(&mut rng);
    v
}

const LOOKUP_MAP_SIZE: u32 = 100_000_u32;

static LOOKUP_SHUFFLED_KEYS: LazyLock<Vec<u32>> =
    LazyLock::new(|| shuffled_keys(0..LOOKUP_MAP_SIZE));

static LOOKUP_HASHMAP: LazyLock<HashMap<u32, u32>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(LOOKUP_SHUFFLED_KEYS.len());
    let keys = &*LOOKUP_SHUFFLED_KEYS;
    for &key in keys {
        map.insert(key, key);
    }
    map
});

static LOOKUP_INDEXMAP: LazyLock<IndexMap<u32, u32>> = LazyLock::new(|| {
    let mut map = IndexMap::with_capacity(LOOKUP_SHUFFLED_KEYS.len());
    let keys = &*LOOKUP_SHUFFLED_KEYS;
    for &key in keys {
        map.insert(key, key);
    }
    map
});

static LOOKUP_HASHSLABMAP: LazyLock<HashSlabMap<u32, u32>> = LazyLock::new(|| {
    let mut map = HashSlabMap::with_capacity(LOOKUP_SHUFFLED_KEYS.len());
    let keys = &*LOOKUP_SHUFFLED_KEYS;
    for &key in keys {
        map.insert(key, key);
    }
    map
});

const LOOKUP_SAMPLE_SIZE: u32 = 5000;
// const SORT_MAP_SIZE: usize = 10_000;

fn bench_lookup_key_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_key_single");
    group.bench_function("hashmap", |b| {
        let map = &*LOOKUP_HASHMAP;
        let mut iter = (0..LOOKUP_MAP_SIZE + LOOKUP_SAMPLE_SIZE).cycle();
        b.iter(|| {
            let key = iter.next().unwrap();
            map.get(&key);
        });
    });
    group.bench_function("indexmap", |b| {
        let map = &*LOOKUP_INDEXMAP;
        let mut iter = (0..LOOKUP_MAP_SIZE + LOOKUP_SAMPLE_SIZE).cycle();
        b.iter(|| {
            let key = iter.next().unwrap();
            map.get(&key);
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*LOOKUP_HASHSLABMAP;
        let mut iter = (0..LOOKUP_MAP_SIZE + LOOKUP_SAMPLE_SIZE).cycle();
        b.iter(|| {
            let key = iter.next().unwrap();
            map.get(&key);
        });
    });
}

fn bench_lookup_key_few(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_key_few");
    group.bench_function("hashmap", |b| {
        let map = &*LOOKUP_HASHMAP;
        b.iter(|| {
            for key in 0..LOOKUP_SAMPLE_SIZE {
                map.get(&key);
            }
        });
    });
    group.bench_function("indexmap", |b| {
        let map = &*LOOKUP_INDEXMAP;
        b.iter(|| {
            for key in 0..LOOKUP_SAMPLE_SIZE {
                map.get(&key);
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*LOOKUP_HASHSLABMAP;
        b.iter(|| {
            for key in 0..LOOKUP_SAMPLE_SIZE {
                map.get(&key);
            }
        });
    });
}

// Test looking up keys in the same order as they were inserted
fn bench_lookup_key_few_inorder(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_key_few_inorder");
    group.bench_function("hashmap", |b| {
        let map = &*LOOKUP_HASHMAP;
        let keys = &*LOOKUP_SHUFFLED_KEYS;
        b.iter(|| {
            for key in &keys[0..LOOKUP_SAMPLE_SIZE as usize] {
                map.get(key);
            }
        });
    });
    group.bench_function("indexmap", |b| {
        let map = &*LOOKUP_INDEXMAP;
        let keys = &*LOOKUP_SHUFFLED_KEYS;
        b.iter(|| {
            for key in &keys[0..LOOKUP_SAMPLE_SIZE as usize] {
                map.get(key);
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*LOOKUP_HASHSLABMAP;
        let keys = &*LOOKUP_SHUFFLED_KEYS;
        b.iter(|| {
            for key in &keys[0..LOOKUP_SAMPLE_SIZE as usize] {
                map.get(key);
            }
        });
    });
}

/*
    Index lookup
*/
fn bench_lookup_index_exist(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_index_exist");
    group.bench_function("indexmap", |b| {
        let map = &*LOOKUP_INDEXMAP;
        let end = map.len();
        let start = end / 2;
        b.iter(|| {
            for idx in start..end {
                black_box(map.get_index(idx));
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*LOOKUP_HASHSLABMAP;
        let end = map.len();
        let start = end / 2;
        b.iter(|| {
            for idx in start..end {
                black_box(map.get_index(idx));
            }
        });
    });
}

fn bench_lookup_index_noexist(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_index_noexist");
    group.bench_function("indexmap", |b| {
        let map = &*LOOKUP_INDEXMAP;
        let start = map.len();
        let end = start + start / 2;
        b.iter(|| {
            for idx in start..end {
                black_box(map.get_index(idx));
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*LOOKUP_HASHSLABMAP;
        let start = map.len();
        let end = start + start / 2;
        b.iter(|| {
            for idx in start..end {
                black_box(map.get_index(idx));
            }
        });
    });
}

fn bench_lookup_index_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_index_single");
    group.bench_function("indexmap", |b| {
        let map = &*LOOKUP_INDEXMAP;
        let mut iter = (0..(2 * map.len())).cycle();
        b.iter(|| {
            let idx = iter.next().unwrap();
            black_box(map.get_index(idx));
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*LOOKUP_HASHSLABMAP;
        let mut iter = (0..(2 * map.len())).cycle();
        b.iter(|| {
            let idx = iter.next().unwrap();
            black_box(map.get_index(idx));
        });
    });
}

fn bench_lookup_index_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_index_random");
    let indices: Vec<usize> = (&*LOOKUP_SHUFFLED_KEYS)
        .iter()
        .map(|n| *n as usize)
        .collect();

    group.bench_function("indexmap", |b| {
        let map = &*LOOKUP_INDEXMAP;
        b.iter(|| {
            for idx in &indices {
                black_box(map.get_index(*idx));
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*LOOKUP_HASHSLABMAP;
        b.iter(|| {
            for idx in &indices {
                black_box(map.get_index(*idx));
            }
        });
    });
}

// fn entry_hashmap_150(c: &mut Criterion) {
//     let c = 150;
//     c.bench_function("entry_hashmap_150", |b| b.iter()|| {
//         let mut map = HashMap::with_capacity(c);
//         for x in 0..c {
//             map.entry(x).or_insert(());
//         }
//         map
//     });
// }

// fn entry_indexmap_150(c: &mut Criterion) {
//     let c = 150;
//     c.bench_function("entry_indexmap_150", |b| b.iter()|| {
//         let mut map = IndexMap::with_capacity(c);
//         for x in 0..c {
//             map.entry(x).or_insert(());
//         }
//         map
//     });
// }

/*
    Iterators
*/
fn bench_iter_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter_sum");
    let cap = 10_000;
    group.bench_function("hashmap", |b| {
        let mut map = HashMap::with_capacity(cap);
        for x in 0..cap {
            map.insert(x, ());
        }
        b.iter(|| map.keys().sum::<usize>());
    });
    group.bench_function("indexmap", |b| {
        let mut map = IndexMap::with_capacity(cap);
        for x in 0..cap {
            map.insert(x, ());
        }
        b.iter(|| map.keys().sum::<usize>());
    });
    group.bench_function("hashslabmap", |b| {
        let mut map = HashSlabMap::with_capacity(cap);
        for x in 0..cap {
            map.insert(x, ());
        }
        b.iter(|| map.keys().sum::<usize>());
    });
}

fn bench_iter_black_box(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter_black_box");
    let cap = 10_000;
    group.bench_function("hashmap", |b| {
        let mut map = HashMap::with_capacity(cap);
        for x in 0..cap {
            map.insert(x, ());
        }
        b.iter(|| {
            for &key in map.keys() {
                black_box(key);
            }
        });
    });
    group.bench_function("indexmap", |b| {
        let mut map = IndexMap::with_capacity(cap);
        for x in 0..cap {
            map.insert(x, ());
        }
        b.iter(|| {
            for &key in map.keys() {
                black_box(key);
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let mut map = HashSlabMap::with_capacity(cap);
        for x in 0..cap {
            map.insert(x, ());
        }
        b.iter(|| {
            for &key in map.keys() {
                black_box(key);
            }
        });
    });
}

// // use lazy_static so that comparison benchmarks use the exact same inputs
// lazy_static! {
//     static ref KEYS: Vec<u32> = shuffled_keys(0..LOOKUP_MAP_SIZE);
// }

// lazy_static! {
//     static ref HMAP_100K: HashMap<u32, u32> = {
//         let mut map = HashMap::with_capacity(LOOKUP_MAP_SIZE as usize);
//         let keys = &*KEYS;
//         for &key in keys {
//             map.insert(key, key);
//         }
//         map
//     };
// }

// lazy_static! {
//     static ref IMAP_100K: IndexMap<u32, u32> = {
//         let mut map = IndexMap::with_capacity(LOOKUP_MAP_SIZE as usize);
//         let keys = &*KEYS;
//         for &key in keys {
//             map.insert(key, key);
//         }
//         map
//     };
// }

// lazy_static! {
//     static ref HSMAP_100K: HashSlabMap<u32, u32> = {
//         let mut map = HashSlabMap::with_capacity(LOOKUP_MAP_SIZE as usize);
//         let keys = &*KEYS;
//         for &key in keys {
//             map.insert(key, key);
//         }
//         map
//     };
// }

// lazy_static! {
//     static ref IMAP_SORT_U32: IndexMap<u32, u32> = {
//         let mut map = IndexMap::with_capacity(SORT_MAP_SIZE);
//         for &key in &KEYS[..SORT_MAP_SIZE] {
//             map.insert(key, key);
//         }
//         map
//     };
// }
// lazy_static! {
//     static ref IMAP_SORT_S: IndexMap<String, String> = {
//         let mut map = IndexMap::with_capacity(SORT_MAP_SIZE);
//         for &key in &KEYS[..SORT_MAP_SIZE] {
//             map.insert(format!("{:^16x}", &key), String::new());
//         }
//         map
//     };
// }

// const MERGE: u64 = 10_000;
// fn hashmap_merge_simple(c: &mut Criterion) {
//     let first_map: HashMap<u64, _> = (0..MERGE).map(|i| (i, ())).collect();
//     let second_map: HashMap<u64, _> = (MERGE..MERGE * 2).map(|i| (i, ())).collect();
//     c.bench_function("hashmap_merge_simple", |b| b.iter()|| {
//         let mut merged = first_map.clone();
//         merged.extend(second_map.iter().map(|(&k, &v)| (k, v)));
//         merged
//     });
// }

// fn hashmap_merge_shuffle(c: &mut Criterion) {
//     let first_map: HashMap<u64, _> = (0..MERGE).map(|i| (i, ())).collect();
//     let second_map: HashMap<u64, _> = (MERGE..MERGE * 2).map(|i| (i, ())).collect();
//     let mut v = Vec::new();
//     let mut rng = small_rng();
//     c.bench_function("hashmap_merge_shuffle", |b| b.iter()|| {
//         let mut merged = first_map.clone();
//         v.extend(second_map.iter().map(|(&k, &v)| (k, v)));
//         v.shuffle(&mut rng);
//         merged.extend(v.drain(..));

//         merged
//     });
// }

// fn indexmap_merge_simple(c: &mut Criterion) {
//     let first_map: IndexMap<u64, _> = (0..MERGE).map(|i| (i, ())).collect();
//     let second_map: IndexMap<u64, _> = (MERGE..MERGE * 2).map(|i| (i, ())).collect();
//     c.bench_function("indexmap_merge_simple", |b| b.iter()|| {
//         let mut merged = first_map.clone();
//         merged.extend(second_map.iter().map(|(&k, &v)| (k, v)));
//         merged
//     });
// }

// fn indexmap_merge_shuffle(c: &mut Criterion) {
//     let first_map: IndexMap<u64, _> = (0..MERGE).map(|i| (i, ())).collect();
//     let second_map: IndexMap<u64, _> = (MERGE..MERGE * 2).map(|i| (i, ())).collect();
//     let mut v = Vec::new();
//     let mut rng = small_rng();
//     c.bench_function("indexmap_merge_shuffle", |b| b.iter()|| {
//         let mut merged = first_map.clone();
//         v.extend(second_map.iter().map(|(&k, &v)| (k, v)));
//         v.shuffle(&mut rng);
//         merged.extend(v.drain(..));

//         merged
//     });
// }

/*
    Remove
*/
fn bench_remove_key(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_key");

    let size = 1000;
    let vec: Vec<(u32, u32)> = (0..size).map(|x| (x, x)).collect();

    group.bench_function("hashmap", |b| {
        let map = HashMap::<u32, u32>::from_iter(vec.iter().cloned());
        b.iter(|| {
            let mut map = map.clone();
            for key in 0..size {
                map.remove(&key);
            }
            assert_eq!(map.len(), 0);
        })
    });
    group.bench_function("indexmap.swap_remove", |b| {
        let map = IndexMap::<u32, u32>::from_iter(vec.iter().cloned());
        b.iter(|| {
            let mut map = map.clone();
            for key in 0..size {
                map.swap_remove(&key);
            }
            assert_eq!(map.len(), 0);
        })
    });
    group.bench_function("indexmap.shift_remove", |b| {
        let map = IndexMap::<u32, u32>::from_iter(vec.iter().cloned());
        b.iter(|| {
            let mut map = map.clone();
            for key in 0..size {
                map.shift_remove(&key);
            }
            assert_eq!(map.len(), 0);
        })
    });
    group.bench_function("hashslabmap", |b| {
        let map = HashSlabMap::<u32, u32>::from_iter(vec.iter().cloned());
        b.iter(|| {
            let mut map = map.clone();
            for key in 0..size {
                map.remove(&key);
            }
            assert_eq!(map.len(), 0);
        })
    });
}

fn bench_remove_key_few(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_key_few");

    let few_len = 50;
    let mut keys = (&*LOOKUP_SHUFFLED_KEYS).clone();
    let mut rng = small_rng();
    keys.shuffle(&mut rng);
    keys.truncate(few_len);
    let keys = &keys;

    group.bench_function("hashmap", |b| {
        let map = LOOKUP_HASHMAP.clone();
        let removed_len = map.len() - few_len;
        b.iter(|| {
            let mut map = map.clone();
            for key in keys {
                map.remove(key);
            }
            assert_eq!(map.len(), removed_len);
        })
    });
    group.bench_function("indexmap.swap_remove", |b| {
        let map = LOOKUP_INDEXMAP.clone();
        let removed_len = map.len() - few_len;
        b.iter(|| {
            let mut map = map.clone();
            for key in keys {
                map.swap_remove(key);
            }
            assert_eq!(map.len(), removed_len);
        })
    });
    group.bench_function("indexmap.shift_remove", |b| {
        let map = LOOKUP_INDEXMAP.clone();
        let removed_len = map.len() - few_len;
        b.iter(|| {
            let mut map = map.clone();
            for key in keys {
                map.shift_remove(key);
            }
            assert_eq!(map.len(), removed_len);
        })
    });
    group.bench_function("hashslabmap", |b| {
        let map = LOOKUP_HASHSLABMAP.clone();
        let removed_len = map.len() - few_len;
        b.iter(|| {
            let mut map = map.clone();
            for key in keys {
                map.remove(key);
            }
            assert_eq!(map.len(), removed_len);
        })
    });
}

// indexmap on any removing reduce it length, but not the hashslabmap
fn bench_remove_index_half(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_index_half");

    let size = 1000;
    let vec: Vec<(u32, u32)> = (0..size).map(|x| (x, x)).collect();
    let mut keys = (0..size as usize / 2).collect::<Vec<_>>();
    keys.shuffle(&mut small_rng());
    let indices = &keys;

    group.bench_function("indexmap.swap_remove", |b| {
        let map = IndexMap::<u32, u32>::from_iter(vec.iter().cloned());
        b.iter(|| {
            let mut map = map.clone();
            for idx in indices {
                map.swap_remove_index(*idx);
            }
            assert_eq!(map.len(), indices.len());
        });
    });
    group.bench_function("indexmap.shift_remove", |b| {
        let map = IndexMap::<u32, u32>::from_iter(vec.iter().cloned());
        b.iter(|| {
            let mut map = map.clone();
            for idx in indices {
                map.shift_remove_index(*idx);
            }
            assert_eq!(map.len(), indices.len());
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = HashSlabMap::<u32, u32>::from_iter(vec.iter().cloned());
        b.iter(|| {
            let mut map = map.clone();
            for idx in indices {
                map.remove_index(*idx);
            }
            assert_eq!(map.len(), indices.len());
        });
    });
}

fn bench_remove_index_few(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_index_few");

    let few_len = 50;
    let mut keys = (0..few_len).collect::<Vec<_>>();
    keys.shuffle(&mut small_rng());
    let indices = &keys;

    group.bench_function("indexmap.swap_remove", |b| {
        let map = LOOKUP_INDEXMAP.clone();
        let removed_len = map.len() - few_len;
        b.iter(|| {
            let mut map = map.clone();
            for idx in indices {
                map.swap_remove_index(*idx);
            }
            assert_eq!(map.len(), removed_len);
        });
    });
    group.bench_function("indexmap.shift_remove", |b| {
        let map = LOOKUP_INDEXMAP.clone();
        let removed_len = map.len() - few_len;
        b.iter(|| {
            let mut map = map.clone();
            for idx in indices {
                map.shift_remove_index(*idx);
            }
            assert_eq!(map.len(), removed_len);
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = LOOKUP_HASHSLABMAP.clone();
        let removed_len = map.len() - few_len;
        b.iter(|| {
            let mut map = map.clone();
            for idx in indices {
                map.remove_index(*idx);
            }
            assert_eq!(map.len(), removed_len);
        });
    });
}

// fn few_retain_indexmap_100_000(c: &mut Criterion) {
//     let map = IMAP_100K.clone();

//     c.bench_function("few_retain_indexmap_100_000", |b| b.iter()|| {
//         let mut map = map.clone();
//         map.retain(|k, _| *k % 7 == 0);
//         map
//     });
// }

// fn few_retain_hashmap_100_000(c: &mut Criterion) {
//     let map = HMAP_100K.clone();

//     c.bench_function("few_retain_hashmap_100_000", |b| b.iter()|| {
//         let mut map = map.clone();
//         map.retain(|k, _| *k % 7 == 0);
//         map
//     });
// }

// fn half_retain_indexmap_100_000(c: &mut Criterion) {
//     let map = IMAP_100K.clone();

//     c.bench_function("half_retain_indexmap_100_000", |b| b.iter()|| {
//         let mut map = map.clone();
//         map.retain(|k, _| *k % 2 == 0);
//         map
//     });
// }

// fn half_retain_hashmap_100_000(c: &mut Criterion) {
//     let map = HMAP_100K.clone();

//     c.bench_function("half_retain_hashmap_100_000", |b| b.iter()|| {
//         let mut map = map.clone();
//         map.retain(|k, _| *k % 2 == 0);
//         map
//     });
// }

// fn many_retain_indexmap_100_000(c: &mut Criterion) {
//     let map = IMAP_100K.clone();

//     c.bench_function("many_retain_indexmap_100_000", |b| b.iter()|| {
//         let mut map = map.clone();
//         map.retain(|k, _| *k % 100 != 0);
//         map
//     });
// }

// fn many_retain_hashmap_100_000(c: &mut Criterion) {
//     let map = HMAP_100K.clone();

//     c.bench_function("many_retain_hashmap_100_000", |b| b.iter()|| {
//         let mut map = map.clone();
//         map.retain(|k, _| *k % 100 != 0);
//         map
//     });
// }

// // simple sort impl for comparison
// pub fn simple_sort<K: Ord + Hash, V>(m: &mut IndexMap<K, V>) {
//     let mut ordered: Vec<_> = m.drain(..).collect();
//     ordered.sort_by(|left, right| left.0.cmp(&right.0));
//     m.extend(ordered);
// }

// fn indexmap_sort_s(c: &mut Criterion) {
//     let map = IMAP_SORT_S.clone();

//     // there's a map clone there, but it's still useful to profile this
//     c.bench_function("indexmap_sort_s", |b| b.iter()|| {
//         let mut map = map.clone();
//         map.sort_keys();
//         map
//     });
// }

// fn indexmap_simple_sort_s(c: &mut Criterion) {
//     let map = IMAP_SORT_S.clone();

//     // there's a map clone there, but it's still useful to profile this
//     c.bench_function("indexmap_simple_sort_s", |b| b.iter()|| {
//         let mut map = map.clone();
//         simple_sort(&mut map);
//         map
//     });
// }

// fn indexmap_sort_u32(c: &mut Criterion) {
//     let map = IMAP_SORT_U32.clone();

//     // there's a map clone there, but it's still useful to profile this
//     c.bench_function("indexmap_sort_u32", |b| b.iter()|| {
//         let mut map = map.clone();
//         map.sort_keys();
//         map
//     });
// }

// fn indexmap_simple_sort_u32(c: &mut Criterion) {
//     let map = IMAP_SORT_U32.clone();

//     // there's a map clone there, but it's still useful to profile this
//     c.bench_function("indexmap_simple_sort_u32", |b| b.iter()|| {
//         let mut map = map.clone();
//         simple_sort(&mut map);
//         map
//     });
// }

// // measure the fixed overhead of cloning in sort benchmarks
// fn indexmap_clone_for_sort_s(c: &mut Criterion) {
//     let map = IMAP_SORT_S.clone();

//     c.bench_function("indexmap_clone_for_sort_s", |b| b.iter(|| map.clone()));
// }

// fn indexmap_clone_for_sort_u32(c: &mut Criterion) {
//     let map = IMAP_SORT_U32.clone();

//     c.bench_function("indexmap_clone_for_sort_u32", |b| b.iter(|| map.clone()));
// }

criterion_group!(
    benches,
    bench_new,
    bench_with_capacity,
    bench_grow,
    bench_insert,
    bench_lookup_key_exist,
    bench_lookup_key_noexist,
    bench_lookup_key_single,
    bench_lookup_key_few,
    bench_lookup_key_few_inorder,
    bench_lookup_index_exist,
    bench_lookup_index_noexist,
    bench_lookup_index_single,
    bench_lookup_index_random,
    // entry_hashmap_150,
    // entry_indexmap_150,
    bench_iter_sum,
    bench_iter_black_box,
    // hashmap_merge_simple,
    // hashmap_merge_shuffle,
    // indexmap_merge_simple,
    // indexmap_merge_shuffle,
    // few_retain_indexmap_100_000,
    // few_retain_hashmap_100_000,
    // half_retain_indexmap_100_000,
    // half_retain_hashmap_100_000,
    // many_retain_indexmap_100_000,
    // many_retain_hashmap_100_000,
    // simple_sort,
    // indexmap_sort_s,
    // indexmap_simple_sort_s,
    // indexmap_sort_u32,
    // indexmap_simple_sort_u32,
    // indexmap_clone_for_sort_s,
    // indexmap_clone_for_sort_u32,
    bench_remove_key,
    bench_remove_key_few,
    bench_remove_index_half,
    bench_remove_index_few,
);
criterion_main!(benches);
