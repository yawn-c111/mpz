pub(crate) mod binary;

use std::{
    fmt::Display,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use mpz_dynamic_types::{Memory, MemoryGet, MemoryMut};

/// A feed in a circuit.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Feed;

/// A sink in a circuit.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sink;

/// A node in a circuit.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Node<T> {
    pub(crate) id: usize,
    _pd: std::marker::PhantomData<T>,
}

impl Display for Node<Feed> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Feed({})", self.id)
    }
}

impl Display for Node<Sink> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sink({})", self.id)
    }
}

impl<T> Node<T> {
    #[inline(always)]
    pub(crate) fn new(id: usize) -> Self {
        Self {
            id,
            _pd: PhantomData,
        }
    }

    /// Returns the id of the node.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Shifts the node ID by the given offset.
    pub(crate) fn shift_left(&mut self, offset: usize) {
        self.id -= offset;
    }
}

impl<T> AsRef<Node<T>> for Node<T> {
    fn as_ref(&self) -> &Node<T> {
        self
    }
}

impl From<Node<Feed>> for Node<Sink> {
    fn from(node: Node<Feed>) -> Self {
        Self {
            id: node.id,
            _pd: PhantomData,
        }
    }
}

impl From<Node<Sink>> for Node<Feed> {
    fn from(node: Node<Sink>) -> Self {
        Self {
            id: node.id,
            _pd: PhantomData,
        }
    }
}

/// Registers used in a circuit evaluation.
#[derive(Debug, Default)]
pub(crate) struct Registers<T>(Vec<T>);

impl<T: Default> Registers<T> {
    /// Creates a new set of registers with the given count.
    pub(crate) fn new(count: usize) -> Self {
        Self((0..count).map(|_| T::default()).collect())
    }
}

impl<T> Memory for Registers<T> {
    type Id = Node<Feed>;
    type Type = T;
}

impl<T> MemoryGet for Registers<T> {
    fn get(&self, id: &Self::Id) -> Option<&T> {
        self.0.get(&id.id)
    }
}

impl<T> MemoryMut for Registers<T> {
    fn set(&mut self, id: &Self::Id, value: T) {
        self.0[id.id] = value;
    }
}

impl<T, U> Index<Node<T>> for Registers<U> {
    type Output = U;

    #[inline]
    fn index(&self, index: Node<T>) -> &Self::Output {
        self.0.index(index.id)
    }
}

impl<T, U> IndexMut<Node<T>> for Registers<U> {
    #[inline]
    fn index_mut(&mut self, index: Node<T>) -> &mut Self::Output {
        self.0.index_mut(index.id)
    }
}
