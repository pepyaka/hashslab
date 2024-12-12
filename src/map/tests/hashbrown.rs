use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::{cell::RefCell, format, thread_local, vec, vec::Vec};

use fnv::FnvBuildHasher;

use super::Entry::{Occupied, Vacant};
use super::HashSlabMap;

#[test]
fn test_zero_capacities() {
    type HM = HashSlabMap<i32, i32>;

    let m = HM::new();
    assert_eq!(m.capacity(), 0);

    let m = HM::default();
    assert_eq!(m.capacity(), 0);

    let m = HashSlabMap::<u8, u8, _>::with_hasher(FnvBuildHasher::default());
    assert_eq!(m.capacity(), 0);

    let m = HM::with_capacity(0);
    assert_eq!(m.capacity(), 0);

    let m = HashSlabMap::<u8, u8, _>::with_capacity_and_hasher(0, FnvBuildHasher::default());
    assert_eq!(m.capacity(), 0);

    let mut m = HM::new();
    m.insert(1, 1);
    m.insert(2, 2);
    m.remove(&1);
    m.remove(&2);
    m.shrink_to_fit();
    assert_eq!(m.capacity(), 0);

    let mut m = HM::new();
    m.reserve(0);
    assert_eq!(m.capacity(), 0);
}

#[test]
fn test_create_capacity_zero() {
    let mut m = HashSlabMap::with_capacity(0);

    assert!(m.insert(1, 1).is_none());

    assert!(m.contains_key(&1));
    assert!(!m.contains_key(&0));
}

#[test]
fn test_insert() {
    let mut m = HashSlabMap::new();
    assert_eq!(m.len(), 0);
    assert!(m.insert(1, 2).is_none());
    assert_eq!(m.len(), 1);
    assert!(m.insert(2, 4).is_none());
    assert_eq!(m.len(), 2);
    assert_eq!(*m.get(&1).unwrap(), 2);
    assert_eq!(*m.get(&2).unwrap(), 4);
}

#[test]
fn test_clone() {
    let mut m = HashSlabMap::new();
    assert_eq!(m.len(), 0);
    assert!(m.insert(1, 2).is_none());
    assert_eq!(m.len(), 1);
    assert!(m.insert(2, 4).is_none());
    assert_eq!(m.len(), 2);
    #[allow(clippy::redundant_clone)]
    let m2 = m.clone();
    assert_eq!(*m2.get(&1).unwrap(), 2);
    assert_eq!(*m2.get(&2).unwrap(), 4);
    assert_eq!(m2.len(), 2);
}

#[test]
fn test_clone_from() {
    let mut m = HashSlabMap::new();
    let mut m2 = HashSlabMap::new();
    assert_eq!(m.len(), 0);
    assert!(m.insert(1, 2).is_none());
    assert_eq!(m.len(), 1);
    assert!(m.insert(2, 4).is_none());
    assert_eq!(m.len(), 2);
    m2.clone_from(&m);
    assert_eq!(*m2.get(&1).unwrap(), 2);
    assert_eq!(*m2.get(&2).unwrap(), 4);
    assert_eq!(m2.len(), 2);
}

thread_local! { static DROP_VECTOR: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) } }

#[derive(Hash, PartialEq, Eq)]
struct Droppable {
    k: usize,
}

impl Droppable {
    fn new(k: usize) -> Droppable {
        DROP_VECTOR.with(|slot| {
            slot.borrow_mut()[k] += 1;
        });

        Droppable { k }
    }
}

impl Drop for Droppable {
    fn drop(&mut self) {
        DROP_VECTOR.with(|slot| {
            slot.borrow_mut()[self.k] -= 1;
        });
    }
}

impl Clone for Droppable {
    fn clone(&self) -> Self {
        Droppable::new(self.k)
    }
}

#[test]
fn test_drops() {
    DROP_VECTOR.with(|slot| {
        *slot.borrow_mut() = vec![0; 200];
    });

    {
        let mut m = HashSlabMap::new();

        DROP_VECTOR.with(|v| {
            for i in 0..200 {
                assert_eq!(v.borrow()[i], 0);
            }
        });

        for i in 0..100 {
            let d1 = Droppable::new(i);
            let d2 = Droppable::new(i + 100);
            m.insert(d1, d2);
        }

        DROP_VECTOR.with(|v| {
            for i in 0..200 {
                assert_eq!(v.borrow()[i], 1);
            }
        });

        for i in 0..50 {
            let k = Droppable::new(i);
            let v = m.remove(&k);

            assert!(v.is_some());

            DROP_VECTOR.with(|v| {
                assert_eq!(v.borrow()[i], 1);
                assert_eq!(v.borrow()[i + 100], 1);
            });
        }

        DROP_VECTOR.with(|v| {
            for i in 0..50 {
                assert_eq!(v.borrow()[i], 0);
                assert_eq!(v.borrow()[i + 100], 0);
            }

            for i in 50..100 {
                assert_eq!(v.borrow()[i], 1);
                assert_eq!(v.borrow()[i + 100], 1);
            }
        });
    }

    DROP_VECTOR.with(|v| {
        for i in 0..200 {
            assert_eq!(v.borrow()[i], 0);
        }
    });
}

#[test]
fn test_into_iter_drops() {
    DROP_VECTOR.with(|v| {
        *v.borrow_mut() = vec![0; 200];
    });

    let hm = {
        let mut hm = HashSlabMap::new();

        DROP_VECTOR.with(|v| {
            for i in 0..200 {
                assert_eq!(v.borrow()[i], 0);
            }
        });

        for i in 0..100 {
            let d1 = Droppable::new(i);
            let d2 = Droppable::new(i + 100);
            hm.insert(d1, d2);
        }

        DROP_VECTOR.with(|v| {
            for i in 0..200 {
                assert_eq!(v.borrow()[i], 1);
            }
        });

        hm
    };

    // By the way, ensure that cloning doesn't screw up the dropping.
    drop(hm.clone());

    {
        let mut half = hm.into_iter().take(50);

        DROP_VECTOR.with(|v| {
            for i in 0..200 {
                assert_eq!(v.borrow()[i], 1);
            }
        });

        for _ in half.by_ref() {}

        DROP_VECTOR.with(|v| {
            let nk = (0..100).filter(|&i| v.borrow()[i] == 1).count();

            let nv = (0..100).filter(|&i| v.borrow()[i + 100] == 1).count();

            assert_eq!(nk, 50);
            assert_eq!(nv, 50);
        });
    };

    DROP_VECTOR.with(|v| {
        for i in 0..200 {
            assert_eq!(v.borrow()[i], 0);
        }
    });
}

#[test]
fn test_empty_remove() {
    let mut m: HashSlabMap<i32, bool> = HashSlabMap::new();
    assert_eq!(m.remove(&0), None);
}

#[test]
fn test_empty_entry() {
    let mut m: HashSlabMap<i32, bool> = HashSlabMap::new();
    match m.entry(0) {
        Occupied(_) => panic!(),
        Vacant(_) => {}
    }
    assert!(*m.entry(0).or_insert(true));
    assert_eq!(m.len(), 1);
}

// #[test]
// fn test_empty_entry_ref() {
//     let mut m: HashSlabMap<std::string::String, bool> = HashSlabMap::new();
//     match m.entry_ref("poneyland") {
//         EntryRef::Occupied(_) => panic!(),
//         EntryRef::Vacant(_) => {}
//     }
//     assert!(*m.entry_ref("poneyland").or_insert(true));
//     assert_eq!(m.len(), 1);
// }

#[test]
fn test_empty_iter() {
    let mut m: HashSlabMap<i32, bool> = HashSlabMap::new();
    assert_eq!(m.drain().next(), None);
    assert_eq!(m.keys().next(), None);
    assert_eq!(m.values().next(), None);
    assert_eq!(m.values_mut().next(), None);
    assert_eq!(m.iter().next(), None);
    assert_eq!(m.iter_mut().next(), None);
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert_eq!(m.into_iter().next(), None);
}

#[test]
#[cfg_attr(miri, ignore)] // FIXME: takes too long
fn test_lots_of_insertions() {
    let mut m = HashSlabMap::new();

    // Try this a few times to make sure we never screw up the hashmap's
    // internal state.
    for _ in 0..10 {
        assert!(m.is_empty());

        for i in 1..1001 {
            assert!(m.insert(i, i).is_none());

            for j in 1..=i {
                let r = m.get(&j);
                assert_eq!(r, Some(&j));
            }

            for j in i + 1..1001 {
                let r = m.get(&j);
                assert_eq!(r, None);
            }
        }

        for i in 1001..2001 {
            assert!(!m.contains_key(&i));
        }

        // remove forwards
        for i in 1..1001 {
            assert!(m.remove(&i).is_some());

            for j in 1..=i {
                assert!(!m.contains_key(&j));
            }

            for j in i + 1..1001 {
                assert!(m.contains_key(&j));
            }
        }

        for i in 1..1001 {
            assert!(!m.contains_key(&i));
        }

        for i in 1..1001 {
            assert!(m.insert(i, i).is_none());
        }

        // remove backwards
        for i in (1..1001).rev() {
            assert!(m.remove(&i).is_some());

            for j in i..1001 {
                assert!(!m.contains_key(&j));
            }

            for j in 1..i {
                assert!(m.contains_key(&j));
            }
        }
    }
}

#[test]
fn test_find_mut() {
    let mut m = HashSlabMap::new();
    assert!(m.insert(1, 12).is_none());
    assert!(m.insert(2, 8).is_none());
    assert!(m.insert(5, 14).is_none());
    let new = 100;
    match m.get_mut(&5) {
        None => panic!(),
        Some(x) => *x = new,
    }
    assert_eq!(m.get(&5), Some(&new));
}

#[test]
fn test_insert_overwrite() {
    let mut m = HashSlabMap::new();
    assert!(m.insert(1, 2).is_none());
    assert_eq!(*m.get(&1).unwrap(), 2);
    assert!(m.insert(1, 3).is_some());
    assert_eq!(*m.get(&1).unwrap(), 3);
}

#[test]
fn test_insert_conflicts() {
    let mut m = HashSlabMap::with_capacity(4);
    assert!(m.insert(1, 2).is_none());
    assert!(m.insert(5, 3).is_none());
    assert!(m.insert(9, 4).is_none());
    assert_eq!(*m.get(&9).unwrap(), 4);
    assert_eq!(*m.get(&5).unwrap(), 3);
    assert_eq!(*m.get(&1).unwrap(), 2);
}

#[test]
fn test_conflict_remove() {
    let mut m = HashSlabMap::with_capacity(4);
    assert!(m.insert(1, 2).is_none());
    assert_eq!(*m.get(&1).unwrap(), 2);
    assert!(m.insert(5, 3).is_none());
    assert_eq!(*m.get(&1).unwrap(), 2);
    assert_eq!(*m.get(&5).unwrap(), 3);
    assert!(m.insert(9, 4).is_none());
    assert_eq!(*m.get(&1).unwrap(), 2);
    assert_eq!(*m.get(&5).unwrap(), 3);
    assert_eq!(*m.get(&9).unwrap(), 4);
    assert!(m.remove(&1).is_some());
    assert_eq!(*m.get(&9).unwrap(), 4);
    assert_eq!(*m.get(&5).unwrap(), 3);
}

// #[test]
// fn test_insert_unique_unchecked() {
//     let mut map = HashSlabMap::new();
//     let (k1, v1) = unsafe { map.insert_unique_unchecked(10, 11) };
//     assert_eq!((&10, &mut 11), (k1, v1));
//     let (k2, v2) = unsafe { map.insert_unique_unchecked(20, 21) };
//     assert_eq!((&20, &mut 21), (k2, v2));
//     assert_eq!(Some(&11), map.get(&10));
//     assert_eq!(Some(&21), map.get(&20));
//     assert_eq!(None, map.get(&30));
// }

#[test]
fn test_is_empty() {
    let mut m = HashSlabMap::with_capacity(4);
    assert!(m.insert(1, 2).is_none());
    assert!(!m.is_empty());
    assert!(m.remove(&1).is_some());
    assert!(m.is_empty());
}

#[test]
fn test_remove() {
    let mut m = HashSlabMap::new();
    m.insert(1, 2);
    assert_eq!(m.remove(&1), Some(2));
    assert_eq!(m.remove(&1), None);
}

#[test]
fn test_remove_entry() {
    let mut m = HashSlabMap::new();
    m.insert(1, 2);
    assert_eq!(m.remove_entry(&1), Some((1, 2)));
    assert_eq!(m.remove(&1), None);
}

#[test]
fn test_iterate() {
    let mut m = HashSlabMap::with_capacity(4);
    for i in 0..32 {
        assert!(m.insert(i, i * 2).is_none());
    }
    assert_eq!(m.len(), 32);

    let mut observed: u32 = 0;

    for (k, v) in &m {
        assert_eq!(*v, *k * 2);
        observed |= 1 << *k;
    }
    assert_eq!(observed, 0xFFFF_FFFF);
}

#[test]
fn test_keys() {
    let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
    let map: HashSlabMap<_, _> = vec.into_iter().collect();
    let keys: Vec<_> = map.keys().copied().collect();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&1));
    assert!(keys.contains(&2));
    assert!(keys.contains(&3));
}

#[test]
fn test_values() {
    let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
    let map: HashSlabMap<_, _> = vec.into_iter().collect();
    let values: Vec<_> = map.values().copied().collect();
    assert_eq!(values.len(), 3);
    assert!(values.contains(&'a'));
    assert!(values.contains(&'b'));
    assert!(values.contains(&'c'));
}

#[test]
fn test_values_mut() {
    let vec = vec![(1, 1), (2, 2), (3, 3)];
    let mut map: HashSlabMap<_, _> = vec.into_iter().collect();
    for value in map.values_mut() {
        *value *= 2;
    }
    let values: Vec<_> = map.values().copied().collect();
    assert_eq!(values.len(), 3);
    assert!(values.contains(&2));
    assert!(values.contains(&4));
    assert!(values.contains(&6));
}

#[test]
fn test_into_keys() {
    let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
    let map: HashSlabMap<_, _> = vec.into_iter().collect();
    let keys: Vec<_> = map.into_keys().collect();

    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&1));
    assert!(keys.contains(&2));
    assert!(keys.contains(&3));
}

#[test]
fn test_into_values() {
    let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
    let map: HashSlabMap<_, _> = vec.into_iter().collect();
    let values: Vec<_> = map.into_values().collect();

    assert_eq!(values.len(), 3);
    assert!(values.contains(&'a'));
    assert!(values.contains(&'b'));
    assert!(values.contains(&'c'));
}

#[test]
fn test_find() {
    let mut m = HashSlabMap::new();
    assert!(m.get(&1).is_none());
    m.insert(1, 2);
    match m.get(&1) {
        None => panic!(),
        Some(v) => assert_eq!(*v, 2),
    }
}

#[test]
fn test_eq() {
    let mut m1 = HashSlabMap::new();
    m1.insert(1, 2);
    m1.insert(2, 3);
    m1.insert(3, 4);

    let mut m2 = HashSlabMap::new();
    m2.insert(1, 2);
    m2.insert(2, 3);

    assert!(m1 != m2);

    m2.insert(3, 4);

    assert_eq!(m1, m2);
}

#[test]
fn test_show() {
    let mut map = HashSlabMap::new();
    let empty: HashSlabMap<i32, i32> = HashSlabMap::new();

    map.insert(1, 2);
    map.insert(3, 4);

    let map_str = format!("{map:?}");

    assert!(
        map_str == "{0: (1, 2), 1: (3, 4)}" || map_str == "{1: (3, 4), 0: (1, 2)}",
        "{map_str}"
    );
    assert_eq!(format!("{empty:?}"), "{}");
}

#[test]
fn test_reserve_shrink_to_fit() {
    let mut m = HashSlabMap::new();
    m.insert(0, 0);
    m.remove(&0);
    assert!(m.capacity() >= m.len());
    for i in 0..128 {
        m.insert(i, i);
    }
    m.reserve(256);

    let usable_cap = m.capacity();
    for i in 128..(128 + 256) {
        m.insert(i, i);
        assert_eq!(m.capacity(), usable_cap);
    }

    for i in 100..(128 + 256) {
        assert_eq!(m.remove(&i), Some(i));
    }
    m.shrink_to_fit();

    assert_eq!(m.len(), 100);
    assert!(!m.is_empty());
    assert!(m.capacity() >= m.len());

    for i in 0..100 {
        assert_eq!(m.remove(&i), Some(i));
    }
    m.shrink_to_fit();
    m.insert(0, 0);

    assert_eq!(m.len(), 1);
    assert!(m.capacity() >= m.len());
    assert_eq!(m.remove(&0), Some(0));
}

#[test]
fn test_from_iter() {
    let xs = [(1, 1), (2, 2), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

    let map: HashSlabMap<_, _> = xs.iter().copied().collect();

    for &(k, v) in &xs {
        assert_eq!(map.get(&k), Some(&v));
    }

    assert_eq!(map.iter().len(), xs.len() - 1);
}

#[test]
fn test_size_hint() {
    let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

    let map: HashSlabMap<_, _> = xs.iter().copied().collect();

    let mut iter = map.iter();

    for _ in iter.by_ref().take(3) {}

    assert_eq!(iter.size_hint(), (3, Some(3)));
}

#[test]
fn test_iter_len() {
    let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

    let map: HashSlabMap<_, _> = xs.iter().copied().collect();

    let mut iter = map.iter();

    for _ in iter.by_ref().take(3) {}

    assert_eq!(iter.len(), 3);
}

#[test]
fn test_mut_size_hint() {
    let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

    let mut map: HashSlabMap<_, _> = xs.iter().copied().collect();

    let mut iter = map.iter_mut();

    for _ in iter.by_ref().take(3) {}

    assert_eq!(iter.size_hint(), (3, Some(3)));
}

#[test]
fn test_iter_mut_len() {
    let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

    let mut map: HashSlabMap<_, _> = xs.iter().copied().collect();

    let mut iter = map.iter_mut();

    for _ in iter.by_ref().take(3) {}

    assert_eq!(iter.len(), 3);
}

#[test]
fn test_index() {
    let mut map = HashSlabMap::new();

    map.insert(1, 2);
    map.insert(2, 1);
    map.insert(3, 4);

    assert_eq!(map[&2], 1);
}

#[test]
#[should_panic]
fn test_index_nonexistent() {
    let mut map = HashSlabMap::new();

    map.insert(1, 2);
    map.insert(2, 1);
    map.insert(3, 4);

    #[allow(clippy::no_effect)] // false positive lint
    map[&4];
}

#[test]
fn test_entry() {
    let xs = [(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)];

    let mut map: HashSlabMap<_, _> = xs.iter().copied().collect();

    // Existing key (insert)
    match map.entry(1) {
        Vacant(_) => unreachable!(),
        Occupied(mut view) => {
            assert_eq!(view.get(), &10);
            assert_eq!(view.insert(100), 10);
        }
    }
    assert_eq!(map.get(&1).unwrap(), &100);
    assert_eq!(map.len(), 6);

    // Existing key (update)
    match map.entry(2) {
        Vacant(_) => unreachable!(),
        Occupied(mut view) => {
            let v = view.get_mut();
            let new_v = (*v) * 10;
            *v = new_v;
        }
    }
    assert_eq!(map.get(&2).unwrap(), &200);
    assert_eq!(map.len(), 6);

    // Existing key (take)
    match map.entry(3) {
        Vacant(_) => unreachable!(),
        Occupied(view) => {
            assert_eq!(view.remove(), 30);
        }
    }
    assert_eq!(map.get(&3), None);
    assert_eq!(map.len(), 5);

    // Inexistent key (insert)
    match map.entry(10) {
        Occupied(_) => unreachable!(),
        Vacant(view) => {
            assert_eq!(*view.insert(1000), 1000);
        }
    }
    assert_eq!(map.get(&10).unwrap(), &1000);
    assert_eq!(map.len(), 6);
}

// #[test]
// fn test_entry_ref() {
//     let xs = [
//         ("One".to_owned(), 10),
//         ("Two".to_owned(), 20),
//         ("Three".to_owned(), 30),
//         ("Four".to_owned(), 40),
//         ("Five".to_owned(), 50),
//         ("Six".to_owned(), 60),
//     ];

//     let mut map: HashSlabMap<_, _> = xs.iter().cloned().collect();

//     // Existing key (insert)
//     match map.entry_ref("One") {
//         EntryRef::Vacant(_) => unreachable!(),
//         EntryRef::Occupied(mut view) => {
//             assert_eq!(view.get(), &10);
//             assert_eq!(view.insert(100), 10);
//         }
//     }
//     assert_eq!(map.get("One").unwrap(), &100);
//     assert_eq!(map.len(), 6);

//     // Existing key (update)
//     match map.entry_ref("Two") {
//         EntryRef::Vacant(_) => unreachable!(),
//         EntryRef::Occupied(mut view) => {
//             let v = view.get_mut();
//             let new_v = (*v) * 10;
//             *v = new_v;
//         }
//     }
//     assert_eq!(map.get("Two").unwrap(), &200);
//     assert_eq!(map.len(), 6);

//     // Existing key (take)
//     match map.entry_ref("Three") {
//         EntryRef::Vacant(_) => unreachable!(),
//         EntryRef::Occupied(view) => {
//             assert_eq!(view.remove(), 30);
//         }
//     }
//     assert_eq!(map.get("Three"), None);
//     assert_eq!(map.len(), 5);

//     // Inexistent key (insert)
//     match map.entry_ref("Ten") {
//         EntryRef::Occupied(_) => unreachable!(),
//         EntryRef::Vacant(view) => {
//             assert_eq!(*view.insert(1000), 1000);
//         }
//     }
//     assert_eq!(map.get("Ten").unwrap(), &1000);
//     assert_eq!(map.len(), 6);
// }

#[test]
fn test_entry_take_doesnt_corrupt() {
    #![allow(deprecated)] //rand
                          // Test for #19292
    fn check(m: &HashSlabMap<i32, ()>) {
        for k in m.keys() {
            assert!(m.contains_key(k), "{k} is in keys() but not in the map?");
        }
    }

    let mut m = HashSlabMap::new();

    let mut rng = {
        let seed = u64::from_le_bytes(*b"testseed");
        SmallRng::seed_from_u64(seed)
    };

    // Populate the map with some items.
    for _ in 0..50 {
        let x = rng.gen_range(-10..10);
        m.insert(x, ());
    }

    for _ in 0..1000 {
        let x = rng.gen_range(-10..10);
        match m.entry(x) {
            Vacant(_) => {}
            Occupied(e) => {
                e.remove();
            }
        }

        check(&m);
    }
}

// #[test]
// fn test_entry_ref_take_doesnt_corrupt() {
//     #![allow(deprecated)] //rand
//                           // Test for #19292
//     fn check(m: &HashSlabMap<std::string::String, ()>) {
//         for k in m.keys() {
//             assert!(m.contains_key(k), "{k} is in keys() but not in the map?");
//         }
//     }

//     let mut m = HashSlabMap::new();

//     let mut rng = {
//         let seed = u64::from_le_bytes(*b"testseed");
//         SmallRng::seed_from_u64(seed)
//     };

//     // Populate the map with some items.
//     for _ in 0..50 {
//         let mut x = std::string::String::with_capacity(1);
//         x.push(rng.gen_range('a'..='z'));
//         m.insert(x, ());
//     }

//     for _ in 0..1000 {
//         let mut x = std::string::String::with_capacity(1);
//         x.push(rng.gen_range('a'..='z'));
//         match m.entry_ref(x.as_str()) {
//             EntryRef::Vacant(_) => {}
//             EntryRef::Occupied(e) => {
//                 e.remove();
//             }
//         }

//         check(&m);
//     }
// }

#[test]
fn test_extend_ref_k_ref_v() {
    let mut a = HashSlabMap::new();
    a.insert(1, "one");
    let mut b = HashSlabMap::new();
    b.insert(2, "two");
    b.insert(3, "three");

    a.extend(&b);

    assert_eq!(a.len(), 3);
    assert_eq!(a[&1], "one");
    assert_eq!(a[&2], "two");
    assert_eq!(a[&3], "three");
}

#[test]
#[allow(clippy::needless_borrow)]
fn test_extend_ref_kv_tuple() {
    use std::ops::AddAssign;
    let mut a = HashSlabMap::new();
    a.insert(0, 0);

    fn create_arr<T: AddAssign<T> + Copy, const N: usize>(start: T, step: T) -> [(T, T); N] {
        let mut outs: [(T, T); N] = [(start, start); N];
        let mut element = step;
        outs.iter_mut().skip(1).for_each(|(k, v)| {
            *k += element;
            *v += element;
            element += step;
        });
        outs
    }

    let for_iter: Vec<_> = (0..100).map(|i| (i, i)).collect();
    let iter = for_iter.iter();
    let vec: Vec<_> = (100..200).map(|i| (i, i)).collect();
    a.extend(iter);
    a.extend(&vec);
    a.extend(create_arr::<i32, 100>(200, 1));

    assert_eq!(a.len(), 300);

    for item in 0..300 {
        assert_eq!(a[&item], item);
    }
}

#[test]
fn test_capacity_not_less_than_len() {
    let mut a = HashSlabMap::new();
    let mut item = 0;

    for _ in 0..116 {
        a.insert(item, 0);
        item += 1;
    }

    assert!(a.capacity() > a.len());

    let free = a.capacity() - a.len();
    for _ in 0..free {
        a.insert(item, 0);
        item += 1;
    }

    assert_eq!(a.len(), a.capacity());

    // Insert at capacity should cause allocation.
    a.insert(item, 0);
    assert!(a.capacity() > a.len());
}

#[test]
fn test_occupied_entry_key() {
    let mut a = HashSlabMap::new();
    let key = "hello there";
    let value = "value goes here";
    assert!(a.is_empty());
    a.insert(key, value);
    assert_eq!(a.len(), 1);
    assert_eq!(a[key], value);

    match a.entry(key) {
        Vacant(_) => panic!(),
        Occupied(e) => assert_eq!(key, *e.key()),
    }
    assert_eq!(a.len(), 1);
    assert_eq!(a[key], value);
}

// #[test]
// fn test_occupied_entry_ref_key() {
//     let mut a = HashSlabMap::new();
//     let key = "hello there";
//     let value = "value goes here";
//     assert!(a.is_empty());
//     a.insert(key.to_owned(), value);
//     assert_eq!(a.len(), 1);
//     assert_eq!(a[key], value);

//     match a.entry_ref(key) {
//         EntryRef::Vacant(_) => panic!(),
//         EntryRef::Occupied(e) => assert_eq!(key, e.key()),
//     }
//     assert_eq!(a.len(), 1);
//     assert_eq!(a[key], value);
// }

#[test]
fn test_vacant_entry_key() {
    let mut a = HashSlabMap::new();
    let key = "hello there";
    let value = "value goes here";

    assert!(a.is_empty());
    match a.entry(key) {
        Occupied(_) => panic!(),
        Vacant(e) => {
            assert_eq!(key, *e.key());
            e.insert(value);
        }
    }
    assert_eq!(a.len(), 1);
    assert_eq!(a[key], value);
}

// #[test]
// fn test_vacant_entry_ref_key() {
//     let mut a: HashSlabMap<std::string::String, &str> = HashSlabMap::new();
//     let key = "hello there";
//     let value = "value goes here";

//     assert!(a.is_empty());
//     match a.entry_ref(key) {
//         EntryRef::Occupied(_) => panic!(),
//         EntryRef::Vacant(e) => {
//             assert_eq!(key, e.key());
//             e.insert(value);
//         }
//     }
//     assert_eq!(a.len(), 1);
//     assert_eq!(a[key], value);
// }

// #[test]
// fn test_occupied_entry_replace_entry_with() {
//     let mut a = HashSlabMap::new();

//     let key = "a key";
//     let value = "an initial value";
//     let new_value = "a new value";

//     let entry = a.entry(key).insert_entry(value).replace_entry_with(|k, v| {
//         assert_eq!(k, &key);
//         assert_eq!(v, value);
//         Some(new_value)
//     });

//     match entry {
//         Occupied(e) => {
//             assert_eq!(e.key(), &key);
//             assert_eq!(e.get(), &new_value);
//         }
//         Vacant(_) => panic!(),
//     }

//     assert_eq!(a[key], new_value);
//     assert_eq!(a.len(), 1);

//     let entry = match a.entry(key) {
//         Occupied(e) => e.replace_entry_with(|k, v| {
//             assert_eq!(k, &key);
//             assert_eq!(v, new_value);
//             None
//         }),
//         Vacant(_) => panic!(),
//     };

//     match entry {
//         Vacant(e) => assert_eq!(e.key(), &key),
//         Occupied(_) => panic!(),
//     }

//     assert!(!a.contains_key(key));
//     assert_eq!(a.len(), 0);
// }

// #[test]
// fn test_entry_and_replace_entry_with() {
//     let mut a = HashSlabMap::new();

//     let key = "a key";
//     let value = "an initial value";
//     let new_value = "a new value";

//     let entry = a.entry(key).and_replace_entry_with(|_, _| panic!());

//     match entry {
//         Vacant(e) => assert_eq!(e.key(), &key),
//         Occupied(_) => panic!(),
//     }

//     a.insert(key, value);

//     let entry = a.entry(key).and_replace_entry_with(|k, v| {
//         assert_eq!(k, &key);
//         assert_eq!(v, value);
//         Some(new_value)
//     });

//     match entry {
//         Occupied(e) => {
//             assert_eq!(e.key(), &key);
//             assert_eq!(e.get(), &new_value);
//         }
//         Vacant(_) => panic!(),
//     }

//     assert_eq!(a[key], new_value);
//     assert_eq!(a.len(), 1);

//     let entry = a.entry(key).and_replace_entry_with(|k, v| {
//         assert_eq!(k, &key);
//         assert_eq!(v, new_value);
//         None
//     });

//     match entry {
//         Vacant(e) => assert_eq!(e.key(), &key),
//         Occupied(_) => panic!(),
//     }

//     assert!(!a.contains_key(key));
//     assert_eq!(a.len(), 0);
// }

// #[test]
// fn test_replace_entry_with_doesnt_corrupt() {
//     #![allow(deprecated)] //rand
//                           // Test for #19292
//     fn check(m: &HashSlabMap<i32, ()>) {
//         for k in m.keys() {
//             assert!(m.contains_key(k), "{k} is in keys() but not in the map?");
//         }
//     }

//     let mut m = HashSlabMap::new();

//     let mut rng = {
//         let seed = u64::from_le_bytes(*b"testseed");
//         SmallRng::seed_from_u64(seed)
//     };

//     // Populate the map with some items.
//     for _ in 0..50 {
//         let x = rng.gen_range(-10..10);
//         m.insert(x, ());
//     }

//     for _ in 0..1000 {
//         let x = rng.gen_range(-10..10);
//         m.entry(x).and_replace_entry_with(|_, _| None);
//         check(&m);
//     }
// }

// #[test]
// fn test_retain() {
//     let mut map: HashSlabMap<i32, i32> = (0..100).map(|x| (x, x * 10)).collect();

//     map.retain(|&k, _| k % 2 == 0);
//     assert_eq!(map.len(), 50);
//     assert_eq!(map[&2], 20);
//     assert_eq!(map[&4], 40);
//     assert_eq!(map[&6], 60);
// }

// #[test]
// fn test_extract_if() {
//     {
//         let mut map: HashSlabMap<i32, i32> = (0..8).map(|x| (x, x * 10)).collect();
//         let drained = map.extract_if(|&k, _| k % 2 == 0);
//         let mut out = drained.collect::<Vec<_>>();
//         out.sort_unstable();
//         assert_eq!(vec![(0, 0), (2, 20), (4, 40), (6, 60)], out);
//         assert_eq!(map.len(), 4);
//     }
//     {
//         let mut map: HashSlabMap<i32, i32> = (0..8).map(|x| (x, x * 10)).collect();
//         map.extract_if(|&k, _| k % 2 == 0).for_each(drop);
//         assert_eq!(map.len(), 4);
//     }
// }

#[test]
#[cfg_attr(miri, ignore)] // FIXME: no OOM signalling (https://github.com/rust-lang/miri/issues/613)
fn test_try_reserve() {
    use crate::TryReserveError::{AllocError, CapacityOverflow};

    const MAX_ISIZE: usize = isize::MAX as usize;

    let mut empty_bytes: HashSlabMap<u8, u8> = HashSlabMap::new();

    if let Err(CapacityOverflow) = empty_bytes.try_reserve(usize::MAX) {
    } else {
        panic!("usize::MAX should trigger an overflow!");
    }

    if let Err(CapacityOverflow) = empty_bytes.try_reserve(MAX_ISIZE) {
    } else {
        panic!("isize::MAX should trigger an overflow!");
    }

    let mut vec = Vec::new();
    const SUCC_SIZE: usize = MAX_ISIZE / 256;
    for n in 0.. {
        let mut empty_bytes: HashSlabMap<u8, u8> = HashSlabMap::new();
        match empty_bytes.try_reserve(SUCC_SIZE) {
            Ok(()) => {}
            Err(AllocError { .. }) => break,
            Err(err) => panic!("iter #{n}: {SUCC_SIZE} should trigger an OOM, not {err:?}"),
        }
        vec.push(empty_bytes);
    }
}

// #[test]
// fn test_const_with_hasher() {
//     use core::hash::BuildHasher;
//     use std::collections::hash_map::DefaultHasher;

//     #[derive(Clone)]
//     struct MyHasher;
//     impl BuildHasher for MyHasher {
//         type Hasher = DefaultHasher;

//         fn build_hasher(&self) -> DefaultHasher {
//             DefaultHasher::new()
//         }
//     }

//     const EMPTY_MAP: HashSlabMap<u32, std::string::String, MyHasher> =
//         HashSlabMap::with_hasher(MyHasher);

//     let mut map = EMPTY_MAP;
//     map.insert(17, "seventeen".to_owned());
//     assert_eq!("seventeen", map[&17]);
// }

// #[test]
// fn test_get_many_mut() {
//     let mut map = HashSlabMap::new();
//     map.insert("foo".to_owned(), 0);
//     map.insert("bar".to_owned(), 10);
//     map.insert("baz".to_owned(), 20);
//     map.insert("qux".to_owned(), 30);

//     let xs = map.get_many_mut(["foo", "qux"]);
//     assert_eq!(xs, [Some(&mut 0), Some(&mut 30)]);

//     let xs = map.get_many_mut(["foo", "dud"]);
//     assert_eq!(xs, [Some(&mut 0), None]);

//     let ys = map.get_many_key_value_mut(["bar", "baz"]);
//     assert_eq!(
//         ys,
//         [
//             Some((&"bar".to_owned(), &mut 10)),
//             Some((&"baz".to_owned(), &mut 20))
//         ],
//     );

//     let ys = map.get_many_key_value_mut(["bar", "dip"]);
//     assert_eq!(ys, [Some((&"bar".to_string(), &mut 10)), None]);
// }

// #[test]
// #[should_panic = "duplicate keys found"]
// fn test_get_many_mut_duplicate() {
//     let mut map = HashSlabMap::new();
//     map.insert("foo".to_owned(), 0);

//     let _xs = map.get_many_mut(["foo", "foo"]);
// }

#[test]
#[should_panic = "panic in drop"]
fn test_clone_from_double_drop() {
    #[derive(Clone)]
    struct CheckedDrop {
        panic_in_drop: bool,
        dropped: bool,
    }
    impl Drop for CheckedDrop {
        fn drop(&mut self) {
            if self.panic_in_drop {
                self.dropped = true;
                panic!("panic in drop");
            }
            if self.dropped {
                panic!("double drop");
            }
            self.dropped = true;
        }
    }
    const DISARMED: CheckedDrop = CheckedDrop {
        panic_in_drop: false,
        dropped: false,
    };
    const ARMED: CheckedDrop = CheckedDrop {
        panic_in_drop: true,
        dropped: false,
    };

    let mut map1 = HashSlabMap::new();
    map1.insert(1, DISARMED);
    map1.insert(2, DISARMED);
    map1.insert(3, DISARMED);
    map1.insert(4, DISARMED);

    let mut map2 = HashSlabMap::new();
    map2.insert(1, DISARMED);
    map2.insert(2, ARMED);
    map2.insert(3, DISARMED);
    map2.insert(4, DISARMED);

    map2.clone_from(&map1);
}

#[test]
#[should_panic = "panic in clone"]
fn test_clone_from_memory_leaks() {
    struct CheckedClone {
        panic_in_clone: bool,
        need_drop: Vec<i32>,
    }
    impl Clone for CheckedClone {
        fn clone(&self) -> Self {
            if self.panic_in_clone {
                panic!("panic in clone")
            }
            Self {
                panic_in_clone: self.panic_in_clone,
                need_drop: self.need_drop.clone(),
            }
        }
    }
    let mut map1 = HashSlabMap::new();
    map1.insert(
        1,
        CheckedClone {
            panic_in_clone: false,
            need_drop: vec![0, 1, 2],
        },
    );
    map1.insert(
        2,
        CheckedClone {
            panic_in_clone: false,
            need_drop: vec![3, 4, 5],
        },
    );
    map1.insert(
        3,
        CheckedClone {
            panic_in_clone: true,
            need_drop: vec![6, 7, 8],
        },
    );
    let _map2 = map1.clone();
}

// const DISARMED: bool = false;
// const ARMED: bool = true;

// const ARMED_FLAGS: [bool; 8] = [
//     DISARMED, DISARMED, DISARMED, ARMED, DISARMED, DISARMED, DISARMED, DISARMED,
// ];

// const DISARMED_FLAGS: [bool; 8] = [
//     DISARMED, DISARMED, DISARMED, DISARMED, DISARMED, DISARMED, DISARMED, DISARMED,
// ];

// #[test]
// fn test_allocation_info() {
//     assert_eq!(HashSlabMap::<(), ()>::new().allocation_size(), 0);
//     assert_eq!(HashSlabMap::<u32, u32>::new().allocation_size(), 0);
//     assert!(
//         HashSlabMap::<u32, u32>::with_capacity(1).allocation_size() > core::mem::size_of::<u32>()
//     );
// }
