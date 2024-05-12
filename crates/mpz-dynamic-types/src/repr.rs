//! Type representations in memory.

pub mod binary;

use crate::{
    primitive::{PrimitiveType, StaticPrimitiveType},
    ConvertError, MemoryAlloc, MemoryGet, MemoryMut, MemoryReserve,
};

/// A primitive representation.
pub trait PrimitiveRepr<Id>: PrimitiveType + Sized {
    /// Attempts to convert ids to a representation of the given type.
    fn try_from_ids(ty: Self::Type, ids: Vec<Id>) -> Result<Self, ConvertError>;
}

/// A type with a static primitive representation.
pub trait StaticPrimitiveRepr: StaticPrimitiveType {
    /// Primitive representation type.
    type Repr<Id>: PrimitiveRepr<Id, Type = Self::Type>;
}

/// A representation of a value in memory.
///
/// Data may be stored in memory in a different format than it is represented in a program. For example,
/// memory is often byte-addressable while a program may operate on higher-level data types such as 32-bit integers.
///
/// This trait provides an interface between a memory store and higher-level data types.
pub trait Repr<V, M> {
    /// Type information for `V`.
    type Type;

    /// Gets a value from memory if it exists.
    fn get(&self, mem: &M) -> Option<V>
    where
        M: MemoryGet;

    /// Sets a value in memory.
    fn set(&self, mem: &mut M, value: V)
    where
        M: MemoryMut;

    /// Allocates a value in memory.
    fn alloc(mem: &mut M, value: V) -> Self
    where
        Self: Sized,
        M: MemoryAlloc;

    /// Defines a value in memory with the given type.
    fn reserve(mem: &mut M, ty: Self::Type) -> Self
    where
        Self: Sized,
        M: MemoryReserve;
}
