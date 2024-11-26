use std::collections::HashMap;

use criterion::{criterion_group, criterion_main, Criterion};
use rand::seq::SliceRandom;

use hashslab::HashSlabMap;
use indexmap::IndexMap;

mod util;
use util::*;

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
    let mut keys = (&*SHUFFLED_KEYS).clone();
    let mut rng = small_rng();
    keys.shuffle(&mut rng);
    keys.truncate(few_len);
    let keys = &keys;

    group.bench_function("hashmap", |b| {
        let map = SHUFFLED_HASHMAP.clone();
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
        let map = SHUFFLED_INDEXMAP.clone();
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
        let map = SHUFFLED_INDEXMAP.clone();
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
        let map = SHUFFLED_HASHSLABMAP.clone();
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
        let map = SHUFFLED_INDEXMAP.clone();
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
        let map = SHUFFLED_INDEXMAP.clone();
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
        let map = SHUFFLED_HASHSLABMAP.clone();
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

criterion_group!(
    benches,
    bench_remove_key,
    bench_remove_key_few,
    bench_remove_index_half,
    bench_remove_index_few,
);
criterion_main!(benches);
