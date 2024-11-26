use std::collections::HashMap;

use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration,
};

use hashslab::HashSlabMap;
use indexmap::IndexMap;

mod util;
use rand::seq::SliceRandom;
use util::*;

const LOOKUP_SEQ_SIZE: usize = 10_000;
const LOOKUP_SAMPLE_SIZE: u32 = 5000;

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

fn bench_lookup_key_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_key_single");

    group.bench_function("hashmap", |b| {
        let map = &*SHUFFLED_HASHMAP;
        let mut iter = (0..LOOKUP_MAP_SIZE + LOOKUP_SAMPLE_SIZE).cycle();
        b.iter(|| {
            let key = iter.next().unwrap();
            map.get(&key);
        });
    });
    group.bench_function("indexmap", |b| {
        let map = &*SHUFFLED_INDEXMAP;
        let mut iter = (0..LOOKUP_MAP_SIZE + LOOKUP_SAMPLE_SIZE).cycle();
        b.iter(|| {
            let key = iter.next().unwrap();
            map.get(&key);
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*SHUFFLED_HASHSLABMAP;
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
        let map = &*SHUFFLED_HASHMAP;
        b.iter(|| {
            for key in 0..LOOKUP_SAMPLE_SIZE {
                map.get(&key);
            }
        });
    });
    group.bench_function("indexmap", |b| {
        let map = &*SHUFFLED_INDEXMAP;
        b.iter(|| {
            for key in 0..LOOKUP_SAMPLE_SIZE {
                map.get(&key);
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*SHUFFLED_HASHSLABMAP;
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
        let map = &*SHUFFLED_HASHMAP;
        let keys = &*SHUFFLED_KEYS;
        b.iter(|| {
            for key in &keys[0..LOOKUP_SAMPLE_SIZE as usize] {
                map.get(key);
            }
        });
    });
    group.bench_function("indexmap", |b| {
        let map = &*SHUFFLED_INDEXMAP;
        let keys = &*SHUFFLED_KEYS;
        b.iter(|| {
            for key in &keys[0..LOOKUP_SAMPLE_SIZE as usize] {
                map.get(key);
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*SHUFFLED_HASHSLABMAP;
        let keys = &*SHUFFLED_KEYS;
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
        let map = &*SHUFFLED_INDEXMAP;
        let end = map.len();
        let start = end / 2;
        b.iter(|| {
            for idx in start..end {
                black_box(map.get_index(idx));
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*SHUFFLED_HASHSLABMAP;
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
        let map = &*SHUFFLED_INDEXMAP;
        let start = map.len();
        let end = start + start / 2;
        b.iter(|| {
            for idx in start..end {
                black_box(map.get_index(idx));
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*SHUFFLED_HASHSLABMAP;
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
        let map = &*SHUFFLED_INDEXMAP;
        let mut iter = (0..(2 * map.len())).cycle();
        b.iter(|| {
            let idx = iter.next().unwrap();
            black_box(map.get_index(idx));
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*SHUFFLED_HASHSLABMAP;
        let mut iter = (0..(2 * map.len())).cycle();
        b.iter(|| {
            let idx = iter.next().unwrap();
            black_box(map.get_index(idx));
        });
    });
}

fn bench_lookup_index_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_index_random");
    let indices: Vec<usize> = (&*SHUFFLED_KEYS).iter().map(|n| *n as usize).collect();

    group.bench_function("indexmap", |b| {
        let map = &*SHUFFLED_INDEXMAP;
        b.iter(|| {
            for idx in &indices {
                black_box(map.get_index(*idx));
            }
        });
    });
    group.bench_function("hashslabmap", |b| {
        let map = &*SHUFFLED_HASHSLABMAP;
        b.iter(|| {
            for idx in &indices {
                black_box(map.get_index(*idx));
            }
        });
    });
}

fn bench_lookup_key_size(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("lookup_key_size");
    group.plot_config(plot_config);

    let cap = u8::MAX;

    for size in [1usize, 10, 100, 1000, 10000] {
        let mut list = (0..cap)
            .map(|n| {
                let mut key = vec![0u8; size];
                key[0] = n;
                (key, ())
            })
            .collect::<Vec<_>>();
        list.shuffle(&mut small_rng());
        let lookup = list
            .iter()
            .cloned()
            .take(cap as usize / 10)
            .collect::<Vec<_>>();
        list.shuffle(&mut small_rng());

        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, _| {
            let map = HashMap::<Vec<u8>, ()>::from_iter(list.clone());
            b.iter(|| {
                let lookup = lookup.clone();
                for (k, _) in lookup {
                    black_box(map.get(&k));
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("indexmap", size), &size, |b, _| {
            let map = IndexMap::<Vec<u8>, ()>::from_iter(list.clone());
            b.iter(|| {
                let lookup = lookup.clone();
                for (k, _) in lookup {
                    black_box(map.get(&k));
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("hashslabmap", size), &size, |b, _| {
            let map = HashSlabMap::<Vec<u8>, ()>::from_iter(list.clone());
            b.iter(|| {
                let lookup = lookup.clone();
                for (k, _) in lookup {
                    black_box(map.get(&k));
                }
            })
        });
    }
}

criterion_group!(
    benches,
    bench_lookup_key_exist,
    bench_lookup_key_noexist,
    bench_lookup_key_single,
    bench_lookup_key_few,
    bench_lookup_key_few_inorder,
    bench_lookup_key_size,
    bench_lookup_index_exist,
    bench_lookup_index_noexist,
    bench_lookup_index_single,
    bench_lookup_index_random,
);
criterion_main!(benches);
