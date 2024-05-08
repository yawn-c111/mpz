//! Representation of values in memory.

#[cfg(feature = "binary")]
pub mod binary;

use crate::{Memory, MemoryMut};

/// A representation of a value in memory.
///
/// Data may be stored in memory in a different format than it is represented in a program. For example,
/// memory is often byte-addressable while a program may operate on higher-level data types such as 32-bit integers.
///
/// This trait provides an interface between a memory store and higher-level data types.
pub trait Repr<T, M: Memory<T>> {
    /// Type of value represented.
    type Value;

    /// Gets a value from memory if it exists.
    fn get(&self, mem: &M) -> Option<Self::Value>;

    /// Sets a value in memory.
    fn set(&self, mem: &mut M, value: Self::Value)
    where
        M: MemoryMut<T>;

    /// Allocates a value in memory.
    fn alloc(mem: &mut M, value: Self::Value) -> Self
    where
        Self: Sized;
}
