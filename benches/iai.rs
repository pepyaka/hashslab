use std::{hash::Hash, hint::black_box};

use iai_callgrind::{library_benchmark, library_benchmark_group, main};

use hashslab::HashSlabMap;

mod util;
use rand::seq::SliceRandom;
use util::small_rng;

#[library_benchmark]
#[bench::small_key(HashSlabMap::new(), 0u8, ())]
#[bench::big_key(HashSlabMap::new(), [0xAAu8; 100], ())]
#[bench::key_value(HashSlabMap::new(), 0u8, [0xAAu8; 100])]
fn hashslabmap_insert<K, V>(mut map: HashSlabMap<K, V>, key: K, value: V) -> Option<V>
where
    K: Hash + Eq,
{
    black_box(map.insert(key, value))
}

fn setup_lookup_key_size(size: usize) -> (HashSlabMap<Vec<u8>, ()>, Vec<Vec<u8>>) {
    let cap = u8::MAX;

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
        .map(|(k, _)| k.clone())
        .take(cap as usize / 10)
        .collect();
    list.shuffle(&mut small_rng());

    let map = HashSlabMap::<Vec<u8>, ()>::from_iter(list);
    (map, lookup)
}

#[library_benchmark]
#[benches::lookup_key_size(args = [1, 100, 10000], setup = setup_lookup_key_size)]
fn lookup_key_size((map, lookup): (HashSlabMap<Vec<u8>, ()>, Vec<Vec<u8>>)) {
    for k in lookup {
        black_box(map.get(&k));
    }
}

library_benchmark_group!(
    name = insert;
    benchmarks =
        hashslabmap_insert,
);
library_benchmark_group!(
    name = lookup;
    benchmarks =
        lookup_key_size,
);
main!(library_benchmark_groups = insert, lookup,);
