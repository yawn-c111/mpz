//! Binary representation types.

use crate::{
    composite::Composite,
    primitive::{
        binary::{Binary, BinaryType, BitLength},
        PrimitiveType, StaticPrimitiveType,
    },
    repr::{PrimitiveRepr, Repr, StaticPrimitiveRepr},
    ConvertError, Memory, MemoryAlloc, MemoryGet, MemoryMut,
};

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
pub enum BinaryRepr<Id> {
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

impl<Id> BinaryRepr<Id> {
    /// Tries to create a representation from memory ids, returning `None` the number of ids
    /// does not match the expected length.
    pub fn try_from_ids(ty: BinaryType, ids: impl IntoIterator<Item = Id>) -> Option<Self> {
        let mut iter = ids.into_iter();
        let mut ids: Vec<_> = iter.by_ref().take(ty.bit_length()).collect();

        if iter.next().is_some() {
            return None;
        }

        Some(match ty {
            BinaryType::Bit => BinaryRepr::Bit(Bit::new(ids.pop()?)),
            BinaryType::U8 => BinaryRepr::U8(U8::new(ids.try_into().ok()?)),
            BinaryType::U16 => BinaryRepr::U16(U16::new(ids.try_into().ok()?)),
            BinaryType::U32 => BinaryRepr::U32(U32::new(ids.try_into().ok()?)),
            BinaryType::U64 => BinaryRepr::U64(U64::new(ids.try_into().ok()?)),
            BinaryType::U128 => BinaryRepr::U128(U128::new(ids.try_into().ok()?)),
        })
    }

    /// Returns an iterator over the memory ids.
    pub fn iter(&self) -> PrimitiveIter<'_, Id> {
        match self {
            BinaryRepr::Bit(repr) => PrimitiveIter {
                inner: PrimitiveIterInner::Bit(core::iter::once(repr.id())),
            },
            BinaryRepr::U8(repr) => PrimitiveIter {
                inner: PrimitiveIterInner::U8(repr.ids().iter()),
            },
            BinaryRepr::U16(repr) => PrimitiveIter {
                inner: PrimitiveIterInner::U16(repr.ids().iter()),
            },
            BinaryRepr::U32(repr) => PrimitiveIter {
                inner: PrimitiveIterInner::U32(repr.ids().iter()),
            },
            BinaryRepr::U64(repr) => PrimitiveIter {
                inner: PrimitiveIterInner::U64(repr.ids().iter()),
            },
            BinaryRepr::U128(repr) => PrimitiveIter {
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
            BinaryRepr::Bit(repr) => PrimitiveIterMut {
                inner: PrimitiveIterInnerMut::Bit(core::iter::once(repr.id_mut())),
            },
            BinaryRepr::U8(repr) => PrimitiveIterMut {
                inner: PrimitiveIterInnerMut::U8(repr.ids_mut().iter_mut()),
            },
            BinaryRepr::U16(repr) => PrimitiveIterMut {
                inner: PrimitiveIterInnerMut::U16(repr.ids_mut().iter_mut()),
            },
            BinaryRepr::U32(repr) => PrimitiveIterMut {
                inner: PrimitiveIterInnerMut::U32(repr.ids_mut().iter_mut()),
            },
            BinaryRepr::U64(repr) => PrimitiveIterMut {
                inner: PrimitiveIterInnerMut::U64(repr.ids_mut().iter_mut()),
            },
            BinaryRepr::U128(repr) => PrimitiveIterMut {
                inner: PrimitiveIterInnerMut::U128(repr.ids_mut().iter_mut()),
            },
        }
    }
}

impl<Id> PrimitiveType for BinaryRepr<Id> {
    type Type = BinaryType;

    fn primitive_type(&self) -> Self::Type {
        match self {
            BinaryRepr::Bit(_) => BinaryType::Bit,
            BinaryRepr::U8(_) => BinaryType::U8,
            BinaryRepr::U16(_) => BinaryType::U16,
            BinaryRepr::U32(_) => BinaryType::U32,
            BinaryRepr::U64(_) => BinaryType::U64,
            BinaryRepr::U128(_) => BinaryType::U128,
        }
    }
}

impl<Id> BitLength for BinaryRepr<Id> {
    fn bit_length(&self) -> usize {
        match self {
            BinaryRepr::Bit(_) => 1,
            BinaryRepr::U8(repr) => repr.bit_length(),
            BinaryRepr::U16(repr) => repr.bit_length(),
            BinaryRepr::U32(repr) => repr.bit_length(),
            BinaryRepr::U64(repr) => repr.bit_length(),
            BinaryRepr::U128(repr) => repr.bit_length(),
        }
    }
}

impl<Id, V, M> Repr<V, M> for BinaryRepr<Id>
where
    M: Memory<Id = Id, Type = V::Bit>,
    V: Binary,
{
    fn get(&self, mem: &M) -> Option<V>
    where
        M: MemoryGet,
    {
        match self {
            BinaryRepr::Bit(repr) => mem.get(repr.id()).cloned().map(V::from),
            BinaryRepr::U8(repr) => {
                let mut bits: [V::Bit; 8] = core::array::from_fn(|_| V::Bit::default());
                for (i, id) in repr.ids().iter().enumerate() {
                    bits[i] = mem.get(id)?.clone();
                }
                Some(V::from_u8(bits))
            }
            BinaryRepr::U16(repr) => {
                let mut bits: [V::Bit; 16] = core::array::from_fn(|_| V::Bit::default());
                for (i, id) in repr.ids().iter().enumerate() {
                    bits[i] = mem.get(id)?.clone();
                }
                Some(V::from_u16(bits))
            }
            BinaryRepr::U32(repr) => {
                let mut bits: [V::Bit; 32] = core::array::from_fn(|_| V::Bit::default());
                for (i, id) in repr.ids().iter().enumerate() {
                    bits[i] = mem.get(id)?.clone();
                }
                Some(V::from_u32(bits))
            }
            BinaryRepr::U64(repr) => {
                let mut bits: [V::Bit; 64] = core::array::from_fn(|_| V::Bit::default());
                for (i, id) in repr.ids().iter().enumerate() {
                    bits[i] = mem.get(id)?.clone();
                }
                Some(V::from_u64(bits))
            }
            BinaryRepr::U128(repr) => {
                let mut bits: [V::Bit; 128] = core::array::from_fn(|_| V::Bit::default());
                for (i, id) in repr.ids().iter().enumerate() {
                    bits[i] = mem.get(id)?.clone();
                }
                Some(V::from_u128(bits))
            }
        }
    }

    fn set(&self, mem: &mut M, value: V)
    where
        M: MemoryMut,
    {
        match self {
            BinaryRepr::Bit(bit) => mem.set(bit.id(), value.into_bit()),
            BinaryRepr::U8(repr) => {
                repr.0.iter().zip(value.into_u8()).for_each(|(id, bit)| {
                    mem.set(id, bit);
                });
            }
            BinaryRepr::U16(repr) => {
                repr.0.iter().zip(value.into_u16()).for_each(|(id, bit)| {
                    mem.set(id, bit);
                });
            }
            BinaryRepr::U32(repr) => {
                repr.0.iter().zip(value.into_u32()).for_each(|(id, bit)| {
                    mem.set(id, bit);
                });
            }
            BinaryRepr::U64(repr) => {
                repr.0.iter().zip(value.into_u64()).for_each(|(id, bit)| {
                    mem.set(id, bit);
                });
            }
            BinaryRepr::U128(repr) => {
                repr.0.iter().zip(value.into_u128()).for_each(|(id, bit)| {
                    mem.set(id, bit);
                });
            }
        }
    }

    fn alloc(mem: &mut M, value: V) -> Self
    where
        Self: Sized,
        M: MemoryAlloc,
    {
        match value.primitive_type() {
            BinaryType::Bit => BinaryRepr::Bit(Bit::new(mem.alloc(value.into_bit()))),
            BinaryType::U8 => BinaryRepr::U8(U8::new(value.into_u8().map(|bit| mem.alloc(bit)))),
            BinaryType::U16 => {
                BinaryRepr::U16(U16::new(value.into_u16().map(|bit| mem.alloc(bit))))
            }
            BinaryType::U32 => {
                BinaryRepr::U32(U32::new(value.into_u32().map(|bit| mem.alloc(bit))))
            }
            BinaryType::U64 => {
                BinaryRepr::U64(U64::new(value.into_u64().map(|bit| mem.alloc(bit))))
            }
            BinaryType::U128 => {
                BinaryRepr::U128(U128::new(value.into_u128().map(|bit| mem.alloc(bit))))
            }
        }
    }
}

// impl<Id, M> Repr<bool, M> for PrimitiveRepr<Id>
// where
//     M: Memory<bool, Id = Id>,
// {
//     type Value = Primitive;

//     #[inline]
//     fn get(&self, mem: &M) -> Option<Self::Value> {
//         match self {
//             PrimitiveRepr::Bit(repr) => repr.get(mem).map(Primitive::Bit),
//             PrimitiveRepr::U8(repr) => repr.get(mem).map(Primitive::U8),
//             PrimitiveRepr::U16(repr) => repr.get(mem).map(Primitive::U16),
//             PrimitiveRepr::U32(repr) => repr.get(mem).map(Primitive::U32),
//             PrimitiveRepr::U64(repr) => repr.get(mem).map(Primitive::U64),
//             PrimitiveRepr::U128(repr) => repr.get(mem).map(Primitive::U128),
//         }
//     }

//     #[inline]
//     fn set(&self, mem: &mut M, value: Self::Value)
//     where
//         M: MemoryMut<bool>,
//     {
//         match (self, value) {
//             (PrimitiveRepr::Bit(repr), Primitive::Bit(value)) => repr.set(mem, value),
//             (PrimitiveRepr::U8(repr), Primitive::U8(value)) => repr.set(mem, value),
//             (PrimitiveRepr::U16(repr), Primitive::U16(value)) => repr.set(mem, value),
//             (PrimitiveRepr::U32(repr), Primitive::U32(value)) => repr.set(mem, value),
//             (PrimitiveRepr::U64(repr), Primitive::U64(value)) => repr.set(mem, value),
//             (PrimitiveRepr::U128(repr), Primitive::U128(value)) => repr.set(mem, value),
//             _ => panic!("value type mismatch"),
//         }
//     }

//     #[inline]
//     fn alloc(mem: &mut M, value: Self::Value) -> Self
//     where
//         Self: Sized,
//     {
//         match value {
//             Primitive::Bit(bit) => PrimitiveRepr::Bit(Bit::alloc(mem, bit)),
//             Primitive::U8(u8) => PrimitiveRepr::U8(U8::alloc(mem, u8)),
//             Primitive::U16(u16) => PrimitiveRepr::U16(U16::alloc(mem, u16)),
//             Primitive::U32(u32) => PrimitiveRepr::U32(U32::alloc(mem, u32)),
//             Primitive::U64(u64) => PrimitiveRepr::U64(U64::alloc(mem, u64)),
//             Primitive::U128(u128) => PrimitiveRepr::U128(U128::alloc(mem, u128)),
//         }
//     }
// }

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
    pub fn primitive_type(&self) -> BinaryType {
        BinaryType::Bit
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
}

impl<Id> BitLength for Bit<Id> {
    fn bit_length(&self) -> usize {
        1
    }
}

impl<Id> From<Bit<Id>> for BinaryRepr<Id> {
    fn from(v: Bit<Id>) -> Self {
        BinaryRepr::Bit(v)
    }
}

impl<Id> From<Bit<Id>> for Composite<BinaryRepr<Id>> {
    #[inline]
    fn from(v: Bit<Id>) -> Self {
        Composite::Primitive(BinaryRepr::Bit(v))
    }
}

impl<Id> TryFrom<BinaryRepr<Id>> for Bit<Id> {
    type Error = ConvertError;

    fn try_from(value: BinaryRepr<Id>) -> Result<Self, Self::Error> {
        match value {
            BinaryRepr::Bit(repr) => Ok(repr),
            _ => Err(ConvertError::new(format!(
                "failed to convert {} to Bit",
                value.primitive_type()
            ))),
        }
    }
}

impl<Id> StaticPrimitiveType for Bit<Id> {
    const TYPE: BinaryType = BinaryType::Bit;
}

impl<Id> PrimitiveType for Bit<Id> {
    type Type = BinaryType;

    fn primitive_type(&self) -> Self::Type {
        BinaryType::Bit
    }
}

impl<Id> PrimitiveRepr<Id> for Bit<Id> {
    fn try_from_ids(ty: Self::Type, mut ids: Vec<Id>) -> Result<Self, ConvertError> {
        if ty != BinaryType::Bit {
            return Err(ConvertError::new(format!(
                "expected type {}, got {}",
                BinaryType::Bit,
                ty
            )));
        }

        if ids.len() != 1 {
            return Err(ConvertError::new(format!(
                "repr expected 1 id, got {}",
                ids.len()
            )));
        }

        Ok(Bit(ids.pop().unwrap()))
    }
}

impl StaticPrimitiveRepr for bool {
    type Repr<Id> = Bit<Id>;
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
            pub fn primitive_type(&self) -> BinaryType {
                BinaryType::$id
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

        impl<T> From<$id<T>> for BinaryRepr<T> {
            fn from(v: $id<T>) -> Self {
                BinaryRepr::$id(v)
            }
        }

        impl<T> From<$id<T>> for Composite<BinaryRepr<T>> {
            #[inline]
            fn from(v: $id<T>) -> Self {
                Composite::Primitive(BinaryRepr::$id(v))
            }
        }

        impl<Id> TryFrom<BinaryRepr<Id>> for $id<Id> {
            type Error = ConvertError;

            fn try_from(value: BinaryRepr<Id>) -> Result<Self, Self::Error> {
                match value {
                    BinaryRepr::$id(repr) => Ok(repr),
                    _ => Err(ConvertError::new(format!(
                        "failed to convert {} to {}",
                        value.primitive_type(),
                        stringify!($id)
                    ))),
                }
            }
        }

        impl<T> BitLength for $id<T> {
            fn bit_length(&self) -> usize {
                $len
            }
        }

        impl<Id> StaticPrimitiveType for $id<Id> {
            const TYPE: BinaryType = BinaryType::$id;
        }

        impl<Id> PrimitiveType for $id<Id> {
            type Type = BinaryType;

            fn primitive_type(&self) -> Self::Type {
                BinaryType::$id
            }
        }

        impl<Id> PrimitiveRepr<Id> for $id<Id> {
            fn try_from_ids(ty: Self::Type, ids: Vec<Id>) -> Result<Self, ConvertError> {
                if ty != BinaryType::$id {
                    return Err(ConvertError::new(format!(
                        "expected type {}, got {}",
                        BinaryType::$id,
                        ty
                    )));
                }

                if ids.len() != $len {
                    return Err(ConvertError::new(format!(
                        "repr expected {} ids, got {}",
                        $len,
                        ids.len()
                    )));
                }

                let mut ids = ids.into_iter();

                Ok($id(core::array::from_fn(|_| ids.next().unwrap())))
            }
        }

        impl StaticPrimitiveRepr for $ty {
            type Repr<Id> = $id<Id>;
        }

        // impl<Id: Clone> StaticPrimitiveRepr<Id> for $id<Id> {
        //     fn try_from_ids(ids: Vec<Id>) -> Option<Self> {
        //         if ids.len() != $len {
        //             return None;
        //         }

        //         let mut iter = ids.into_iter();
        //         Some($id(core::array::from_fn(|_| iter.next().unwrap())))
        //     }
        // }
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
