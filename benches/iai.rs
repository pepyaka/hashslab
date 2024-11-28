use std::{hash::Hash, hint::black_box};

use iai_callgrind::{
    library_benchmark, library_benchmark_group, main, EntryPoint, LibraryBenchmarkConfig,
};

use hashslab::HashSlabMap;

#[library_benchmark]
#[bench::small_key(HashSlabMap::new(), 0u8, ())]
#[bench::big_key(HashSlabMap::new(), [0xAAu8; 100], ())]
#[bench::key_value(HashSlabMap::new(), 0u8, [0xAAu8; 100])]
fn hashslabmap_insert<K, V>(mut map: HashSlabMap<K, V>, key: K, value: V)
where
    K: Hash + Eq,
{
    black_box({
        map.insert(key, value);
    })
}

library_benchmark_group!(
    name = insert;
    benchmarks =
        hashslabmap_insert,
);
main!(library_benchmark_groups = insert);
