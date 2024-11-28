# HashSlab

[![build status](https://github.com/pepyaka/hashslab/actions/workflows/ci.yml/badge.svg)](https://github.com/pepyaka/hashslab/actions)
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

`HashSlab` is implemented using a [`HashMap`](https://docs.rs/hashbrown/latest/hashbrown/struct.HashMap.html) for key-value pairs and a [`Slab`](https://docs.rs/slab/latest/slab/struct.Slab.html) for managing pair indexes. Keys and values are stored in the `HashMap`, while each `Slab` entry also holds the *raw* hash value (`u64`) of the key, which can be used to efficiently locate the corresponding key entry in the `HashMap`.

## Capacity and reallocation

`HashSlab` stores keys and values in a `HashMap` and indexes in a `Slab`. These data structures manage their capacities independently: key storage is allocated by the `HashMap`'s allocator, while value storage is handled by the `Vec` allocator (as `Slab` is implemented on top of a `Vec`).
