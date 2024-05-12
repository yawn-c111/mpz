//! Binary primitive types.

use std::fmt;

use itybity::{FromBitIterator, IntoBits, ToBits};
use rand::Rng;

use crate::{
    composite::{Array, Composite, CompositeType},
    primitive::{PrimitiveType, StaticPrimitiveType},
    ConvertError,
};

/// A binary type.
pub trait Binary:
    PrimitiveType<Type = BinaryType>
    + From<Self::Bit>
    + From<Self::U8>
    + From<Self::U16>
    + From<Self::U32>
    + From<Self::U64>
    + From<Self::U128>
    + Sized
{
    /// Backing type.
    type BackingType: Clone;

    /// Bit type.
    type Bit;
    /// U8 type.
    type U8;
    /// U16 type.
    type U16;
    /// U32 type.
    type U32;
    /// U64 type.
    type U64;
    /// U128 type.
    type U128;

    /// Converts from a bit to self.
    fn from_bit(bit: Self::BackingType) -> Self;

    /// Converts from bits in LSB0 order to u8.
    fn from_u8(bits: [Self::BackingType; 8]) -> Self;

    /// Converts from bits in LSB0 order to u16.
    fn from_u16(bits: [Self::BackingType; 16]) -> Self;

    /// Converts from bits in LSB0 order to u32.
    fn from_u32(bits: [Self::BackingType; 32]) -> Self;

    /// Converts from bits in LSB0 order to u64.
    fn from_u64(bits: [Self::BackingType; 64]) -> Self;

    /// Converts from bits in LSB0 order to u128.
    fn from_u128(bits: [Self::BackingType; 128]) -> Self;

    /// Converts self to a bit.
    fn into_bit(self) -> Self::BackingType;

    /// Converts a u8 to bits in LSB0 order.
    fn into_u8(self) -> [Self::BackingType; 8];

    /// Converts a u16 to bits in LSB0 order.
    fn into_u16(self) -> [Self::BackingType; 16];

    /// Converts a u32 to bits in LSB0 order.
    fn into_u32(self) -> [Self::BackingType; 32];

    /// Converts a u64 to bits in LSB0 order.
    fn into_u64(self) -> [Self::BackingType; 64];

    /// Converts a u128 to bits in LSB0 order.
    fn into_u128(self) -> [Self::BackingType; 128];
}

/// A type with a bit length.
pub trait BitLength {
    /// Returns the bit length.
    fn bit_length(&self) -> usize;
}

impl<P> BitLength for Array<P>
where
    P: BitLength,
{
    #[inline]
    fn bit_length(&self) -> usize {
        self.get(0)
            .map(|elem| elem.bit_length())
            .unwrap_or_default()
            * self.len()
    }
}

impl<P> BitLength for CompositeType<P>
where
    P: BitLength,
{
    #[inline]
    fn bit_length(&self) -> usize {
        match self {
            CompositeType::Primitive(ty) => ty.bit_length(),
            CompositeType::Array { ty, len } => {
                ty.as_ref().map(|ty| ty.bit_length()).unwrap_or_default() * len
            }
        }
    }
}

impl<P> BitLength for Composite<P>
where
    P: BitLength,
{
    #[inline]
    fn bit_length(&self) -> usize {
        match self {
            Composite::Primitive(primitive) => primitive.bit_length(),
            Composite::Array(array) => array.bit_length(),
        }
    }
}

/// A type with a static bit length.
pub trait StaticBitLength {
    /// The bit length.
    const BIT_LENGTH: usize;
}

/// A binary type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BinaryType {
    /// A bit.
    Bit,
    /// An unsigned 8-bit integer.
    U8,
    /// An unsigned 16-bit integer.
    U16,
    /// An unsigned 32-bit integer.
    U32,
    /// An unsigned 64-bit integer.
    U64,
    /// An unsigned 128-bit integer.
    U128,
}

impl BinaryType {
    /// Generates a random binary value.
    pub fn random<R: Rng + ?Sized>(&self, rng: &mut R) -> Value {
        match self {
            BinaryType::Bit => Value::Bit(rng.gen()),
            BinaryType::U8 => Value::U8(rng.gen()),
            BinaryType::U16 => Value::U16(rng.gen()),
            BinaryType::U32 => Value::U32(rng.gen()),
            BinaryType::U64 => Value::U64(rng.gen()),
            BinaryType::U128 => Value::U128(rng.gen()),
        }
    }
}

impl fmt::Display for BinaryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinaryType::Bit => write!(f, "Bit"),
            BinaryType::U8 => write!(f, "U8"),
            BinaryType::U16 => write!(f, "U16"),
            BinaryType::U32 => write!(f, "U32"),
            BinaryType::U64 => write!(f, "U64"),
            BinaryType::U128 => write!(f, "U128"),
        }
    }
}

impl BitLength for BinaryType {
    #[inline]
    fn bit_length(&self) -> usize {
        match self {
            BinaryType::Bit => 1,
            BinaryType::U8 => 8,
            BinaryType::U16 => 16,
            BinaryType::U32 => 32,
            BinaryType::U64 => 64,
            BinaryType::U128 => 128,
        }
    }
}

/// A binary value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Value {
    /// A bit.
    Bit(bool),
    /// An unsigned 8-bit integer.
    U8(u8),
    /// An unsigned 16-bit integer.
    U16(u16),
    /// An unsigned 32-bit integer.
    U32(u32),
    /// An unsigned 64-bit integer.
    U64(u64),
    /// An unsigned 128-bit integer.
    U128(u128),
}

impl PrimitiveType for Value {
    type Type = BinaryType;

    fn primitive_type(&self) -> BinaryType {
        match self {
            Value::Bit(_) => BinaryType::Bit,
            Value::U8(_) => BinaryType::U8,
            Value::U16(_) => BinaryType::U16,
            Value::U32(_) => BinaryType::U32,
            Value::U64(_) => BinaryType::U64,
            Value::U128(_) => BinaryType::U128,
        }
    }
}

impl Binary for Value {
    type BackingType = bool;
    type Bit = bool;
    type U8 = u8;
    type U16 = u16;
    type U32 = u32;
    type U64 = u64;
    type U128 = u128;

    fn from_bit(bit: bool) -> Self {
        Value::Bit(bit)
    }

    fn from_u8(bits: [bool; 8]) -> Self {
        Value::U8(u8::from_lsb0_iter(bits))
    }

    fn from_u16(bits: [bool; 16]) -> Self {
        Value::U16(u16::from_lsb0_iter(bits))
    }

    fn from_u32(bits: [bool; 32]) -> Self {
        Value::U32(u32::from_lsb0_iter(bits))
    }

    fn from_u64(bits: [bool; 64]) -> Self {
        Value::U64(u64::from_lsb0_iter(bits))
    }

    fn from_u128(bits: [bool; 128]) -> Self {
        Value::U128(u128::from_lsb0_iter(bits))
    }

    fn into_bit(self) -> bool {
        match self {
            Value::Bit(value) => value,
            _ => panic!("value is not a bit: {:?}", self),
        }
    }

    fn into_u8(self) -> [bool; 8] {
        let Value::U8(value) = self else {
            panic!("value is not u8: {:?}", self);
        };

        let mut bits = [false; 8];
        value
            .iter_lsb0()
            .zip(bits.iter_mut())
            .for_each(|(bit, b)| *b = bit);
        bits
    }

    fn into_u16(self) -> [bool; 16] {
        let Value::U16(value) = self else {
            panic!("value is not u16: {:?}", self);
        };

        let mut bits = [false; 16];
        value
            .iter_lsb0()
            .zip(bits.iter_mut())
            .for_each(|(bit, b)| *b = bit);
        bits
    }

    fn into_u32(self) -> [bool; 32] {
        let Value::U32(value) = self else {
            panic!("value is not u32: {:?}", self);
        };

        let mut bits = [false; 32];
        value
            .iter_lsb0()
            .zip(bits.iter_mut())
            .for_each(|(bit, b)| *b = bit);
        bits
    }

    fn into_u64(self) -> [bool; 64] {
        let Value::U64(value) = self else {
            panic!("value is not u64: {:?}", self);
        };

        let mut bits = [false; 64];
        value
            .iter_lsb0()
            .zip(bits.iter_mut())
            .for_each(|(bit, b)| *b = bit);
        bits
    }

    fn into_u128(self) -> [bool; 128] {
        let Value::U128(value) = self else {
            panic!("value is not u128: {:?}", self);
        };

        let mut bits = [false; 128];
        value
            .iter_lsb0()
            .zip(bits.iter_mut())
            .for_each(|(bit, b)| *b = bit);
        bits
    }
}

impl BitLength for Value {
    #[inline]
    fn bit_length(&self) -> usize {
        match self {
            Value::Bit(_) => 1,
            Value::U8(_) => 8,
            Value::U16(_) => 16,
            Value::U32(_) => 32,
            Value::U64(_) => 64,
            Value::U128(_) => 128,
        }
    }
}

impl IntoBits for Value {
    type IterLsb0 = std::vec::IntoIter<bool>;
    type IterMsb0 = std::vec::IntoIter<bool>;

    fn into_iter_lsb0(self) -> Self::IterLsb0 {
        match self {
            Value::Bit(v) => v.into_lsb0_vec().into_iter(),
            Value::U8(v) => v.into_lsb0_vec().into_iter(),
            Value::U16(v) => v.into_lsb0_vec().into_iter(),
            Value::U32(v) => v.into_lsb0_vec().into_iter(),
            Value::U64(v) => v.into_lsb0_vec().into_iter(),
            Value::U128(v) => v.into_lsb0_vec().into_iter(),
        }
    }

    fn into_iter_msb0(self) -> Self::IterMsb0 {
        match self {
            Value::Bit(v) => v.into_msb0_vec().into_iter(),
            Value::U8(v) => v.into_msb0_vec().into_iter(),
            Value::U16(v) => v.into_msb0_vec().into_iter(),
            Value::U32(v) => v.into_msb0_vec().into_iter(),
            Value::U64(v) => v.into_msb0_vec().into_iter(),
            Value::U128(v) => v.into_msb0_vec().into_iter(),
        }
    }
}

macro_rules! impl_primitive {
    ($(($ty:ty, $ident:ident, $len:literal)),*) => {
        impl_traits!($(($ty, $ident, $len)),*);
        impl_convert!($(($ty, $ident)),*);
    };
}

/// Implements custom traits for binary types.
macro_rules! impl_traits {
    ($(($ty:ty, $ident:ident, $len:literal)),*) => {
        $(
            impl StaticBitLength for $ty {
                const BIT_LENGTH: usize = $len;
            }

            impl PrimitiveType for $ty {
                type Type = BinaryType;

                fn primitive_type(&self) -> BinaryType {
                    BinaryType::$ident
                }
            }

            impl StaticPrimitiveType for $ty {
                const TYPE: BinaryType = BinaryType::$ident;
            }
        )*
    };
}

/// Implements conversion methods for binary types.
macro_rules! impl_convert {
    ($(($ty:ty, $ident:ident)),*) => {
        $(
            impl From<$ty> for Value {
                #[inline]
                fn from(value: $ty) -> Self {
                    Value::$ident(value)
                }
            }

            impl From<$ty> for Composite<Value> {
                #[inline]
                fn from(value: $ty) -> Self {
                    Composite::Primitive(Value::$ident(value))
                }
            }

            impl TryFrom<Value> for $ty {
                type Error = ConvertError;

                fn try_from(value: Value) -> Result<Self, Self::Error> {
                    match value {
                        Value::$ident(value) => Ok(value),
                        _ => Err(ConvertError::new(
                            format!("failed to convert {} to {}", value.primitive_type(), stringify!($ty)),
                        )),
                    }
                }
            }

            impl TryFrom<Composite<Value>> for $ty {
                type Error = ConvertError;

                fn try_from(value: Composite<Value>) -> Result<Self, Self::Error> {
                    match value {
                        Composite::Primitive(Value::$ident(value)) => Ok(value),
                        _ => Err(ConvertError::new(
                            format!("failed to convert {} to {}", value.composite_type(), stringify!($ty)),
                        )),
                    }
                }
            }

            impl<const N: usize> TryFrom<Array<Value>> for [$ty; N] {
                type Error = ConvertError;

                fn try_from(arr: Array<Value>) -> Result<Self, Self::Error> {
                    if let Some(ty) = arr.primitive_type() {
                        if ty == BinaryType::$ident && arr.len() == N {
                            let mut iter = arr.into_iter().map(|elem| <$ty>::try_from(elem).unwrap());
                            return Ok(core::array::from_fn(|_| iter.next().unwrap()));
                        }

                        return Err(ConvertError::new(
                            format!("failed to convert [{}; {}] to [{}; {}]", ty, arr.len(), stringify!($ty), N),
                        ));
                    } else if N == 0 {
                        return Ok([<$ty>::default(); N]);
                    }

                    Err(ConvertError::new(
                        format!("failed to convert [] to [{}; {}]", stringify!($ty), N),
                    ))
                }
            }

            impl<const N: usize> TryFrom<Composite<Value>> for [$ty; N] {
                type Error = ConvertError;

                fn try_from(value: Composite<Value>) -> Result<Self, Self::Error> {
                    match value {
                        Composite::Array(arr) => Self::try_from(arr),
                        _ => Err(ConvertError::new(
                            format!("failed to convert {} to [{}; {}]", value.composite_type(), stringify!($ty), N),
                        )),
                    }
                }
            }

            // impl TryFrom<Value> for $ty {
            //     type Error = ValueConvertError;

            //     fn try_from(binary: Value) -> Result<Self, Self::Error> {
            //         match binary {
            //             Value::$ident(value) => Ok(value),
            //             _ => Err(ValueConvertError()),
            //         }
            //     }
            // }

            // impl<const N: usize> TryFrom<Value> for [$ty; N] {
            //     type Error = ValueConvertError;

            //     fn try_from(value: Value) -> Result<Self, Self::Error> {
            //         match value {
            //             Value::Array(array) => {
            //                 if array.elem_type() != BinaryType::$ident {
            //                     return Err(ValueConvertError::new(
            //                         BinaryType::$ident,
            //                         array.elem_type(),
            //                     ));
            //                 }

            //                 if array.len() != N {
            //                     return Err(ValueConvertError());
            //                 }

            //                 let mut iter = array.into_iter().map(TryInto::try_into);
            //                 let mut result = [<$ty>::default(); N];

            //                 for value in result.iter_mut() {
            //                     *value = iter.next().unwrap()?;
            //                 }

            //                 Ok(result)
            //             }
            //             _ => Err(return Err(ValueConvertError())),
            //         }
            //     }
            // }
        )*
    };
}

impl_primitive!(
    (bool, Bit, 1),
    (u8, U8, 8),
    (u16, U16, 16),
    (u32, U32, 32),
    (u64, U64, 64),
    (u128, U128, 128)
);
