use std::{collections::HashMap, sync::LazyLock};

use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration, Throughput,
};

use hashslab::HashSlabMap;
use indexmap::IndexMap;

static PLOT_CONFIG_LOG: LazyLock<PlotConfiguration> =
    LazyLock::new(|| PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

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
    group.plot_config(PLOT_CONFIG_LOG.clone());

    for cap in [1, 100, 10_000].iter() {
        group.throughput(Throughput::Elements(*cap as u64));

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

// Test grow/resize without preallocation
fn bench_grow(c: &mut Criterion) {
    let mut group = c.benchmark_group("grow");
    group.plot_config(PLOT_CONFIG_LOG.clone());

    for grow_size in [1, 100, 10_000].iter() {
        group.throughput(Throughput::Elements(*grow_size as u64));

        group.bench_with_input(
            BenchmarkId::new("hashmap", grow_size),
            grow_size,
            |b, grow_size| {
                let mut map = HashMap::new();
                b.iter(|| {
                    for x in 0..*grow_size {
                        black_box(map.insert(x, x));
                    }
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("indexmap", grow_size),
            grow_size,
            |b, grow_size| {
                let mut map = IndexMap::new();
                b.iter(|| {
                    for x in 0..*grow_size {
                        black_box(map.insert(x, x));
                    }
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("hashslabmap", grow_size),
            grow_size,
            |b, grow_size| {
                let mut map = HashSlabMap::new();
                b.iter(|| {
                    for x in 0..*grow_size {
                        black_box(map.insert(x, x));
                    }
                })
            },
        );
    }
}
criterion_group!(benches, bench_new, bench_with_capacity, bench_grow,);
criterion_main!(benches);
