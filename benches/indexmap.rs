use criterion::measurement::Measurement;
use lazy_static::lazy_static;

use fnv::FnvHasher;
use std::hash::BuildHasherDefault;
use std::hash::Hash;
type FnvBuilder = BuildHasherDefault<FnvHasher>;

use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion,
};

use hashslab::HashSlabMap;
use indexmap::IndexMap;

use std::collections::HashMap;

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

/// Use a consistently seeded Rng for benchmark stability
fn small_rng() -> SmallRng {
    let seed = u64::from_le_bytes(*b"indexmap");
    SmallRng::seed_from_u64(seed)
}

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

fn insert<'a, M, K, V>(group: &mut BenchmarkGroup<'a, M>, name: &str, list: Vec<(K, V)>)
where
    M: Measurement,
    K: Hash + Eq,
{
    let len = list.len();
    group.bench_with_input(
        BenchmarkId::new("hashmap", format!("{name} x {len}")),
        &list,
        |b, list| {
            b.iter(|| {
                let mut map = HashMap::with_capacity(len);
                for (k, v) in list {
                    map.insert(k, v);
                }
            })
        },
    );
    group.bench_with_input(
        BenchmarkId::new("indexmap", format!("{name} x {len}")),
        &list,
        |b, list| {
            b.iter(|| {
                let mut map = IndexMap::with_capacity(len);
                for (k, v) in list {
                    map.insert(k, v);
                }
            })
        },
    );
    group.bench_with_input(
        BenchmarkId::new("hashslabmap", format!("{name} x {len}")),
        &list,
        |b, list| {
            b.iter(|| {
                let mut map = HashSlabMap::with_capacity(len);
                for (k, v) in list {
                    map.insert(k, v);
                }
            })
        },
    );
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

fn hashmap_iter_sum_10_000(c: &mut Criterion) {
    let cap = 10_000;
    let mut map = HashMap::with_capacity(cap);
    let len = cap - cap / 10;
    for x in 0..len {
        map.insert(x, ());
    }
    assert_eq!(map.len(), len);
    c.bench_function("hashmap_iter_sum_10_000", |b| {
        b.iter(|| map.keys().sum::<usize>())
    });
}

fn indexmap_iter_sum_10_000(c: &mut Criterion) {
    let cap = 10_000;
    let mut map = IndexMap::with_capacity(cap);
    let len = cap - cap / 10;
    for x in 0..len {
        map.insert(x, ());
    }
    assert_eq!(map.len(), len);
    c.bench_function("indexmap_iter_sum_10_000", |b| {
        b.iter(|| map.keys().sum::<usize>())
    });
}

fn hashslabmap_iter_sum_10_000(c: &mut Criterion) {
    let cap = 10_000;
    let mut map = HashSlabMap::with_capacity(cap);
    let len = cap - cap / 10;
    for x in 0..len {
        map.insert(x, ());
    }
    assert_eq!(map.len(), len);
    c.bench_function("hashslabmap_iter_sum_10_000", |b| {
        b.iter(|| map.keys().sum::<usize>())
    });
}

fn hashmap_iter_black_box_10_000(c: &mut Criterion) {
    let cap = 10_000;
    let mut map = HashMap::with_capacity(cap);
    let len = cap - cap / 10;
    for x in 0..len {
        map.insert(x, ());
    }
    assert_eq!(map.len(), len);
    c.bench_function("hashmap_iter_black_box_10_000", |b| {
        b.iter(|| {
            for &key in map.keys() {
                black_box(key);
            }
        })
    });
}

fn indexmap_iter_black_box_10_000(c: &mut Criterion) {
    let cap = 10_000;
    let mut map = IndexMap::with_capacity(cap);
    let len = cap - cap / 10;
    for x in 0..len {
        map.insert(x, ());
    }
    assert_eq!(map.len(), len);
    c.bench_function("indexmap_iter_black_box_10_000", |b| {
        b.iter(|| {
            for &key in map.keys() {
                black_box(key);
            }
        })
    });
}

fn hashslabmap_iter_black_box_10_000(c: &mut Criterion) {
    let cap = 10_000;
    let mut map = HashSlabMap::with_capacity(cap);
    let len = cap - cap / 10;
    for x in 0..len {
        map.insert(x, ());
    }
    assert_eq!(map.len(), len);
    c.bench_function("hashslabmap_iter_black_box_10_000", |b| {
        b.iter(|| {
            for &key in map.keys() {
                black_box(key);
            }
        })
    });
}

fn shuffled_keys<I>(iter: I) -> Vec<I::Item>
where
    I: IntoIterator,
{
    let mut v = Vec::from_iter(iter);
    let mut rng = small_rng();
    v.shuffle(&mut rng);
    v
}

fn hashmap_lookup_10_000_exist(c: &mut Criterion) {
    let cap = 10_000;
    let mut map = HashMap::with_capacity(cap);
    let keys = shuffled_keys(0..cap);
    for &key in &keys {
        map.insert(key, 1);
    }
    c.bench_function("hashmap_lookup_10_000_exist", |b| {
        b.iter(|| {
            let mut found = 0;
            for key in 5000..cap {
                found += map.get(&key).is_some() as i32;
            }
            found
        })
    });
}

fn indexmap_lookup_10_000_exist(c: &mut Criterion) {
    let cap = 10_000;
    let mut map = IndexMap::with_capacity(cap);
    let keys = shuffled_keys(0..cap);
    for &key in &keys {
        map.insert(key, 1);
    }
    c.bench_function("indexmap_lookup_10_000_exist", |b| {
        b.iter(|| {
            let mut found = 0;
            for key in 5000..cap {
                found += map.get(&key).is_some() as i32;
            }
            found
        })
    });
}

fn hashslabmap_lookup_10_000_exist(c: &mut Criterion) {
    let cap = 10_000;
    let mut map = HashSlabMap::with_capacity(cap);
    let keys = shuffled_keys(0..cap);
    for &key in &keys {
        map.insert(key, 1);
    }
    c.bench_function("hashslabmap_lookup_10_000_exist", |b| {
        b.iter(|| {
            let mut found = 0;
            for key in 5000..cap {
                found += map.get(&key).is_some() as i32;
            }
            found
        })
    });
}

fn hashmap_lookup_10_000_noexist(c: &mut Criterion) {
    let cap = 10_000;
    let mut map = HashMap::with_capacity(cap);
    let keys = shuffled_keys(0..cap);
    for &key in &keys {
        map.insert(key, 1);
    }
    c.bench_function("hashmap_lookup_10_000_noexist", |b| {
        b.iter(|| {
            let mut found = 0;
            for key in cap..15000 {
                found += map.get(&key).is_some() as i32;
            }
            found
        })
    });
}

fn indexmap_lookup_10_000_noexist(c: &mut Criterion) {
    let cap = 10_000;
    let mut map = IndexMap::with_capacity(cap);
    let keys = shuffled_keys(0..cap);
    for &key in &keys {
        map.insert(key, 1);
    }
    c.bench_function("indexmap_lookup_10_000_noexist", |b| {
        b.iter(|| {
            let mut found = 0;
            for key in cap..15000 {
                found += map.get(&key).is_some() as i32;
            }
            found
        })
    });
}

fn hashslabmap_lookup_10_000_noexist(c: &mut Criterion) {
    let cap = 10_000;
    let mut map = IndexMap::with_capacity(cap);
    let keys = shuffled_keys(0..cap);
    for &key in &keys {
        map.insert(key, 1);
    }
    c.bench_function("hashslabmap_lookup_10_000_noexist", |b| {
        b.iter(|| {
            let mut found = 0;
            for key in cap..15000 {
                found += map.get(&key).is_some() as i32;
            }
            found
        })
    });
}

// number of items to look up
const LOOKUP_MAP_SIZE: u32 = 100_000_u32;
// const LOOKUP_SAMPLE_SIZE: u32 = 5000;
const SORT_MAP_SIZE: usize = 10_000;

// use lazy_static so that comparison benchmarks use the exact same inputs
lazy_static! {
    static ref KEYS: Vec<u32> = shuffled_keys(0..LOOKUP_MAP_SIZE);
}

lazy_static! {
    static ref HMAP_100K: HashMap<u32, u32> = {
        let mut map = HashMap::with_capacity(LOOKUP_MAP_SIZE as usize);
        let keys = &*KEYS;
        for &key in keys {
            map.insert(key, key);
        }
        map
    };
}

lazy_static! {
    static ref IMAP_100K: IndexMap<u32, u32> = {
        let mut map = IndexMap::with_capacity(LOOKUP_MAP_SIZE as usize);
        let keys = &*KEYS;
        for &key in keys {
            map.insert(key, key);
        }
        map
    };
}

lazy_static! {
    static ref HSMAP_100K: HashSlabMap<u32, u32> = {
        let mut map = HashSlabMap::with_capacity(LOOKUP_MAP_SIZE as usize);
        let keys = &*KEYS;
        for &key in keys {
            map.insert(key, key);
        }
        map
    };
}

lazy_static! {
    static ref IMAP_SORT_U32: IndexMap<u32, u32> = {
        let mut map = IndexMap::with_capacity(SORT_MAP_SIZE);
        for &key in &KEYS[..SORT_MAP_SIZE] {
            map.insert(key, key);
        }
        map
    };
}
lazy_static! {
    static ref IMAP_SORT_S: IndexMap<String, String> = {
        let mut map = IndexMap::with_capacity(SORT_MAP_SIZE);
        for &key in &KEYS[..SORT_MAP_SIZE] {
            map.insert(format!("{:^16x}", &key), String::new());
        }
        map
    };
}

// fn lookup_hashmap_100_000_multi(c: &mut Criterion) {
//     let map = &*HMAP_100K;
//     c.bench_function("lookup_hashmap_100_000_multi", |b| b.iter()|| {
//         let mut found = 0;
//         for key in 0..LOOKUP_SAMPLE_SIZE {
//             found += map.get(&key).is_some() as u32;
//         }
//         found
//     });
// }

// fn lookup_indexmap_100_000_multi(c: &mut Criterion) {
//     let map = &*IMAP_100K;
//     c.bench_function("lookup_indexmap_100_000_multi", |b| b.iter()|| {
//         let mut found = 0;
//         for key in 0..LOOKUP_SAMPLE_SIZE {
//             found += map.get(&key).is_some() as u32;
//         }
//         found
//     });
// }

// // inorder: Test looking up keys in the same order as they were inserted
// fn lookup_hashmap_100_000_inorder_multi(c: &mut Criterion) {
//     let map = &*HMAP_100K;
//     let keys = &*KEYS;
//     c.bench_function("lookup_hashmap_100_000_inorder_multi", |b| b.iter()|| {
//         let mut found = 0;
//         for key in &keys[0..LOOKUP_SAMPLE_SIZE as usize] {
//             found += map.get(key).is_some() as u32;
//         }
//         found
//     });
// }

// fn lookup_indexmap_100_000_inorder_multi(c: &mut Criterion) {
//     let map = &*IMAP_100K;
//     let keys = &*KEYS;
//     c.bench_function("lookup_indexmap_100_000_inorder_multi", |b| b.iter()|| {
//         let mut found = 0;
//         for key in &keys[0..LOOKUP_SAMPLE_SIZE as usize] {
//             found += map.get(key).is_some() as u32;
//         }
//         found
//     });
// }

// fn lookup_hashmap_100_000_single(c: &mut Criterion) {
//     let map = &*HMAP_100K;
//     let mut iter = (0..LOOKUP_MAP_SIZE + LOOKUP_SAMPLE_SIZE).cycle();
//     c.bench_function("lookup_hashmap_100_000_single", |b| b.iter()|| {
//         let key = iter.next().unwrap();
//         map.get(&key).is_some()
//     });
// }

// fn lookup_indexmap_100_000_single(c: &mut Criterion) {
//     let map = &*IMAP_100K;
//     let mut iter = (0..LOOKUP_MAP_SIZE + LOOKUP_SAMPLE_SIZE).cycle();
//     c.bench_function("lookup_indexmap_100_000_single", |b| b.iter()|| {
//         let key = iter.next().unwrap();
//         map.get(&key).is_some()
//     });
// }

const GROW_SIZE: usize = 100_000;
type GrowKey = u32;

// Test grow/resize without preallocation
fn hashmap_grow_fnv_100_000(c: &mut Criterion) {
    c.bench_function("hashmap_grow_fnv_100_000", |b| {
        b.iter(|| {
            let mut map: HashMap<_, _, FnvBuilder> = HashMap::default();
            for x in 0..GROW_SIZE {
                map.insert(x as GrowKey, x as GrowKey);
            }
            map
        })
    });
}

fn indexmap_grow_fnv_100_000(c: &mut Criterion) {
    c.bench_function("indexmap_grow_fnv_100_000", |b| {
        b.iter(|| {
            let mut map: IndexMap<_, _, FnvBuilder> = IndexMap::default();
            for x in 0..GROW_SIZE {
                map.insert(x as GrowKey, x as GrowKey);
            }
            map
        })
    });
}

fn hashslabmap_grow_fnv_100_000(c: &mut Criterion) {
    c.bench_function("hashslabmap_grow_fnv_100_000", |b| {
        b.iter(|| {
            let mut map: HashSlabMap<_, _, FnvBuilder> = HashSlabMap::default();
            for x in 0..GROW_SIZE {
                map.insert(x as GrowKey, x as GrowKey);
            }
            map
        })
    });
}

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

fn indexmap_swap_remove_100_000(c: &mut Criterion) {
    let map = IMAP_100K.clone();
    let mut keys = Vec::from_iter(map.keys().copied());
    let mut rng = small_rng();
    keys.shuffle(&mut rng);

    c.bench_function("indexmap_swap_remove_100_000", |b| {
        b.iter(|| {
            let mut map = map.clone();
            for key in &keys {
                map.swap_remove(key);
            }
            assert_eq!(map.len(), 0);
            map
        })
    });
}

fn hashslabmap_remove_100_000(c: &mut Criterion) {
    let map = HSMAP_100K.clone();
    let mut keys = Vec::from_iter(map.keys().copied());
    let mut rng = small_rng();
    keys.shuffle(&mut rng);

    c.bench_function("hashslabmap_remove_100_000", |b| {
        b.iter(|| {
            let mut map = map.clone();
            for key in &keys {
                map.remove(key);
            }
            assert_eq!(map.len(), 0);
            map
        })
    });
}

fn indexmap_shift_remove_100_000_few(c: &mut Criterion) {
    let map = IMAP_100K.clone();
    let mut keys = Vec::from_iter(map.keys().copied());
    let mut rng = small_rng();
    keys.shuffle(&mut rng);
    keys.truncate(50);

    c.bench_function("indexmap_shift_remove_100_000_few", |b| {
        b.iter(|| {
            let mut map = map.clone();
            for key in &keys {
                map.shift_remove(key);
            }
            assert_eq!(map.len(), IMAP_100K.len() - keys.len());
            map
        })
    });
}

fn hashslabmap_remove_100_000_few(c: &mut Criterion) {
    let map = HSMAP_100K.clone();
    let mut keys = Vec::from_iter(map.keys().copied());
    let mut rng = small_rng();
    keys.shuffle(&mut rng);
    keys.truncate(50);

    c.bench_function("hashslabmap_remove_100_000_few", |b| {
        b.iter(|| {
            let mut map = map.clone();
            for key in &keys {
                map.remove(key);
            }
            assert_eq!(map.len(), IMAP_100K.len() - keys.len());
            map
        })
    });
}

// fn shift_remove_indexmap_2_000_full(c: &mut Criterion) {
//     let mut keys = KEYS[..2_000].to_vec();
//     let mut map = IndexMap::with_capacity(keys.len());
//     for &key in &keys {
//         map.insert(key, key);
//     }
//     let mut rng = small_rng();
//     keys.shuffle(&mut rng);

//     c.bench_function("shift_remove_indexmap_2_000_full", |b| b.iter()|| {
//         let mut map = map.clone();
//         for key in &keys {
//             map.shift_remove(key);
//         }
//         assert_eq!(map.len(), 0);
//         map
//     });
// }

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

// Emulate HashSlab::remove as update Option<T>
fn indexmap_option_remove_100_000(c: &mut Criterion) {
    let cap = LOOKUP_MAP_SIZE;
    let mut map = IndexMap::with_capacity(cap as usize);
    let keys = &*KEYS;
    for &key in keys {
        map.insert(key, Some(key));
    }
    let mut keys = Vec::from_iter(map.keys().copied());
    let mut rng = small_rng();
    keys.shuffle(&mut rng);

    c.bench_function("indexmap_option_remove_100_000", |b| {
        b.iter(|| {
            let mut map = map.clone();
            for key in &keys {
                map.get_mut(key).map(|v| *v = None);
            }
            assert_eq!(map.values().filter(|v| v.is_some()).count(), 0);
            map
        })
    });
}

fn hashslabmap_option_remove_100_000(c: &mut Criterion) {
    let cap = LOOKUP_MAP_SIZE;
    let mut map = HashSlabMap::with_capacity(cap as usize);
    let keys = &*KEYS;
    for &key in keys {
        map.insert(key, Some(key));
    }
    let mut keys = Vec::from_iter(map.keys().copied());
    let mut rng = small_rng();
    keys.shuffle(&mut rng);

    c.bench_function("hashslabmap_option_remove_100_000", |b| {
        b.iter(|| {
            let mut map = map.clone();
            for key in &keys {
                map.remove(key);
            }
            assert_eq!(map.len(), 0);
            map
        })
    });
}

fn indexmap_option_remove_100_000_few(c: &mut Criterion) {
    let cap = LOOKUP_MAP_SIZE;
    let mut map = IndexMap::with_capacity(cap as usize);
    let keys = &*KEYS;
    for &key in keys {
        map.insert(key, Some(key));
    }
    let mut keys = Vec::from_iter(map.keys().copied());
    let mut rng = small_rng();
    keys.shuffle(&mut rng);
    keys.truncate(50);

    c.bench_function("indexmap_option_remove_100_000_few", |b| {
        b.iter(|| {
            let mut map = map.clone();
            for key in &keys {
                map.get_mut(key).map(|v| *v = None);
            }
            assert_eq!(
                map.values().filter(|v| v.is_some()).count(),
                LOOKUP_MAP_SIZE as usize - 50
            );
            map
        })
    });
}

fn hashslabmap_option_remove_100_000_few(c: &mut Criterion) {
    let mut map = HashSlabMap::with_capacity(LOOKUP_MAP_SIZE as usize);
    let keys = &*KEYS;
    for &key in keys {
        map.insert(key, Some(key));
    }
    let mut keys = Vec::from_iter(map.keys().copied());
    let mut rng = small_rng();
    keys.shuffle(&mut rng);
    keys.truncate(50);

    c.bench_function("hashslabmap_option_remove_100_000_few", |b| {
        b.iter(|| {
            let mut map = map.clone();
            for key in &keys {
                map.remove(key);
            }
            assert_eq!(map.len(), LOOKUP_MAP_SIZE as usize - 50);
            map
        })
    });
}

criterion_group!(
    benches,
    bench_new,
    bench_with_capacity,
    bench_insert,
    // entry_hashmap_150,
    // entry_indexmap_150,
    hashmap_iter_sum_10_000,
    indexmap_iter_sum_10_000,
    hashslabmap_iter_sum_10_000,
    hashmap_iter_black_box_10_000,
    indexmap_iter_black_box_10_000,
    hashslabmap_iter_black_box_10_000,
    hashmap_lookup_10_000_exist,
    indexmap_lookup_10_000_exist,
    hashslabmap_lookup_10_000_exist,
    hashmap_lookup_10_000_noexist,
    indexmap_lookup_10_000_noexist,
    hashslabmap_lookup_10_000_noexist,
    // lookup_hashmap_100_000_multi,
    // lookup_indexmap_100_000_multi,
    // lookup_hashmap_100_000_inorder_multi,
    // lookup_indexmap_100_000_inorder_multi,
    // lookup_hashmap_100_000_single,
    // lookup_indexmap_100_000_single,
    hashmap_grow_fnv_100_000,
    indexmap_grow_fnv_100_000,
    hashslabmap_grow_fnv_100_000,
    // hashmap_merge_simple,
    // hashmap_merge_shuffle,
    // indexmap_merge_simple,
    // indexmap_merge_shuffle,
    indexmap_swap_remove_100_000,
    hashslabmap_remove_100_000,
    indexmap_shift_remove_100_000_few,
    hashslabmap_remove_100_000_few,
    // shift_remove_indexmap_2_000_full,
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
    indexmap_option_remove_100_000,
    hashslabmap_option_remove_100_000,
    indexmap_option_remove_100_000_few,
    hashslabmap_option_remove_100_000_few
);
criterion_main!(benches);
