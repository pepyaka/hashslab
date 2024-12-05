use std::{collections::HashMap, sync::LazyLock};

use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
};

use hashslab::HashSlabMap;
use indexmap::IndexMap;

static PLOT_CONFIG_LOG: LazyLock<PlotConfiguration> =
    LazyLock::new(|| PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

fn bench_entry_or_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("entry_or_insert");
    group.plot_config(PLOT_CONFIG_LOG.clone());

    for size in [1usize, 100, 10_000] {
        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, &size| {
            let mut map = HashMap::with_capacity(size);
            b.iter(|| {
                for k in 0..size {
                    map.entry(k).or_insert(());
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("indexmap", size), &size, |b, &size| {
            let mut map = IndexMap::with_capacity(size);
            b.iter(|| {
                for k in 0..size {
                    map.entry(k).or_insert(());
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("hashslabmap", size), &size, |b, &size| {
            let mut map = HashSlabMap::with_capacity(size);
            b.iter(|| {
                for k in 0..size {
                    map.entry(k).or_insert(());
                }
            })
        });
    }
}

fn bench_entry_and_modify(c: &mut Criterion) {
    let mut group = c.benchmark_group("entry_and_modify");
    group.plot_config(PLOT_CONFIG_LOG.clone());

    for size in [1usize, 100, 10_000] {
        let vec: Vec<_> = (0..size).map(|x| (x, x)).collect();

        group.bench_with_input(BenchmarkId::new("hashmap", size), &size, |b, &size| {
            let mut map = HashMap::<usize, usize>::from_iter(vec.iter().cloned());
            b.iter(|| {
                for k in 0..size {
                    map.entry(k).and_modify(|x| {
                        *x *= 2;
                    });
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("indexmap", size), &size, |b, &size| {
            let mut map = IndexMap::<usize, usize>::from_iter(vec.iter().cloned());
            b.iter(|| {
                for k in 0..size {
                    map.entry(k).and_modify(|x| {
                        *x *= 2;
                    });
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("hashslabmap", size), &size, |b, &size| {
            let mut map = HashSlabMap::<usize, usize>::from_iter(vec.iter().cloned());
            b.iter(|| {
                for k in 0..size {
                    map.entry(k).and_modify(|x| {
                        *x *= 2;
                    });
                }
            })
        });
    }
}

criterion_group!(benches, bench_entry_or_insert, bench_entry_and_modify);
criterion_main!(benches);
