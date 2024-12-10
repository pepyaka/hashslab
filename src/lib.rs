#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::alloc::Layout;

use thiserror::Error;

pub mod map;
pub use map::HashSlabMap;

#[derive(Debug, Clone)]
struct ValueData<V> {
    value: V,
    hash: u64,
}

impl<V> ValueData<V> {
    fn new(value: V, hash: u64) -> Self {
        Self { value, hash }
    }
}

#[derive(Debug, Clone)]
struct KeyData<K> {
    key: K,
    index: usize,
}

impl<K> KeyData<K> {
    fn new(key: K, index: usize) -> Self {
        Self { key, index }
    }
}

/// The error type for [`try_reserve`][HashSlabMap::try_reserve] methods.
#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum TryReserveError {
    #[error("Error due to the computed capacity exceeding the collection's maximum (usually `isize::MAX` bytes)")]
    CapacityOverflow,

    #[error("The memory allocator returned an error. The layout of allocation request that failed: {layout:?}")]
    AllocError { layout: Layout },

    #[error(
        "sum of current ({capacity}) and additional ({additional}) capacity exceeds isize::MAX"
    )]
    Slab { capacity: usize, additional: usize },
}

impl From<hashbrown::TryReserveError> for TryReserveError {
    fn from(err: hashbrown::TryReserveError) -> Self {
        match err {
            hashbrown::TryReserveError::CapacityOverflow => Self::CapacityOverflow,
            hashbrown::TryReserveError::AllocError { layout } => Self::AllocError { layout },
        }
    }
}
