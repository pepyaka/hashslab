use hashslab::HashSlabMap;
// use itertools::Itertools;

use quickcheck::Arbitrary;
use quickcheck::Gen;
use quickcheck::QuickCheck;
// use quickcheck::TestResult;

use fnv::FnvHasher;
use std::hash::{BuildHasher, BuildHasherDefault};
type FnvBuilder = BuildHasherDefault<FnvHasher>;
type HashSlabMapFnv<K, V> = HashSlabMap<K, V, FnvBuilder>;

use std::cmp::min;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
// use std::ops::Bound;
use std::ops::Deref;

use hashslab::map::Entry;
use std::collections::hash_map::Entry as StdEntry;

fn set<'a, T: 'a, I>(iter: I) -> HashSet<T>
where
    I: IntoIterator<Item = &'a T>,
    T: Copy + Hash + Eq,
{
    iter.into_iter().copied().collect()
}

// fn indexmap<'a, T: 'a, I>(iter: I) -> HashSlabMap<T, ()>
// where
//     I: IntoIterator<Item = &'a T>,
//     T: Copy + Hash + Eq,
// {
//     HashSlabMap::from_iter(iter.into_iter().copied().map(|k| (k, ())))
// }

// Helper macro to allow us to use smaller quickcheck limits under miri.
macro_rules! quickcheck_limit {
    (@as_items $($i:item)*) => ($($i)*);
    {
        $(
            $(#[$m:meta])*
            fn $fn_name:ident($($arg_name:ident : $arg_ty:ty),*) -> $ret:ty {
                $($code:tt)*
            }
        )*
    } => (
        quickcheck::quickcheck! {
            @as_items
            $(
                #[test]
                $(#[$m])*
                fn $fn_name() {
                    fn prop($($arg_name: $arg_ty),*) -> $ret {
                        $($code)*
                    }
                    let mut quickcheck = QuickCheck::new();
                    if cfg!(miri) {
                        quickcheck = quickcheck
                            .gen(Gen::new(10))
                            .tests(10)
                            .max_tests(100);
                    }

                    quickcheck.quickcheck(prop as fn($($arg_ty),*) -> $ret);
                }
            )*
        }
    )
}

quickcheck_limit! {
    fn contains(insert: Vec<u32>) -> bool {
        let mut map = HashSlabMap::new();
        for &key in &insert {
            map.insert(key, ());
        }
        insert.iter().all(|&key| map.get(&key).is_some())
    }

    fn contains_not(insert: Vec<u8>, not: Vec<u8>) -> bool {
        let mut map = HashSlabMap::new();
        for &key in &insert {
            map.insert(key, ());
        }
        let nots = &set(&not) - &set(&insert);
        nots.iter().all(|&key| map.get(&key).is_none())
    }

    fn insert_remove(insert: Vec<u8>, remove: Vec<u8>) -> bool {
        let mut map = HashSlabMap::new();
        for &key in &insert {
            map.insert(key, ());
        }
        for &key in &remove {
            map.remove(&key);
        }
        let elements = &set(&insert) - &set(&remove);
        map.len() == elements.len() && map.iter().count() == elements.len() &&
            elements.iter().all(|k| map.get(k).is_some())
    }

    fn with_cap(template: Vec<()>) -> bool {
        let cap = template.len();
        let map: HashSlabMap<u8, u8> = HashSlabMap::with_capacity(cap);
        println!("wish: {}, got: {} (diff: {})", cap, map.capacity(), map.capacity() as isize - cap as isize);
        map.capacity() >= cap
    }

    fn drain_full(insert: Vec<u8>) -> bool {
        let mut map = HashSlabMap::new();
        for &key in &insert {
            map.insert(key, ());
        }
        let mut clone = map.clone();
        let drained = clone.drain();
        for (key, _) in drained {
            map.remove(&key);
        }
        map.is_empty()
    }

    fn remove(insert: Vec<u8>, remove: Vec<u8>) -> bool {
        let mut map = HashSlabMap::new();
        for &key in &insert {
            map.insert(key, ());
        }
        for &key in &remove {
            map.remove(&key);
        }
        let elements = &set(&insert) - &set(&remove);

        map.len() == elements.len() && map.iter().count() == elements.len() &&
            elements.iter().all(|k| map.get(k).is_some())
    }

    // fn indexing(insert: Vec<u8>) -> bool {
    //     let mut map: HashSlabMap<_, _> = insert.into_iter().map(|x| (x, x)).collect();
    //     let set: IndexSet<_> = map.keys().copied().collect();
    //     assert_eq!(map.len(), set.len());

    //     for (i, &key) in set.iter().enumerate() {
    //         assert_eq!(map.get_index(i), Some((&key, &key)));
    //         assert_eq!(set.get_index(i), Some(&key));
    //         assert_eq!(map[i], key);
    //         assert_eq!(set[i], key);

    //         *map.get_index_mut(i).unwrap().1 >>= 1;
    //         map[i] <<= 1;
    //     }

    //     set.iter().enumerate().all(|(i, &key)| {
    //         let value = key & !1;
    //         map[&key] == value && map[i] == value
    //     })
    // }

}

use crate::Op::*;
#[derive(Copy, Clone, Debug)]
enum Op<K, V> {
    Add(K, V),
    Remove(K),
    AddEntry(K, V),
    RemoveEntry(K),
}

impl<K, V> Arbitrary for Op<K, V>
where
    K: Arbitrary,
    V: Arbitrary,
{
    fn arbitrary(g: &mut Gen) -> Self {
        match u32::arbitrary(g) % 4 {
            0 => Add(K::arbitrary(g), V::arbitrary(g)),
            1 => AddEntry(K::arbitrary(g), V::arbitrary(g)),
            2 => Remove(K::arbitrary(g)),
            _ => RemoveEntry(K::arbitrary(g)),
        }
    }
}

fn do_ops<K, V, S>(ops: &[Op<K, V>], a: &mut HashSlabMap<K, V, S>, b: &mut HashMap<K, V>)
where
    K: Hash + Eq + Clone,
    V: Clone,
    S: BuildHasher,
{
    for op in ops {
        match *op {
            Add(ref k, ref v) => {
                a.insert(k.clone(), v.clone());
                b.insert(k.clone(), v.clone());
            }
            AddEntry(ref k, ref v) => {
                a.entry(k.clone()).or_insert_with(|| v.clone());
                b.entry(k.clone()).or_insert_with(|| v.clone());
            }
            Remove(ref k) => {
                a.remove(k);
                b.remove(k);
            }
            RemoveEntry(ref k) => {
                if let Entry::Occupied(ent) = a.entry(k.clone()) {
                    ent.remove_entry();
                }
                if let StdEntry::Occupied(ent) = b.entry(k.clone()) {
                    ent.remove_entry();
                }
            }
        }
        //println!("{:?}", a);
    }
}

fn assert_maps_equivalent<K, V>(a: &HashSlabMap<K, V>, b: &HashMap<K, V>) -> bool
where
    K: Hash + Eq + Debug,
    V: Eq + Debug,
{
    assert_eq!(a.len(), b.len());
    assert_eq!(a.iter().next().is_some(), b.iter().next().is_some());
    for key in a.keys() {
        assert!(b.contains_key(key), "b does not contain {:?}", key);
    }
    for key in b.keys() {
        assert!(a.get(key).is_some(), "a does not contain {:?}", key);
    }
    for key in a.keys() {
        assert_eq!(a[key], b[key]);
    }
    true
}

quickcheck_limit! {
    fn operations_i8(ops: Large<Vec<Op<i8, i8>>>) -> bool {
        let mut map = HashSlabMap::new();
        let mut reference = HashMap::new();
        do_ops(&ops, &mut map, &mut reference);
        assert_maps_equivalent(&map, &reference)
    }

    fn operations_string(ops: Vec<Op<Alpha, i8>>) -> bool {
        let mut map = HashSlabMap::new();
        let mut reference = HashMap::new();
        do_ops(&ops, &mut map, &mut reference);
        assert_maps_equivalent(&map, &reference)
    }

    fn keys_values(ops: Large<Vec<Op<i8, i8>>>) -> bool {
        let mut map = HashSlabMap::new();
        let mut reference = HashMap::new();
        do_ops(&ops, &mut map, &mut reference);
        let mut visit = HashSlabMap::new();
        for (k, v) in map.keys().zip(map.values()) {
            assert_eq!(&map[k], v);
            assert!(!visit.contains_key(k));
            visit.insert(*k, *v);
        }
        assert_eq!(visit.len(), reference.len());
        true
    }

    fn keys_values_mut(ops: Large<Vec<Op<i8, i8>>>) -> bool {
        let mut map = HashSlabMap::new();
        let mut reference = HashMap::new();
        do_ops(&ops, &mut map, &mut reference);
        let mut visit = HashSlabMap::new();
        let keys = Vec::from_iter(map.keys().copied());
        for (k, v) in keys.iter().zip(map.values_mut()) {
            assert_eq!(&reference[k], v);
            assert!(!visit.contains_key(k));
            visit.insert(*k, *v);
        }
        assert_eq!(visit.len(), reference.len());
        true
    }

    fn equality(ops1: Vec<Op<i8, i8>>, removes: Vec<usize>) -> bool {
        let mut map = HashSlabMap::new();
        let mut reference = HashMap::new();
        do_ops(&ops1, &mut map, &mut reference);
        let mut ops2 = ops1.clone();
        for &r in &removes {
            if !ops2.is_empty() {
                let i = r % ops2.len();
                ops2.remove(i);
            }
        }
        let mut map2 = HashSlabMapFnv::default();
        let mut reference2 = HashMap::new();
        do_ops(&ops2, &mut map2, &mut reference2);
        assert_eq!(map == map2, reference == reference2);
        true
    }

    // fn retain_ordered(keys: Large<Vec<i8>>, remove: Large<Vec<i8>>) -> () {
    //     let mut map = indexmap(keys.iter());
    //     let initial_map = map.clone(); // deduplicated in-order input
    //     let remove_map = indexmap(remove.iter());
    //     let keys_s = set(keys.iter());
    //     let remove_s = set(remove.iter());
    //     let answer = &keys_s - &remove_s;
    //     map.retain(|k, _| !remove_map.contains_key(k));

    //     // check the values
    //     assert_eq!(map.len(), answer.len());
    //     for key in &answer {
    //         assert!(map.contains_key(key));
    //     }
    //     // check the order
    //     itertools::assert_equal(map.keys(), initial_map.keys().filter(|&k| !remove_map.contains_key(k)));
    // }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Alpha(String);

impl Deref for Alpha {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

impl Arbitrary for Alpha {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = usize::arbitrary(g) % g.size();
        let len = min(len, 16);
        Alpha(
            (0..len)
                .map(|_| ALPHABET[usize::arbitrary(g) % ALPHABET.len()] as char)
                .collect(),
        )
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new((**self).shrink().map(Alpha))
    }
}

/// quickcheck Arbitrary adaptor -- make a larger vec
#[derive(Clone, Debug)]
struct Large<T>(T);

impl<T> Deref for Large<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> Arbitrary for Large<Vec<T>>
where
    T: Arbitrary,
{
    fn arbitrary(g: &mut Gen) -> Self {
        let len = usize::arbitrary(g) % (g.size() * 10);
        Large((0..len).map(|_| T::arbitrary(g)).collect())
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new((**self).shrink().map(Large))
    }
}
