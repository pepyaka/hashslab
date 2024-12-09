# HashSlab

[![build status](https://github.com/pepyaka/hashslab/actions/workflows/check.yaml/badge.svg)](https://github.com/pepyaka/hashslab/actions)
[![crates.io](https://img.shields.io/crates/v/hashslab.svg)](https://crates.io/crates/hashslab)
[![docs](https://docs.rs/hashslab/badge.svg)](https://docs.rs/hashslab)

`HashSlab` is a library inspired by [`IndexMap`](https://docs.rs/indexmap), designed to provide a key-value data structure that allows efficient access by both key and index. Unlike `IndexMap`, `HashSlabMap` guarantees that the index of a key-value pair remains stable and does not change, even when entries are removed.

## Key Features

- **Stable Indexes:** Once a key-value pair is inserted, its `usize` index is preserved throughout the lifetime of the map, regardless of any removals.
- **Dual Access:** Access values either by key or by their associated index.
- **Interface:** `HashSlabMap` methods aim to closely resemble those of `IndexMap`.

## When to Use `HashSlab`

This crate is ideal for scenarios where:
- You need predictable and stable indexing of entries.
- Entries are frequently added and removed, but their indexes must remain consistent for external references.

## Examples

Basic storing and retrieval:

```rust
use hashslab::HashSlabMap;

let mut map = HashSlabMap::new();

map.insert('a', "hello");
assert_eq!(map.get(&'a'), Some(&"hello"));

let (idx, _) = map.insert_full('b', "world");
assert_eq!(idx, 1);
assert_eq!(map[&'b'], "world");

map[idx] = "earth";
assert_eq!(map.get_index(0), Some((&'a', &"hello")));
assert_eq!(map[idx], "earth");
```

HashSlab preserve value's index:

```rust
use hashslab::HashSlabMap;

let mut map = HashSlabMap::new();

map.insert('a', "hello");
map.insert('b', "world");
map.insert('c', "!");

map.remove(&'a');
map.remove_index(1);

assert_eq!(map.get_index_of(&'c'), Some(2));
```

## Implementation

`HashSlab` is implemented using a [`HashMap`](https://docs.rs/hashbrown/latest/hashbrown/struct.HashMap.html) for keys and a [`Slab`](https://docs.rs/slab/latest/slab/struct.Slab.html) for values. The `HashMap` stores the keys, while each entry in the `Slab` contains both the value and the *raw* hash (`u64`) of the corresponding key. This design allows efficient retrieval of the associated key entry in the `HashMap` using the precomputed hash.

## Performance

In general, `HashSlabMap` has comparable performance to `HashMap` and `IndexMap`. Below is a summary of its performance characteristics:

- **Creation:** Empty created `HashSlabMap` performs worse because internally it creates 2 data structures `HashMap` and `Slab`, taking twice as long as `HashMap` and `IndexMap`. With preallocation, performance is similar, as most time is spent on memory allocation.

- **Insertion:** Performance is identical across all three data structures.

- **Lookup:** Searching with `.get()` performs the same as `HashMap` and `IndexMap`. However, `.get_index()` is about 10 times slower than `IndexMap` because `IndexMap` stores entries in a `Vec`-like structure, making index lookups as fast as `.get()` in a `Vec`. In `HashSlabMap`, the hash value is first located in the `Slab`, followed by the corresponding key-value pair in the `HashMap`.

- **Removal:** Removing by key is on par with `HashMap` and faster than `IndexMap`. `IndexMap` provides two methods:
  - `.swap_remove()` - performs similarly to `HashSlabMap::remove()`.
  - `.shift_remove()` - significantly slower, as it shifts elements in the `Vec`. This method is *not* included in benchmarks due to being 50-100 times slower.

Comprehensive benchmarks, including detailed graphs and comparisons, can be found [here](https://pepyaka.github.io/hashslab/report/).
