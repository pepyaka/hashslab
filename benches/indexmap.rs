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

// // use lazy_static so that comparison benchmarks use the exact same inputs
// lazy_static! {
//     static ref KEYS: Vec<u32> = shuffled_keys(0..LOOKUP_MAP_SIZE);
// }

// lazy_static! {
//     static ref HMAP_100K: HashMap<u32, u32> = {
//         let mut map = HashMap::with_capacity(LOOKUP_MAP_SIZE as usize);
//         let keys = &*KEYS;
//         for &key in keys {
//             map.insert(key, key);
//         }
//         map
//     };
// }

// lazy_static! {
//     static ref IMAP_100K: IndexMap<u32, u32> = {
//         let mut map = IndexMap::with_capacity(LOOKUP_MAP_SIZE as usize);
//         let keys = &*KEYS;
//         for &key in keys {
//             map.insert(key, key);
//         }
//         map
//     };
// }

// lazy_static! {
//     static ref HSMAP_100K: HashSlabMap<u32, u32> = {
//         let mut map = HashSlabMap::with_capacity(LOOKUP_MAP_SIZE as usize);
//         let keys = &*KEYS;
//         for &key in keys {
//             map.insert(key, key);
//         }
//         map
//     };
// }

// lazy_static! {
//     static ref IMAP_SORT_U32: IndexMap<u32, u32> = {
//         let mut map = IndexMap::with_capacity(SORT_MAP_SIZE);
//         for &key in &KEYS[..SORT_MAP_SIZE] {
//             map.insert(key, key);
//         }
//         map
//     };
// }
// lazy_static! {
//     static ref IMAP_SORT_S: IndexMap<String, String> = {
//         let mut map = IndexMap::with_capacity(SORT_MAP_SIZE);
//         for &key in &KEYS[..SORT_MAP_SIZE] {
//             map.insert(format!("{:^16x}", &key), String::new());
//         }
//         map
//     };
// }

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
