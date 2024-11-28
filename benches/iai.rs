use std::{collections::HashMap, hint::black_box};

use iai_callgrind::{
    library_benchmark, library_benchmark_group, main, EntryPoint, LibraryBenchmarkConfig,
};

use hashslab::HashSlabMap;
use indexmap::IndexMap;

#[library_benchmark]
#[bench::new()]
fn hashmap_new() -> HashMap<(), ()> {
    black_box(HashMap::new())
}

#[library_benchmark]
#[bench::new()]
fn indexmap_new() -> IndexMap<(), ()> {
    black_box(IndexMap::new())
}

#[library_benchmark]
#[bench::new()]
fn hashslabmap_new() -> HashSlabMap<(), ()> {
    black_box(HashSlabMap::new())
}

#[library_benchmark]
#[benches::with_capacity(args = [1, 100, 10000])]
fn hashmap_with_capacity(cap: usize) -> HashMap<(), ()> {
    black_box(HashMap::with_capacity(cap))
}

#[library_benchmark]
#[benches::with_capacity(args = [1, 100, 10000])]
fn indexmap_with_capacity(cap: usize) -> IndexMap<(), ()> {
    black_box(IndexMap::with_capacity(cap))
}

#[library_benchmark]
#[benches::with_capacity(args = [1, 100, 10000])]
fn hashslabmap_with_capacity(cap: usize) -> HashSlabMap<(), ()> {
    black_box(HashSlabMap::with_capacity(cap))
}

#[library_benchmark(
    config = LibraryBenchmarkConfig::default()
        .entry_point(EntryPoint::Custom("iai::hmap::insert".into()))
)]
#[benches::grow(args = [1, 100, 10000])]
fn hashmap_grow(n: usize) {
    let map = HashMap::new();
    black_box(hmap::insert(map, n));
}

#[library_benchmark(
    config = LibraryBenchmarkConfig::default()
        .entry_point(EntryPoint::Custom("iai::imap::insert".into()))
)]
#[benches::grow(args = [1, 100, 10000])]
fn indexmap_grow(n: usize) {
    let map = IndexMap::new();
    black_box(imap::insert(map, n));
}

#[library_benchmark(
    config = LibraryBenchmarkConfig::default()
        .entry_point(EntryPoint::Custom("iai::hsmap::insert".into()))
)]
#[benches::grow(args = [1, 100, 10000])]
fn hashslabmap_grow(n: usize) {
    let map = HashSlabMap::new();
    black_box(hsmap::insert(map, n));
}

library_benchmark_group!(
    name = allocate;
    compare_by_id = true;
    benchmarks =
        hashmap_new,
        indexmap_new,
        hashslabmap_new,
        hashmap_with_capacity,
        indexmap_with_capacity,
        hashslabmap_with_capacity,
        hashmap_grow,
        indexmap_grow,
        hashslabmap_grow,
);
main!(library_benchmark_groups = allocate);

mod hmap {
    use super::*;

    #[inline(never)]
    pub(crate) fn insert(mut map: HashMap<usize, usize>, n: usize) {
        for x in 0..n {
            black_box(map.insert(x, x));
        }
    }
}

mod imap {
    use super::*;

    #[inline(never)]
    pub(crate) fn insert(mut map: IndexMap<usize, usize>, n: usize) {
        for x in 0..n {
            black_box(map.insert(x, x));
        }
    }
}

mod hsmap {
    use super::*;

    #[inline(never)]
    pub(crate) fn insert(mut map: HashSlabMap<usize, usize>, n: usize) {
        for x in 0..n {
            black_box(map.insert(x, x));
        }
    }
}
