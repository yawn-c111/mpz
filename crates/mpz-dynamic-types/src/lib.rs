//! Dynamic type abstractions used in `mpz`.

#![deny(missing_docs, unreachable_pub, unused_must_use)]

use std::error::Error;

pub mod composite;
pub mod primitive;
pub mod repr;

/// A memory type.
pub trait Memory {
    /// A memory identifier.
    type Id;
    /// Type of value stored in memory.
    type Type;
}

impl<T> Memory for Vec<T> {
    type Id = usize;
    type Type = T;
}

/// A memory store.
pub trait MemoryGet: Memory {
    /// Gets a value from memory if it exists.
    fn get(&self, id: &Self::Id) -> Option<&Self::Type>;
}

impl<T> MemoryGet for Vec<T> {
    fn get(&self, id: &Self::Id) -> Option<&T> {
        self.as_slice().get(*id)
    }
}

/// A memory that can be mutated.
pub trait MemoryMut: Memory {
    /// Sets a value in memory.
    fn set(&mut self, id: &Self::Id, value: Self::Type);
}

impl<T> MemoryMut for Vec<T> {
    fn set(&mut self, id: &Self::Id, value: T) {
        self[*id] = value;
    }
}

/// A memory that can allocate values.
pub trait MemoryAlloc: Memory {
    /// Allocates a value in memory.
    fn alloc(&mut self, value: Self::Type) -> Self::Id;
}

impl<T> MemoryAlloc for Vec<T> {
    fn alloc(&mut self, value: T) -> Self::Id {
        let id = self.len();
        self.push(value);
        id
    }
}

/// A conversion error.
#[derive(Debug, thiserror::Error)]
#[error("failed to convert value: {0}")]
pub struct ConvertError(Box<dyn Error + Send + Sync>);

impl ConvertError {
    /// Creates a new conversion error.
    pub fn new<E>(err: E) -> Self
    where
        E: Into<Box<dyn Error + Send + Sync>>,
    {
        Self(err.into())
    }
}
