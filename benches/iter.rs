use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use hashslab::HashSlabMap;
use indexmap::IndexMap;

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

criterion_group!(benches, bench_iter_sum, bench_iter_black_box);
criterion_main!(benches);
