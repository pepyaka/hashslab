use super::*;

mod hashbrown;
mod indexmap;

#[derive(Debug, Hash, PartialEq, Eq)]
struct Key<K>(K);

#[derive(Debug, PartialEq, Eq)]
struct Value<V>(V);

#[test]
fn reinsert_index() {
    let mut map = HashSlabMap::new();
    map.insert('a', Some(0));
    map.insert('b', Some(1));

    assert_eq!(Some((0, 'a', Some(0))), map.remove_full(&'a'));

    assert_eq!((0, None), map.insert_full('c', Some(2)));
}

#[test]
fn remove_all_but_one() {
    let mut map = HashSlabMap::new();

    for n in 0..100 {
        map.insert(Key(n), Value(n));
    }

    for n in 0..99 {
        map.remove(&Key(n));
    }

    assert_eq!(Some(&Value(99)), map.get_index_value(99));
}

#[test]
fn with_default_hasher() {
    let mut map = HashSlabMap::new();

    for i in 0..20 {
        map.insert(Key(i), ());
    }

    for i in (0..20).step_by(2) {
        map.remove(&Key(i));
    }

    let sample = (1..20).step_by(2).collect::<Vec<_>>();
    let mut output = map.indices().collect::<Vec<_>>();
    output.sort();
    assert_eq!(sample, output);
}

#[test]
fn with_fnv_hasher() {
    let mut map = HashSlabMap::with_hasher(fnv::FnvBuildHasher::default());

    for i in 0..20 {
        map.insert(Key(i), ());
    }

    for i in (0..20).step_by(2) {
        map.remove(&Key(i));
    }

    let sample = (1..20).step_by(2).collect::<Vec<_>>();
    let mut output = map.indices().collect::<Vec<_>>();
    output.sort();
    assert_eq!(sample, output);
}

#[test]
fn with_fxhash_hasher() {
    let mut map = HashSlabMap::with_hasher(fxhash::FxBuildHasher::default());

    for i in 0..20 {
        map.insert(Key(i), ());
    }

    for i in (0..20).step_by(2) {
        map.remove(&Key(i));
    }

    let sample = (1..20).step_by(2).collect::<Vec<_>>();
    let mut output = map.indices().collect::<Vec<_>>();
    output.sort();
    assert_eq!(sample, output);
}

#[test]
fn insert_same_key() {
    let mut map = HashSlabMap::new();

    assert_eq!(None, map.insert(0, "A"));
    assert_eq!(Some("A"), map.insert(0, "B"));
}

#[test]
fn simple_extend() {
    let mut map = HashSlabMap::new();

    assert_eq!(None, map.insert(0, "A"));

    map.extend([(0, "B")]);
    assert_eq!(1, map.len());
    assert_eq!(Some(&"B"), map.get(&0));

    map.extend([(0, "B"), (1, "C")]);
    dbg!(&map);
    assert_eq!(2, map.len());
    assert_eq!(Some(&"B"), map.get(&0));

    map.extend([(0, "B"), (1, "C"), (2, "D")]);
    assert_eq!(3, map.len());
    assert_eq!(Some(&"B"), map.get(&0));
}

#[test]
fn reserve_step_by_step() {
    let mut map = HashSlabMap::<(), ()>::new();

    assert_eq!(0, map.table.len());
    assert_eq!(0, map.table.capacity());
    assert_eq!(0, map.slab.len());
    assert_eq!(0, map.slab.capacity());

    map.reserve(1);
    let cap = 1;

    assert_eq!(0, map.table.len());
    assert!(
        dbg!(map.table.capacity()) >= cap,
        "{} >= {cap}",
        map.slab.capacity()
    );
    assert_eq!(0, map.slab.len());
    assert!(
        map.slab.capacity() >= cap,
        "{} >= {cap}",
        map.slab.capacity()
    );
}
