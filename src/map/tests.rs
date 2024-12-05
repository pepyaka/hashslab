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
