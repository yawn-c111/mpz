//! Composite types.

mod array;

use std::fmt::Display;

#[doc(hidden)]
pub use array::ArrayIterMut;
pub use array::{Array, ArrayIter, InconsistentType};

use crate::{
    primitive::{PrimitiveType, StaticPrimitiveType},
    repr::Repr,
    MemoryAlloc, MemoryGet, MemoryMut,
};

/// A static composite type.
pub trait StaticCompositeType<P> {
    /// The composite type.
    const TYPE: CompositeType<P>;
}

impl<P: StaticPrimitiveType> StaticCompositeType<P::Type> for P {
    const TYPE: CompositeType<P::Type> = CompositeType::Primitive(P::TYPE);
}

impl<const N: usize, P: StaticPrimitiveType> StaticCompositeType<P::Type> for [P; N] {
    const TYPE: CompositeType<P::Type> = CompositeType::Array {
        ty: Some(P::TYPE),
        len: N,
    };
}

/// Type information of a composite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompositeType<P> {
    /// A primitive.
    Primitive(P),
    /// An array.
    Array {
        /// Element type.
        ty: Option<P>,
        /// Length of the array.
        len: usize,
    },
}

impl<P> From<P> for CompositeType<P> {
    #[inline]
    fn from(p: P) -> Self {
        CompositeType::Primitive(p)
    }
}

impl<P: Display> Display for CompositeType<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompositeType::Primitive(p) => write!(f, "{}", p),
            CompositeType::Array { ty, len } => {
                if let Some(ty) = ty {
                    write!(f, "[{}; {}]", ty, len)
                } else {
                    write!(f, "[]")
                }
            }
        }
    }
}

/// A composite.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(deserialize = "P: PrimitiveType + for<'a> serde::de::Deserialize<'a>"))
)]
pub enum Composite<P> {
    /// A primitive.
    Primitive(P),
    /// An array.
    Array(Array<P>),
}

impl<P> Composite<P> {
    /// Returns `true` if the composite is a primitive.
    #[inline]
    pub fn is_primitive(&self) -> bool {
        matches!(self, Composite::Primitive(_))
    }

    /// Returns `true` if the composite is an array.
    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Composite::Array(_))
    }

    /// Returns an iterator over the primitives.
    pub fn iter(&self) -> CompositeIter<P> {
        match self {
            Composite::Primitive(p) => CompositeIter {
                inner: CompositeIterInner::Primitive(core::iter::once(p)),
            },
            Composite::Array(arr) => CompositeIter {
                inner: CompositeIterInner::Array(arr.iter()),
            },
        }
    }

    /// Returns a mutable iterator over the primitives.
    ///
    /// # Internal
    ///
    /// This method is intended for internal use only.
    #[doc(hidden)]
    pub fn iter_mut(&mut self) -> CompositeIterMut<P> {
        match self {
            Composite::Primitive(p) => CompositeIterMut {
                inner: CompositeIterInnerMut::Primitive(core::iter::once(p)),
            },
            Composite::Array(arr) => CompositeIterMut {
                inner: CompositeIterInnerMut::Array(arr.iter_mut()),
            },
        }
    }
}

impl<P: PrimitiveType> Composite<P> {
    /// Returns the composite type.
    pub fn composite_type(&self) -> CompositeType<P::Type> {
        match self {
            Composite::Primitive(p) => CompositeType::Primitive(p.primitive_type()),
            Composite::Array(arr) => CompositeType::Array {
                ty: arr.primitive_type(),
                len: arr.len(),
            },
        }
    }
}

impl<T, P> From<T> for Composite<P>
where
    T: Into<Array<P>>,
{
    fn from(arr: T) -> Self {
        Composite::Array(arr.into())
    }
}

impl<V, R, M> Repr<Composite<V>, M> for Composite<R>
where
    V: PrimitiveType,
    R: PrimitiveType<Type = V::Type> + Repr<V, M>,
    Array<R>: Repr<Array<V>, M>,
{
    fn get(&self, mem: &M) -> Option<Composite<V>>
    where
        M: MemoryGet,
    {
        match self {
            Composite::Primitive(repr) => repr.get(mem).map(Composite::Primitive),
            Composite::Array(repr) => Repr::get(repr, mem).map(Composite::Array),
        }
    }

    fn set(&self, mem: &mut M, value: Composite<V>)
    where
        M: MemoryMut,
    {
        match (self, value) {
            (Composite::Primitive(repr), Composite::Primitive(value)) => repr.set(mem, value),
            (Composite::Array(repr), Composite::Array(value)) => repr.set(mem, value),
            _ => panic!("mismatched types"),
        }
    }

    fn alloc(mem: &mut M, value: Composite<V>) -> Self
    where
        Self: Sized,
        M: MemoryAlloc,
    {
        match value {
            Composite::Primitive(value) => Composite::Primitive(R::alloc(mem, value)),
            Composite::Array(value) => Composite::Array(Array::alloc(mem, value)),
        }
    }
}

/// An iterator for [`Composite`].
#[derive(Debug)]
pub struct CompositeIter<'a, P> {
    inner: CompositeIterInner<'a, P>,
}

impl<'a, P> Iterator for CompositeIter<'a, P> {
    type Item = &'a P;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            CompositeIterInner::Primitive(iter) => iter.next(),
            CompositeIterInner::Array(iter) => iter.next(),
        }
    }
}

#[derive(Debug)]
enum CompositeIterInner<'a, P> {
    Primitive(core::iter::Once<&'a P>),
    Array(ArrayIter<'a, P>),
}

/// A mutating iterator for [`Composite`].
#[derive(Debug)]
#[doc(hidden)]
pub struct CompositeIterMut<'a, P> {
    inner: CompositeIterInnerMut<'a, P>,
}

impl<'a, P> Iterator for CompositeIterMut<'a, P> {
    type Item = &'a mut P;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            CompositeIterInnerMut::Primitive(iter) => iter.next(),
            CompositeIterInnerMut::Array(iter) => iter.next(),
        }
    }
}

#[derive(Debug)]
enum CompositeIterInnerMut<'a, P> {
    Primitive(core::iter::Once<&'a mut P>),
    Array(ArrayIterMut<'a, P>),
}
