use std::{collections::HashMap, sync::LazyLock};

use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
    Throughput,
};

use hashslab::HashSlabMap;
use indexmap::IndexMap;

static PLOT_CONFIG_LOG: LazyLock<PlotConfiguration> =
    LazyLock::new(|| PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

fn bench_insert_small_key(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_small_key");
    group.plot_config(PLOT_CONFIG_LOG.clone());

    for size in [1usize, 100, 10_000] {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, &size| {
            let mut map = HashMap::with_capacity(size);
            b.iter(|| {
                for k in 0..size {
                    map.insert(k, ());
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("indexmap", size), &size, |b, &size| {
            let mut map = IndexMap::with_capacity(size);
            b.iter(|| {
                for k in 0..size {
                    map.insert(k, ());
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("hashslabmap", size), &size, |b, &size| {
            let mut map = HashSlabMap::with_capacity(size);
            b.iter(|| {
                for k in 0..size {
                    map.insert(k, ());
                }
            })
        });
    }
}

fn bench_insert_big_key(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_big_key");
    group.plot_config(PLOT_CONFIG_LOG.clone());
    let key_elements = 100;

    for size in [1usize, 100, 10_000] {
        group.throughput(Throughput::Elements(size as u64));

        let list = (0..size)
            .map(|n| {
                let mut key = vec![0usize; key_elements];
                key[0] = n;
                key
            })
            .collect::<Vec<_>>();

        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, &size| {
            let mut map = HashMap::with_capacity(size);
            b.iter(|| {
                let list = list.clone();
                for k in list {
                    map.insert(k, ());
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("indexmap", size), &size, |b, &size| {
            let mut map = IndexMap::with_capacity(size);
            b.iter(|| {
                let list = list.clone();
                for k in list {
                    map.insert(k, ());
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("hashslabmap", size), &size, |b, &size| {
            let mut map = HashSlabMap::with_capacity(size);
            b.iter(|| {
                let list = list.clone();
                for k in list {
                    map.insert(k, ());
                }
            })
        });
    }
}

fn bench_insert_key_value(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_key_value");
    group.plot_config(PLOT_CONFIG_LOG.clone());

    const VAL_SIZE: usize = 100;

    for size in [1usize, 100, 10_000] {
        group.throughput(Throughput::Elements(size as u64));

        let list = (0..size)
            .map(|n| (n.to_string(), [0u8; VAL_SIZE]))
            .collect::<Vec<_>>();

        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, &size| {
            let mut map = HashMap::with_capacity(size);
            b.iter(|| {
                let list = list.clone();
                for (k, v) in list {
                    map.insert(k, v);
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("indexmap", size), &size, |b, &size| {
            let mut map = IndexMap::with_capacity(size);
            b.iter(|| {
                let list = list.clone();
                for (k, v) in list {
                    map.insert(k, v);
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("hashslabmap", size), &size, |b, &size| {
            let mut map = HashSlabMap::with_capacity(size);
            b.iter(|| {
                let list = list.clone();
                for (k, v) in list {
                    map.insert(k, v);
                }
            })
        });
    }
}

criterion_group!(
    benches,
    bench_insert_small_key,
    bench_insert_big_key,
    bench_insert_key_value
);
criterion_main!(benches);
