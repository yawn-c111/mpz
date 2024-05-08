//! Representations for values stored in bit-addressable memory.

use itybity::{FromBitIterator, IntoBits};
use mpz_binary_types::{BitLength, StaticBitLength, ValueType};

use crate::{repr::Repr, Memory, MemoryMut};

pub use mpz_binary_types::{Array, Primitive, PrimitiveType, Value};

/// A value representation.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "Id: serde::Serialize",
        deserialize = "Id: for<'a> serde::de::Deserialize<'a>"
    ))
)]
pub enum ValueRepr<Id> {
    /// A primitive value.
    Primitive(PrimitiveRepr<Id>),
    /// An array value.
    Array(ArrayRepr<Id>),
}

impl<Id> ValueRepr<Id> {
    pub fn try_from_ids(ty: ValueType, ids: impl IntoIterator<Item = Id>) -> Option<Self> {
        match ty {
            ValueType::Primitive(ty) => {
                Some(ValueRepr::Primitive(PrimitiveRepr::try_from_ids(ty, ids)?))
            }
            ValueType::Array { ty, len } => {
                Some(ValueRepr::Array(ArrayRepr::try_from_ids(ty, len, ids)?))
            }
        }
    }

    /// Returns the value type.
    pub fn value_type(&self) -> ValueType {
        match self {
            ValueRepr::Primitive(repr) => ValueType::Primitive(repr.primitive_type()),
            ValueRepr::Array(repr) => ValueType::Array {
                ty: repr.elem_type(),
                len: repr.len(),
            },
        }
    }

    /// Returns `true` if the representation is a primitive value.
    pub fn is_primitive(&self) -> bool {
        matches!(self, ValueRepr::Primitive(_))
    }

    /// Returns `true` if the representation is an array value.
    pub fn is_array(&self) -> bool {
        matches!(self, ValueRepr::Array(_))
    }

    /// Returns an iterator over the memory ids.
    pub fn iter(&self) -> ValueIter<'_, Id> {
        match self {
            ValueRepr::Primitive(repr) => ValueIter {
                inner: ValueIterInner::Primitive(repr.iter()),
            },
            ValueRepr::Array(repr) => ValueIter {
                inner: ValueIterInner::Array(repr.iter()),
            },
        }
    }

    /// Returns a mutating iterator over the memory ids.
    ///
    /// **This is intended for internal use only.**
    #[doc(hidden)]
    pub fn iter_mut(&mut self) -> ValueIterMut<'_, Id> {
        match self {
            ValueRepr::Primitive(repr) => ValueIterMut {
                inner: ValueIterInnerMut::Primitive(repr.iter_mut()),
            },
            ValueRepr::Array(repr) => ValueIterMut {
                inner: ValueIterInnerMut::Array(repr.iter_mut()),
            },
        }
    }
}

impl<const N: usize, Id, T> From<[T; N]> for ValueRepr<Id>
where
    T: Into<PrimitiveRepr<Id>>,
{
    fn from(value: [T; N]) -> Self {
        Self::Array(value.into())
    }
}

impl<Id, T> From<Vec<T>> for ValueRepr<Id>
where
    T: Into<PrimitiveRepr<Id>>,
{
    fn from(value: Vec<T>) -> Self {
        Self::Array(value.into())
    }
}

impl<Id> BitLength for ValueRepr<Id> {
    fn bit_length(&self) -> usize {
        match self {
            ValueRepr::Primitive(repr) => repr.bit_length(),
            ValueRepr::Array(repr) => repr.bit_length(),
        }
    }
}

impl<Id, M> Repr<bool, M> for ValueRepr<Id>
where
    M: Memory<bool, Id = Id>,
{
    type Value = Value;

    #[inline]
    fn get(&self, mem: &M) -> Option<Self::Value> {
        match self {
            ValueRepr::Primitive(repr) => repr.get(mem).map(Value::Primitive),
            ValueRepr::Array(repr) => repr.get(mem).map(Value::Array),
        }
    }

    #[inline]
    fn set(&self, mem: &mut M, value: Self::Value)
    where
        M: MemoryMut<bool, Id = Id>,
    {
        match (self, value) {
            (ValueRepr::Primitive(repr), Value::Primitive(value)) => repr.set(mem, value),
            (ValueRepr::Array(repr), Value::Array(value)) => repr.set(mem, value),
            _ => panic!("value type mismatch"),
        }
    }

    #[inline]
    fn alloc(mem: &mut M, value: Self::Value) -> Self
    where
        Self: Sized,
    {
        match value {
            Value::Primitive(primitive) => {
                ValueRepr::Primitive(PrimitiveRepr::alloc(mem, primitive))
            }
            Value::Array(array) => ValueRepr::Array(ArrayRepr::alloc(mem, array)),
        }
    }
}

/// An iterator for [`ValueRepr`].
#[derive(Debug)]
pub struct ValueIter<'a, Id> {
    inner: ValueIterInner<'a, Id>,
}

impl<'a, Id> Iterator for ValueIter<'a, Id> {
    type Item = &'a Id;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            ValueIterInner::Primitive(iter) => iter.next(),
            ValueIterInner::Array(iter) => iter.next(),
        }
    }
}

#[derive(Debug)]
enum ValueIterInner<'a, Id> {
    Primitive(PrimitiveIter<'a, Id>),
    Array(ArrayIter<'a, Id>),
}

/// A mutating iterator for [`ValueRepr`].
#[derive(Debug)]
#[doc(hidden)]
pub struct ValueIterMut<'a, Id> {
    inner: ValueIterInnerMut<'a, Id>,
}

impl<'a, Id> Iterator for ValueIterMut<'a, Id> {
    type Item = &'a mut Id;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            ValueIterInnerMut::Primitive(iter) => iter.next(),
            ValueIterInnerMut::Array(iter) => iter.next(),
        }
    }
}

#[derive(Debug)]
enum ValueIterInnerMut<'a, Id> {
    Primitive(PrimitiveIterMut<'a, Id>),
    Array(ArrayIterMut<'a, Id>),
}

/// A primitive representation.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "Id: serde::Serialize",
        deserialize = "Id: for<'a> serde::de::Deserialize<'a>"
    ))
)]
pub enum PrimitiveRepr<Id> {
    /// A bit.
    Bit(Bit<Id>),
    /// An unsigned 8-bit integer.
    U8(U8<Id>),
    /// An unsigned 16-bit integer.
    U16(U16<Id>),
    /// An unsigned 32-bit integer.
    U32(U32<Id>),
    /// An unsigned 64-bit integer.
    U64(U64<Id>),
    /// An unsigned 128-bit integer.
    U128(U128<Id>),
}

impl<Id> From<PrimitiveRepr<Id>> for ValueRepr<Id> {
    fn from(v: PrimitiveRepr<Id>) -> Self {
        ValueRepr::Primitive(v)
    }
}

impl<Id> PrimitiveRepr<Id> {
    /// Tries to create a representation from memory ids, returning `None` the number of ids
    /// does not match the expected length.
    pub fn try_from_ids(ty: PrimitiveType, ids: impl IntoIterator<Item = Id>) -> Option<Self> {
        let mut iter = ids.into_iter();
        let mut ids: Vec<_> = iter.by_ref().take(ty.bit_length()).collect();

        if iter.next().is_some() {
            return None;
        }

        Some(match ty {
            PrimitiveType::Bit => PrimitiveRepr::Bit(Bit::new(ids.pop()?)),
            PrimitiveType::U8 => PrimitiveRepr::U8(U8::new(ids.try_into().ok()?)),
            PrimitiveType::U16 => PrimitiveRepr::U16(U16::new(ids.try_into().ok()?)),
            PrimitiveType::U32 => PrimitiveRepr::U32(U32::new(ids.try_into().ok()?)),
            PrimitiveType::U64 => PrimitiveRepr::U64(U64::new(ids.try_into().ok()?)),
            PrimitiveType::U128 => PrimitiveRepr::U128(U128::new(ids.try_into().ok()?)),
        })
    }

    /// Returns the primitive type.
    pub fn primitive_type(&self) -> PrimitiveType {
        match self {
            PrimitiveRepr::Bit(repr) => repr.primitive_type(),
            PrimitiveRepr::U8(repr) => repr.primitive_type(),
            PrimitiveRepr::U16(repr) => repr.primitive_type(),
            PrimitiveRepr::U32(repr) => repr.primitive_type(),
            PrimitiveRepr::U64(repr) => repr.primitive_type(),
            PrimitiveRepr::U128(repr) => repr.primitive_type(),
        }
    }

    /// Returns an iterator over the memory ids.
    pub fn iter(&self) -> PrimitiveIter<'_, Id> {
        match self {
            PrimitiveRepr::Bit(repr) => PrimitiveIter {
                inner: PrimitiveIterInner::Bit(core::iter::once(repr.id())),
            },
            PrimitiveRepr::U8(repr) => PrimitiveIter {
                inner: PrimitiveIterInner::U8(repr.ids().iter()),
            },
            PrimitiveRepr::U16(repr) => PrimitiveIter {
                inner: PrimitiveIterInner::U16(repr.ids().iter()),
            },
            PrimitiveRepr::U32(repr) => PrimitiveIter {
                inner: PrimitiveIterInner::U32(repr.ids().iter()),
            },
            PrimitiveRepr::U64(repr) => PrimitiveIter {
                inner: PrimitiveIterInner::U64(repr.ids().iter()),
            },
            PrimitiveRepr::U128(repr) => PrimitiveIter {
                inner: PrimitiveIterInner::U128(repr.ids().iter()),
            },
        }
    }

    /// Returns a mutating iterator over the memory ids.
    ///
    /// **This is intended for internal use only.**
    #[doc(hidden)]
    pub fn iter_mut(&mut self) -> PrimitiveIterMut<'_, Id> {
        match self {
            PrimitiveRepr::Bit(repr) => PrimitiveIterMut {
                inner: PrimitiveIterInnerMut::Bit(core::iter::once(repr.id_mut())),
            },
            PrimitiveRepr::U8(repr) => PrimitiveIterMut {
                inner: PrimitiveIterInnerMut::U8(repr.ids_mut().iter_mut()),
            },
            PrimitiveRepr::U16(repr) => PrimitiveIterMut {
                inner: PrimitiveIterInnerMut::U16(repr.ids_mut().iter_mut()),
            },
            PrimitiveRepr::U32(repr) => PrimitiveIterMut {
                inner: PrimitiveIterInnerMut::U32(repr.ids_mut().iter_mut()),
            },
            PrimitiveRepr::U64(repr) => PrimitiveIterMut {
                inner: PrimitiveIterInnerMut::U64(repr.ids_mut().iter_mut()),
            },
            PrimitiveRepr::U128(repr) => PrimitiveIterMut {
                inner: PrimitiveIterInnerMut::U128(repr.ids_mut().iter_mut()),
            },
        }
    }
}

impl<Id> BitLength for PrimitiveRepr<Id> {
    fn bit_length(&self) -> usize {
        match self {
            PrimitiveRepr::Bit(_) => 1,
            PrimitiveRepr::U8(repr) => repr.bit_length(),
            PrimitiveRepr::U16(repr) => repr.bit_length(),
            PrimitiveRepr::U32(repr) => repr.bit_length(),
            PrimitiveRepr::U64(repr) => repr.bit_length(),
            PrimitiveRepr::U128(repr) => repr.bit_length(),
        }
    }
}

impl<Id, M> Repr<bool, M> for PrimitiveRepr<Id>
where
    M: Memory<bool, Id = Id>,
{
    type Value = Primitive;

    #[inline]
    fn get(&self, mem: &M) -> Option<Self::Value> {
        match self {
            PrimitiveRepr::Bit(repr) => repr.get(mem).map(Primitive::Bit),
            PrimitiveRepr::U8(repr) => repr.get(mem).map(Primitive::U8),
            PrimitiveRepr::U16(repr) => repr.get(mem).map(Primitive::U16),
            PrimitiveRepr::U32(repr) => repr.get(mem).map(Primitive::U32),
            PrimitiveRepr::U64(repr) => repr.get(mem).map(Primitive::U64),
            PrimitiveRepr::U128(repr) => repr.get(mem).map(Primitive::U128),
        }
    }

    #[inline]
    fn set(&self, mem: &mut M, value: Self::Value)
    where
        M: MemoryMut<bool>,
    {
        match (self, value) {
            (PrimitiveRepr::Bit(repr), Primitive::Bit(value)) => repr.set(mem, value),
            (PrimitiveRepr::U8(repr), Primitive::U8(value)) => repr.set(mem, value),
            (PrimitiveRepr::U16(repr), Primitive::U16(value)) => repr.set(mem, value),
            (PrimitiveRepr::U32(repr), Primitive::U32(value)) => repr.set(mem, value),
            (PrimitiveRepr::U64(repr), Primitive::U64(value)) => repr.set(mem, value),
            (PrimitiveRepr::U128(repr), Primitive::U128(value)) => repr.set(mem, value),
            _ => panic!("value type mismatch"),
        }
    }

    #[inline]
    fn alloc(mem: &mut M, value: Self::Value) -> Self
    where
        Self: Sized,
    {
        match value {
            Primitive::Bit(bit) => PrimitiveRepr::Bit(Bit::alloc(mem, bit)),
            Primitive::U8(u8) => PrimitiveRepr::U8(U8::alloc(mem, u8)),
            Primitive::U16(u16) => PrimitiveRepr::U16(U16::alloc(mem, u16)),
            Primitive::U32(u32) => PrimitiveRepr::U32(U32::alloc(mem, u32)),
            Primitive::U64(u64) => PrimitiveRepr::U64(U64::alloc(mem, u64)),
            Primitive::U128(u128) => PrimitiveRepr::U128(U128::alloc(mem, u128)),
        }
    }
}

/// An iterator for [`PrimitiveRepr`].
#[derive(Debug)]
pub struct PrimitiveIter<'a, Id> {
    inner: PrimitiveIterInner<'a, Id>,
}

impl<'a, Id> Iterator for PrimitiveIter<'a, Id> {
    type Item = &'a Id;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            PrimitiveIterInner::Bit(iter) => iter.next(),
            PrimitiveIterInner::U8(iter) => iter.next(),
            PrimitiveIterInner::U16(iter) => iter.next(),
            PrimitiveIterInner::U32(iter) => iter.next(),
            PrimitiveIterInner::U64(iter) => iter.next(),
            PrimitiveIterInner::U128(iter) => iter.next(),
        }
    }
}

#[derive(Debug)]
enum PrimitiveIterInner<'a, Id> {
    Bit(core::iter::Once<&'a Id>),
    U8(core::slice::Iter<'a, Id>),
    U16(core::slice::Iter<'a, Id>),
    U32(core::slice::Iter<'a, Id>),
    U64(core::slice::Iter<'a, Id>),
    U128(core::slice::Iter<'a, Id>),
}

/// A mutating iterator for [`PrimitiveRepr`].
#[derive(Debug)]
#[doc(hidden)]
pub struct PrimitiveIterMut<'a, Id> {
    inner: PrimitiveIterInnerMut<'a, Id>,
}

impl<'a, Id> Iterator for PrimitiveIterMut<'a, Id> {
    type Item = &'a mut Id;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            PrimitiveIterInnerMut::Bit(iter) => iter.next(),
            PrimitiveIterInnerMut::U8(iter) => iter.next(),
            PrimitiveIterInnerMut::U16(iter) => iter.next(),
            PrimitiveIterInnerMut::U32(iter) => iter.next(),
            PrimitiveIterInnerMut::U64(iter) => iter.next(),
            PrimitiveIterInnerMut::U128(iter) => iter.next(),
        }
    }
}

#[derive(Debug)]
enum PrimitiveIterInnerMut<'a, Id> {
    Bit(core::iter::Once<&'a mut Id>),
    U8(core::slice::IterMut<'a, Id>),
    U16(core::slice::IterMut<'a, Id>),
    U32(core::slice::IterMut<'a, Id>),
    U64(core::slice::IterMut<'a, Id>),
    U128(core::slice::IterMut<'a, Id>),
}

/// A type with a static primitive representation.
pub trait StaticPrimitive: StaticBitLength {
    /// Representation type.
    type Repr<Id: Clone>: StaticPrimitiveRepr<Id>;
}

pub trait StaticPrimitiveRepr<Id>: Clone + Into<ValueRepr<Id>> {
    /// Tries to create a representation from memory ids, returning `None` if the number of ids
    /// does not match the expected length.
    fn try_from_ids(ids: Vec<Id>) -> Option<Self>;
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(missing_docs)]
pub struct Bit<Id>(Id);

impl<Id> Bit<Id> {
    /// Creates a new bit representation.
    #[inline]
    pub fn new(id: Id) -> Self {
        Bit(id)
    }

    /// Returns the primitive type.
    #[inline]
    pub fn primitive_type(&self) -> PrimitiveType {
        PrimitiveType::Bit
    }

    /// Returns the underlying memory id.
    pub fn into_inner(self) -> Id {
        self.0
    }

    /// Returns a reference to the underlying memory id.
    pub fn id(&self) -> &Id {
        &self.0
    }

    /// Returns a mutable reference to the underlying memory id.
    #[doc(hidden)]
    pub fn id_mut(&mut self) -> &mut Id {
        &mut self.0
    }

    #[inline]
    fn get(&self, mem: &impl Memory<bool, Id = Id>) -> Option<bool> {
        mem.get(&self.0).copied()
    }

    #[inline]
    fn set(&self, mem: &mut impl MemoryMut<bool, Id = Id>, value: bool) {
        mem.set(&self.0, value);
    }

    #[inline]
    fn alloc(mem: &mut impl Memory<bool, Id = Id>, value: bool) -> Self {
        let id = mem.alloc(value);
        Bit(id)
    }
}

impl<Id> BitLength for Bit<Id> {
    fn bit_length(&self) -> usize {
        1
    }
}

impl<Id> From<Bit<Id>> for ValueRepr<Id> {
    fn from(v: Bit<Id>) -> Self {
        ValueRepr::Primitive(PrimitiveRepr::Bit(v))
    }
}

impl StaticPrimitive for bool {
    type Repr<Id: Clone> = Bit<Id>;
}

impl<Id: Clone> StaticPrimitiveRepr<Id> for Bit<Id> {
    fn try_from_ids(mut ids: Vec<Id>) -> Option<Self> {
        if ids.len() != 1 {
            return None;
        }

        Some(Bit(ids.pop().unwrap()))
    }
}

macro_rules! define_primitive {
    ($ty:ty, $id:ident, $len:expr) => {
        #[derive(Debug, Clone, Copy)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[allow(missing_docs)]
        pub struct $id<T>(
            #[cfg_attr(
                feature = "serde",
                serde(
                    bound(
                        serialize = "T: serde::Serialize",
                        deserialize = "T: for<'a> serde::de::Deserialize<'a>"
                    ),
                    with = "serde_arrays"
                )
            )]
            [T; $len],
        );

        impl<T> $id<T> {
            /// Creates a new primitive representation.
            #[inline]
            pub fn new(ids: [T; $len]) -> Self {
                $id(ids)
            }

            /// Returns the bit length.
            #[allow(clippy::len_without_is_empty)]
            pub const fn len(&self) -> usize {
                $len
            }

            /// Returns the primitive type.
            #[inline]
            pub fn primitive_type(&self) -> PrimitiveType {
                PrimitiveType::$id
            }

            /// Returns the underlying memory ids.
            pub fn into_inner(self) -> [T; $len] {
                self.0
            }

            /// Returns a reference to the underlying memory ids.
            pub fn ids(&self) -> &[T; $len] {
                &self.0
            }

            /// Returns a mutable reference to the underlying memory ids.
            #[doc(hidden)]
            pub fn ids_mut(&mut self) -> &mut [T; $len] {
                &mut self.0
            }

            #[inline]
            fn get(&self, mem: &impl Memory<bool, Id = T>) -> Option<$ty> {
                let mut value = [false; $len];
                for (i, id) in self.0.iter().enumerate() {
                    value[i] = *mem.get(id)?;
                }
                Some(<$ty>::from_lsb0_iter(value.into_iter()))
            }

            #[inline]
            fn set(&self, mem: &mut impl MemoryMut<bool, Id = T>, value: $ty) {
                for (id, bit) in self.0.iter().zip(value.into_iter_lsb0()) {
                    mem.set(id, bit);
                }
            }

            #[inline]
            fn alloc(mem: &mut impl Memory<bool, Id = T>, value: $ty) -> Self {
                let mut bits = value.into_iter_lsb0();
                $id(core::array::from_fn(|_| mem.alloc(bits.next().unwrap())))
            }
        }

        impl<T> AsRef<[T]> for $id<T> {
            fn as_ref(&self) -> &[T] {
                &self.0
            }
        }

        impl<T> AsMut<[T]> for $id<T> {
            fn as_mut(&mut self) -> &mut [T] {
                &mut self.0
            }
        }

        impl<T> std::ops::Index<usize> for $id<T> {
            type Output = T;

            fn index(&self, index: usize) -> &Self::Output {
                &self.0[index]
            }
        }

        impl<T> std::ops::Index<std::ops::Range<usize>> for $id<T> {
            type Output = [T];

            fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
                &self.0[index]
            }
        }

        impl<T> From<$id<T>> for PrimitiveRepr<T> {
            fn from(v: $id<T>) -> Self {
                PrimitiveRepr::$id(v)
            }
        }

        impl<T> From<$id<T>> for ValueRepr<T> {
            fn from(v: $id<T>) -> Self {
                PrimitiveRepr::$id(v).into()
            }
        }

        impl<T> BitLength for $id<T> {
            fn bit_length(&self) -> usize {
                $len
            }
        }

        impl StaticPrimitive for $ty {
            type Repr<Id: Clone> = $id<Id>;
        }

        impl<Id: Clone> StaticPrimitiveRepr<Id> for $id<Id> {
            fn try_from_ids(ids: Vec<Id>) -> Option<Self> {
                if ids.len() != $len {
                    return None;
                }

                let mut iter = ids.into_iter();
                Some($id(core::array::from_fn(|_| iter.next().unwrap())))
            }
        }

        impl<const N: usize, T> TryFrom<ArrayRepr<T>> for [$id<T>; N] {
            type Error = ArrayConvertError;

            fn try_from(value: ArrayRepr<T>) -> Result<Self, Self::Error> {
                if value.elems.len() != N {
                    return Err(ArrayConvertError());
                }

                if value.ty != PrimitiveType::$id {
                    return Err(ArrayConvertError());
                }

                let mut iter = value.elems.into_iter();
                Ok(core::array::from_fn(|_| {
                    let PrimitiveRepr::$id(repr) = iter.next().unwrap() else {
                        panic!("unexpected primitive type")
                    };

                    repr
                }))
            }
        }
    };
}

define_primitive!(u8, U8, 8);
define_primitive!(u16, U16, 16);
define_primitive!(u32, U32, 32);
define_primitive!(u64, U64, 64);
define_primitive!(u128, U128, 128);

macro_rules! impl_to_bytes {
    ($ident:ident, $len:literal) => {
        impl<T: Copy> $ident<T> {
            /// Converts to a big-endian byte array representation.
            pub fn to_be_bytes(&self) -> [U8<T>; $len] {
                core::array::from_fn(|i| {
                    U8::new(core::array::from_fn(|j| self.0[($len - i - 1) * 8 + j]))
                })
            }

            /// Converts from a big-endian byte array representation.
            pub fn from_be_bytes(bytes: [U8<T>; $len]) -> Self {
                $ident::new(core::array::from_fn(|i| bytes[$len - (i / 8) - 1].0[i % 8]))
            }

            /// Converts to a little-endian byte array representation.
            pub fn to_le_bytes(&self) -> [U8<T>; $len] {
                core::array::from_fn(|i| U8::new(core::array::from_fn(|j| self.0[i * 8 + j])))
            }

            /// Converts from a little-endian byte array representation.
            pub fn from_le_bytes(bytes: [U8<T>; $len]) -> Self {
                $ident::new(core::array::from_fn(|i| bytes[i / 8].0[i % 8]))
            }
        }
    };
}

impl_to_bytes!(U8, 1);
impl_to_bytes!(U16, 2);
impl_to_bytes!(U32, 4);
impl_to_bytes!(U64, 8);
impl_to_bytes!(U128, 16);

#[derive(Debug, thiserror::Error)]
#[error("failed to convert array repr to another representation")]
pub struct ArrayConvertError();

/// An array representation.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "Id: serde::Serialize",
        deserialize = "Id: for<'a> serde::de::Deserialize<'a>"
    ))
)]
pub struct ArrayRepr<Id> {
    ty: PrimitiveType,
    elems: Vec<PrimitiveRepr<Id>>,
}

impl<Id> ArrayRepr<Id> {
    /// Tries to create a representation from memory ids, returning `None` if the number of ids
    /// does not match the expected length.
    pub fn try_from_ids(
        ty: PrimitiveType,
        len: usize,
        ids: impl IntoIterator<Item = Id>,
    ) -> Option<Self> {
        let mut iter = ids.into_iter();
        let mut elems = Vec::with_capacity(len);

        for _ in 0..len {
            let elem = PrimitiveRepr::try_from_ids(ty, iter.by_ref().take(ty.bit_length()))?;
            elems.push(elem);
        }

        if iter.next().is_some() {
            return None;
        }

        Some(ArrayRepr { ty, elems })
    }

    /// Returns the element type.
    #[inline]
    pub fn elem_type(&self) -> PrimitiveType {
        self.ty
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.elems.len()
    }

    /// Returns `true` if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }

    /// Returns an iterator over the elements.
    pub fn iter_elems(&self) -> impl Iterator<Item = &PrimitiveRepr<Id>> {
        self.elems.iter()
    }

    /// Returns an iterator over the memory ids.
    pub fn iter(&self) -> ArrayIter<'_, Id> {
        ArrayIter(self.elems.iter().flat_map(|elem| elem.iter()))
    }

    /// Returns a mutating iterator over the memory ids.
    #[doc(hidden)]
    pub fn iter_mut(&mut self) -> ArrayIterMut<'_, Id> {
        ArrayIterMut(self.elems.iter_mut().flat_map(|elem| elem.iter_mut()))
    }

    /// Returns the elements as a slice.
    pub fn as_slice(&self) -> &[PrimitiveRepr<Id>] {
        &self.elems
    }

    /// Reverses the order of the elements.
    pub fn reverse(&mut self) {
        self.elems.reverse();
    }
}

impl<Id> From<ArrayRepr<Id>> for ValueRepr<Id> {
    fn from(value: ArrayRepr<Id>) -> Self {
        ValueRepr::Array(value)
    }
}

impl<Id> BitLength for ArrayRepr<Id> {
    fn bit_length(&self) -> usize {
        self.ty.bit_length() * self.elems.len()
    }
}

impl<const N: usize, Id, T> From<[T; N]> for ArrayRepr<Id>
where
    T: Into<PrimitiveRepr<Id>>,
{
    fn from(value: [T; N]) -> Self {
        let elems: Vec<_> = value.into_iter().map(|elem| elem.into()).collect();
        ArrayRepr {
            ty: elems[0].primitive_type(),
            elems,
        }
    }
}

impl<Id, T> From<Vec<T>> for ArrayRepr<Id>
where
    T: Into<PrimitiveRepr<Id>>,
{
    fn from(value: Vec<T>) -> Self {
        let elems: Vec<_> = value.into_iter().map(|elem| elem.into()).collect();
        ArrayRepr {
            ty: elems[0].primitive_type(),
            elems,
        }
    }
}

impl<Id, M> Repr<bool, M> for ArrayRepr<Id>
where
    M: Memory<bool, Id = Id>,
{
    type Value = Array;

    #[inline]
    fn get(&self, mem: &M) -> Option<Self::Value> {
        let mut elems = Vec::with_capacity(self.elems.len());
        for elem in &self.elems {
            elems.push(elem.get(mem)?);
        }
        Some(Array::new_with_type(self.ty, elems).expect("elements are the same type"))
    }

    #[inline]
    fn set(&self, mem: &mut M, value: Self::Value)
    where
        M: MemoryMut<bool>,
    {
        for (repr, value) in self.elems.iter().zip(value.into_iter()) {
            repr.set(mem, value)
        }
    }

    #[inline]
    fn alloc(mem: &mut M, value: Self::Value) -> Self
    where
        Self: Sized,
    {
        let ty = value.elem_type();
        let elems = value
            .into_iter()
            .map(|elem| PrimitiveRepr::alloc(mem, elem))
            .collect();
        ArrayRepr { ty, elems }
    }
}

/// An iterator for [`ArrayRepr`].
#[derive(Debug)]
pub struct ArrayIter<'a, Id>(
    core::iter::FlatMap<
        core::slice::Iter<'a, PrimitiveRepr<Id>>,
        PrimitiveIter<'a, Id>,
        fn(&'a PrimitiveRepr<Id>) -> PrimitiveIter<'a, Id>,
    >,
);

impl<'a, Id> Iterator for ArrayIter<'a, Id> {
    type Item = &'a Id;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// A mutating iterator for [`ArrayRepr`].
#[derive(Debug)]
#[doc(hidden)]
pub struct ArrayIterMut<'a, Id>(
    core::iter::FlatMap<
        core::slice::IterMut<'a, PrimitiveRepr<Id>>,
        PrimitiveIterMut<'a, Id>,
        fn(&'a mut PrimitiveRepr<Id>) -> PrimitiveIterMut<'a, Id>,
    >,
);

impl<'a, Id> Iterator for ArrayIterMut<'a, Id> {
    type Item = &'a mut Id;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_repr() {
        let mut mem = Vec::new();
        let repr = ValueRepr::alloc(&mut mem, Value::from(42u8));
        assert_eq!(repr.get(&mem), Some(Value::from(42u8)));

        let repr2 = ValueRepr::alloc(&mut mem, Value::from(0u8));
        assert_eq!(repr2.get(&mem), Some(Value::from(0u8)));

        let repr3 = ValueRepr::alloc(&mut mem, Value::from([42u8, 69u8]));
        assert_eq!(repr3.get(&mem), Some(Value::from([42u8, 69u8])));

        let repr4 = ValueRepr::alloc(&mut mem, Value::from([0u8, 0u8]));
        assert_eq!(repr4.get(&mem), Some(Value::from([0u8, 0u8])));
    }

    #[test]
    fn test_binary_primitive() {
        let mut mem = Vec::new();
        let repr = PrimitiveRepr::alloc(&mut mem, Primitive::from(42u8));
        assert_eq!(repr.get(&mem), Some(Primitive::from(42u8)));

        let repr2 = PrimitiveRepr::alloc(&mut mem, Primitive::from(0u8));
        assert_eq!(repr2.get(&mem), Some(Primitive::from(0u8)));
    }

    #[test]
    fn test_binary_array() {
        let mut mem = Vec::new();
        let repr = ArrayRepr::alloc(&mut mem, Array::from([42u8, 69u8]));
        assert_eq!(repr.get(&mem), Some(Array::from([42u8, 69u8])));

        // empty array
        let elems: [u8; 0] = [];
        let repr2 = ArrayRepr::alloc(&mut mem, Array::from(elems));
        assert_eq!(repr2.get(&mem), Some(Array::from(elems)));
    }

    #[test]
    fn test_binary_bit() {
        let mut mem = Vec::new();
        let repr = Bit::alloc(&mut mem, true);
        assert_eq!(repr.get(&mem), Some(true));

        let repr2 = Bit::alloc(&mut mem, false);
        assert_eq!(repr2.get(&mem), Some(false));
    }

    macro_rules! test_uint {
        ($name:ident, $ty:ty, $ident:ident) => {
            #[test]
            fn $name() {
                let mut mem = Vec::new();
                let repr = $ident::alloc(&mut mem, 42);
                assert_eq!(repr.get(&mem), Some(42));

                let repr2 = $ident::alloc(&mut mem, 0);
                assert_eq!(repr2.get(&mem), Some(0));
            }
        };
    }

    test_uint!(test_binary_u8, u8, U8);
    test_uint!(test_binary_u16, u16, U16);
    test_uint!(test_binary_u32, u32, U32);
    test_uint!(test_binary_u64, u64, U64);
    test_uint!(test_binary_u128, u128, U128);
}
