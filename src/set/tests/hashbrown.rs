use std::format;
use std::vec::Vec;

use fnv::FnvBuildHasher;

use crate::set::*;

#[test]
fn test_zero_capacities() {
    type HS = HashSlabSet<i32>;

    let s = HS::new();
    assert_eq!(s.capacity(), 0);

    let s = HS::default();
    assert_eq!(s.capacity(), 0);

    let s = HashSlabSet::<i32, _>::with_hasher(FnvBuildHasher::default());
    assert_eq!(s.capacity(), 0);

    let s = HS::with_capacity(0);
    assert_eq!(s.capacity(), 0);

    let s = HashSlabSet::<i32, _>::with_capacity_and_hasher(0, FnvBuildHasher::default());
    assert_eq!(s.capacity(), 0);

    let mut s = HS::new();
    s.insert(1);
    s.insert(2);
    s.remove(&1);
    s.remove(&2);
    s.shrink_to_fit();
    assert_eq!(s.capacity(), 0);

    let mut s = HS::new();
    s.reserve(0);
    assert_eq!(s.capacity(), 0);
}

#[test]
fn test_disjoint() {
    let mut xs = HashSlabSet::new();
    let mut ys = HashSlabSet::new();
    assert!(xs.is_disjoint(&ys));
    assert!(ys.is_disjoint(&xs));
    assert!(xs.insert(5));
    assert!(ys.insert(11));
    assert!(xs.is_disjoint(&ys));
    assert!(ys.is_disjoint(&xs));
    assert!(xs.insert(7));
    assert!(xs.insert(19));
    assert!(xs.insert(4));
    assert!(ys.insert(2));
    assert!(ys.insert(-11));
    assert!(xs.is_disjoint(&ys));
    assert!(ys.is_disjoint(&xs));
    assert!(ys.insert(7));
    assert!(!xs.is_disjoint(&ys));
    assert!(!ys.is_disjoint(&xs));
}

#[test]
fn test_subset_and_superset() {
    let mut a = HashSlabSet::new();
    assert!(a.insert(0));
    assert!(a.insert(5));
    assert!(a.insert(11));
    assert!(a.insert(7));

    let mut b = HashSlabSet::new();
    assert!(b.insert(0));
    assert!(b.insert(7));
    assert!(b.insert(19));
    assert!(b.insert(250));
    assert!(b.insert(11));
    assert!(b.insert(200));

    assert!(!a.is_subset(&b));
    assert!(!a.is_superset(&b));
    assert!(!b.is_subset(&a));
    assert!(!b.is_superset(&a));

    assert!(b.insert(5));

    assert!(a.is_subset(&b));
    assert!(!a.is_superset(&b));
    assert!(!b.is_subset(&a));
    assert!(b.is_superset(&a));
}

#[test]
fn test_iterate() {
    let mut a = HashSlabSet::new();
    for i in 0..32 {
        assert!(a.insert(i));
    }
    let mut observed: u32 = 0;
    for k in &a {
        observed |= 1 << *k;
    }
    assert_eq!(observed, 0xFFFF_FFFF);
}

#[test]
fn test_intersection() {
    let mut a = HashSlabSet::new();
    let mut b = HashSlabSet::new();

    assert!(a.insert(11));
    assert!(a.insert(1));
    assert!(a.insert(3));
    assert!(a.insert(77));
    assert!(a.insert(103));
    assert!(a.insert(5));
    assert!(a.insert(-5));

    assert!(b.insert(2));
    assert!(b.insert(11));
    assert!(b.insert(77));
    assert!(b.insert(-9));
    assert!(b.insert(-42));
    assert!(b.insert(5));
    assert!(b.insert(3));

    let mut i = 0;
    let expected = [3, 5, 11, 77];
    for x in a.intersection(&b) {
        assert!(expected.contains(x));
        i += 1;
    }
    assert_eq!(i, expected.len());
}

#[test]
fn test_difference() {
    let mut a = HashSlabSet::new();
    let mut b = HashSlabSet::new();

    assert!(a.insert(1));
    assert!(a.insert(3));
    assert!(a.insert(5));
    assert!(a.insert(9));
    assert!(a.insert(11));

    assert!(b.insert(3));
    assert!(b.insert(9));

    let mut i = 0;
    let expected = [1, 5, 11];
    for x in a.difference(&b) {
        assert!(expected.contains(x));
        i += 1;
    }
    assert_eq!(i, expected.len());
}

#[test]
fn test_symmetric_difference() {
    let mut a = HashSlabSet::new();
    let mut b = HashSlabSet::new();

    assert!(a.insert(1));
    assert!(a.insert(3));
    assert!(a.insert(5));
    assert!(a.insert(9));
    assert!(a.insert(11));

    assert!(b.insert(-2));
    assert!(b.insert(3));
    assert!(b.insert(9));
    assert!(b.insert(14));
    assert!(b.insert(22));

    let mut i = 0;
    let expected = [-2, 1, 5, 11, 14, 22];
    for x in a.symmetric_difference(&b) {
        assert!(expected.contains(x));
        i += 1;
    }
    assert_eq!(i, expected.len());
}

#[test]
fn test_union() {
    let mut a = HashSlabSet::new();
    let mut b = HashSlabSet::new();

    assert!(a.insert(1));
    assert!(a.insert(3));
    assert!(a.insert(5));
    assert!(a.insert(9));
    assert!(a.insert(11));
    assert!(a.insert(16));
    assert!(a.insert(19));
    assert!(a.insert(24));

    assert!(b.insert(-2));
    assert!(b.insert(1));
    assert!(b.insert(5));
    assert!(b.insert(9));
    assert!(b.insert(13));
    assert!(b.insert(19));

    let mut i = 0;
    let expected = [-2, 1, 3, 5, 9, 11, 13, 16, 19, 24];
    for x in a.union(&b) {
        assert!(expected.contains(x));
        i += 1;
    }
    assert_eq!(i, expected.len());
}

#[test]
fn test_from_map() {
    let mut a = HashSlabMap::new();
    a.insert(1, ());
    a.insert(2, ());
    a.insert(3, ());
    a.insert(4, ());

    let a: HashSlabSet<_> = a.into();

    assert_eq!(a.len(), 4);
    assert!(a.contains(&1));
    assert!(a.contains(&2));
    assert!(a.contains(&3));
    assert!(a.contains(&4));
}

#[test]
fn test_from_iter() {
    let xs = [1, 2, 2, 3, 4, 5, 6, 7, 8, 9];

    let set: HashSlabSet<_> = xs.iter().copied().collect();

    for x in &xs {
        assert!(set.contains(x));
    }

    assert_eq!(set.iter().len(), xs.len() - 1);
}

#[test]
fn test_move_iter() {
    let hs = {
        let mut hs = HashSlabSet::new();

        hs.insert('a');
        hs.insert('b');

        hs
    };

    let v = hs.into_iter().collect::<Vec<char>>();
    assert!(v == ['a', 'b'] || v == ['b', 'a']);
}

#[test]
fn test_eq() {
    // These constants once happened to expose a bug in insert().
    // I'm keeping them around to prevent a regression.
    let mut s1 = HashSlabSet::new();

    s1.insert(1);
    s1.insert(2);
    s1.insert(3);

    let mut s2 = HashSlabSet::new();

    s2.insert(1);
    s2.insert(2);

    assert!(s1 != s2);

    s2.insert(3);

    assert_eq!(s1, s2);
}

#[test]
fn test_show() {
    let mut set = HashSlabSet::new();
    let empty = HashSlabSet::<i32>::new();

    set.insert(1);
    set.insert(2);

    let set_str = format!("{set:?}");

    assert!(set_str == "{1, 2}" || set_str == "{2, 1}");
    assert_eq!(format!("{empty:?}"), "{}");
}

#[test]
fn test_trivial_drain() {
    let mut s = HashSlabSet::<i32>::new();
    for _ in s.drain() {}
    assert!(s.is_empty());
    drop(s);

    let mut s = HashSlabSet::<i32>::new();
    drop(s.drain());
    assert!(s.is_empty());
}

#[test]
fn test_drain() {
    let mut s: HashSlabSet<_> = (1..100).collect();

    // try this a bunch of times to make sure we don't screw up internal state.
    for _ in 0..20 {
        assert_eq!(s.len(), 99);

        {
            let mut last_i = 0;
            let mut d = s.drain();
            for (i, x) in d.by_ref().take(50).enumerate() {
                last_i = i;
                assert!(x != 0);
            }
            assert_eq!(last_i, 49);
        }

        if !s.is_empty() {
            panic!("s should be empty!");
        }

        // reset to try again.
        s.extend(1..100);
    }
}

#[test]
fn test_replace() {
    use core::hash;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct Foo(&'static str, i32);

    impl PartialEq for Foo {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl Eq for Foo {}

    impl hash::Hash for Foo {
        fn hash<H: hash::Hasher>(&self, h: &mut H) {
            self.0.hash(h);
        }
    }

    let mut s = HashSlabSet::new();
    assert_eq!(s.replace(Foo("a", 1)), None);
    assert_eq!(s.len(), 1);
    assert_eq!(s.replace(Foo("a", 2)), Some(Foo("a", 1)));
    assert_eq!(s.len(), 1);

    let mut it = s.iter();
    assert_eq!(it.next(), Some(&Foo("a", 2)));
    assert_eq!(it.next(), None);
}

#[test]
#[allow(clippy::needless_borrow)]
fn test_extend_ref() {
    let mut a = HashSlabSet::new();
    a.insert(1);

    a.extend([2, 3, 4]);

    assert_eq!(a.len(), 4);
    assert!(a.contains(&1));
    assert!(a.contains(&2));
    assert!(a.contains(&3));
    assert!(a.contains(&4));

    let mut b = HashSlabSet::new();
    b.insert(5);
    b.insert(6);

    a.extend(&b);

    assert_eq!(a.len(), 6);
    assert!(a.contains(&1));
    assert!(a.contains(&2));
    assert!(a.contains(&3));
    assert!(a.contains(&4));
    assert!(a.contains(&5));
    assert!(a.contains(&6));
}

#[test]
fn test_retain() {
    let xs = [1, 2, 3, 4, 5, 6];
    let mut set: HashSlabSet<i32> = xs.iter().copied().collect();
    set.retain(|&k| k % 2 == 0);
    assert_eq!(set.len(), 3);
    assert!(set.contains(&2));
    assert!(set.contains(&4));
    assert!(set.contains(&6));
}

// #[test]
// fn test_extract_if() {
//     {
//         let mut set: HashSlabSet<i32> = (0..8).collect();
//         let drained = set.extract_if(|&k| k % 2 == 0);
//         let mut out = drained.collect::<Vec<_>>();
//         out.sort_unstable();
//         assert_eq!(vec![0, 2, 4, 6], out);
//         assert_eq!(set.len(), 4);
//     }
//     {
//         let mut set: HashSlabSet<i32> = (0..8).collect();
//         set.extract_if(|&k| k % 2 == 0).for_each(drop);
//         assert_eq!(set.len(), 4, "Removes non-matching items on drop");
//     }
// }

#[test]
fn test_const_with_hasher() {
    use core::hash::BuildHasher;
    use std::collections::hash_map::DefaultHasher;

    #[derive(Clone)]
    struct MyHasher;
    impl BuildHasher for MyHasher {
        type Hasher = DefaultHasher;

        fn build_hasher(&self) -> DefaultHasher {
            DefaultHasher::new()
        }
    }

    const EMPTY_SET: HashSlabSet<u32, MyHasher> = HashSlabSet::with_hasher(MyHasher);

    let mut set = EMPTY_SET;
    set.insert(19);
    assert!(set.contains(&19));
}

#[test]
fn rehash_in_place() {
    let mut set = HashSlabSet::new();

    for i in 0..224 {
        set.insert(i);
    }

    assert_eq!(
        set.capacity(),
        224,
        "The set must be at or close to capacity to trigger a re hashing"
    );

    for i in 100..1400 {
        set.remove(&(i - 100));
        set.insert(i);
    }
}

#[test]
fn collect() {
    // At the time of writing, this hits the ZST case in from_base_index
    // (and without the `map`, it does not).
    let mut _set: HashSlabSet<_> = (0..3).map(|_| ()).collect();
}

// #[test]
// fn duplicate_insert() {
//     let mut set = HashSlabSet::new();
//     set.insert(1);
//     set.get_or_insert_with(&1, |_| 1);
//     set.get_or_insert_with(&1, |_| 1);
//     assert!([1].iter().eq(set.iter()));
// }

// #[test]
// #[should_panic]
// fn some_invalid_equivalent() {
//     use core::hash::{Hash, Hasher};
//     struct Invalid {
//         count: u32,
//         other: u32,
//     }

//     struct InvalidRef {
//         count: u32,
//         other: u32,
//     }

//     impl PartialEq for Invalid {
//         fn eq(&self, other: &Self) -> bool {
//             self.count == other.count && self.other == other.other
//         }
//     }
//     impl Eq for Invalid {}

//     impl Equivalent<Invalid> for InvalidRef {
//         fn equivalent(&self, key: &Invalid) -> bool {
//             self.count == key.count && self.other == key.other
//         }
//     }
//     impl Hash for Invalid {
//         fn hash<H: Hasher>(&self, state: &mut H) {
//             self.count.hash(state);
//         }
//     }
//     impl Hash for InvalidRef {
//         fn hash<H: Hasher>(&self, state: &mut H) {
//             self.count.hash(state);
//         }
//     }
//     let mut set: HashSlabSet<Invalid> = HashSlabSet::new();
//     let key = InvalidRef { count: 1, other: 1 };
//     let value = Invalid { count: 1, other: 2 };
//     if set.hasher().hash_one(&key) == set.hasher().hash_one(&value) {
//         set.get_or_insert_with(&key, |_| value);
//     }
// }
