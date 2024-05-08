use std::fmt;

use itybity::IntoBits;
use rand::Rng;

use crate::{BitLength, StaticBitLength, StaticPrimitiveType, Value, ValueConvertError, ValueType};

/// A primitive type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PrimitiveType {
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

impl PrimitiveType {
    /// Generates a random primitive value.
    pub fn random<R: Rng + ?Sized>(&self, rng: &mut R) -> Primitive {
        match self {
            PrimitiveType::Bit => Primitive::Bit(rng.gen()),
            PrimitiveType::U8 => Primitive::U8(rng.gen()),
            PrimitiveType::U16 => Primitive::U16(rng.gen()),
            PrimitiveType::U32 => Primitive::U32(rng.gen()),
            PrimitiveType::U64 => Primitive::U64(rng.gen()),
            PrimitiveType::U128 => Primitive::U128(rng.gen()),
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PrimitiveType::Bit => write!(f, "Bit"),
            PrimitiveType::U8 => write!(f, "U8"),
            PrimitiveType::U16 => write!(f, "U16"),
            PrimitiveType::U32 => write!(f, "U32"),
            PrimitiveType::U64 => write!(f, "U64"),
            PrimitiveType::U128 => write!(f, "U128"),
        }
    }
}

impl From<PrimitiveType> for ValueType {
    fn from(primitive_type: PrimitiveType) -> Self {
        ValueType::Primitive(primitive_type)
    }
}

impl BitLength for PrimitiveType {
    #[inline]
    fn bit_length(&self) -> usize {
        match self {
            PrimitiveType::Bit => 1,
            PrimitiveType::U8 => 8,
            PrimitiveType::U16 => 16,
            PrimitiveType::U32 => 32,
            PrimitiveType::U64 => 64,
            PrimitiveType::U128 => 128,
        }
    }
}

/// A primitive value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Primitive {
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

impl Primitive {
    /// Returns the primitive type.
    pub fn primitive_type(&self) -> PrimitiveType {
        match self {
            Primitive::Bit(_) => PrimitiveType::Bit,
            Primitive::U8(_) => PrimitiveType::U8,
            Primitive::U16(_) => PrimitiveType::U16,
            Primitive::U32(_) => PrimitiveType::U32,
            Primitive::U64(_) => PrimitiveType::U64,
            Primitive::U128(_) => PrimitiveType::U128,
        }
    }
}

impl BitLength for Primitive {
    #[inline]
    fn bit_length(&self) -> usize {
        match self {
            Primitive::Bit(_) => 1,
            Primitive::U8(_) => 8,
            Primitive::U16(_) => 16,
            Primitive::U32(_) => 32,
            Primitive::U64(_) => 64,
            Primitive::U128(_) => 128,
        }
    }
}

impl IntoBits for Primitive {
    type IterLsb0 = std::vec::IntoIter<bool>;
    type IterMsb0 = std::vec::IntoIter<bool>;

    fn into_iter_lsb0(self) -> Self::IterLsb0 {
        match self {
            Primitive::Bit(v) => v.into_lsb0_vec().into_iter(),
            Primitive::U8(v) => v.into_lsb0_vec().into_iter(),
            Primitive::U16(v) => v.into_lsb0_vec().into_iter(),
            Primitive::U32(v) => v.into_lsb0_vec().into_iter(),
            Primitive::U64(v) => v.into_lsb0_vec().into_iter(),
            Primitive::U128(v) => v.into_lsb0_vec().into_iter(),
        }
    }

    fn into_iter_msb0(self) -> Self::IterMsb0 {
        match self {
            Primitive::Bit(v) => v.into_msb0_vec().into_iter(),
            Primitive::U8(v) => v.into_msb0_vec().into_iter(),
            Primitive::U16(v) => v.into_msb0_vec().into_iter(),
            Primitive::U32(v) => v.into_msb0_vec().into_iter(),
            Primitive::U64(v) => v.into_msb0_vec().into_iter(),
            Primitive::U128(v) => v.into_msb0_vec().into_iter(),
        }
    }
}

macro_rules! impl_primitive {
    ($(($ty:ty, $ident:ident, $len:literal)),*) => {
        impl_traits!($(($ty, $ident, $len)),*);
        impl_convert!($(($ty, $ident)),*);
    };
}

/// Implements custom traits for primitive types.
macro_rules! impl_traits {
    ($(($ty:ty, $ident:ident, $len:literal)),*) => {
        $(
            impl StaticBitLength for $ty {
                const BIT_LENGTH: usize = $len;
            }

            impl StaticPrimitiveType for $ty {
                const TYPE: PrimitiveType = PrimitiveType::$ident;
            }
        )*
    };
}

/// Implements conversion methods for primitive types.
macro_rules! impl_convert {
    ($(($ty:ty, $ident:ident)),*) => {
        $(
            impl From<$ty> for Primitive {
                fn from(value: $ty) -> Self {
                    match value {
                        value => Primitive::$ident(value),
                    }
                }
            }

            impl From<$ty> for Value {
                fn from(value: $ty) -> Self {
                    Value::Primitive(value.into())
                }
            }

            impl TryFrom<Primitive> for $ty {
                type Error = ValueConvertError;

                fn try_from(primitive: Primitive) -> Result<Self, Self::Error> {
                    match primitive {
                        Primitive::$ident(value) => Ok(value),
                        _ => Err(ValueConvertError::new(
                            PrimitiveType::$ident,
                            primitive.primitive_type(),
                        )),
                    }
                }
            }

            impl TryFrom<Value> for $ty {
                type Error = ValueConvertError;

                fn try_from(value: Value) -> Result<Self, Self::Error> {
                    match value {
                        Value::Primitive(primitive) => Self::try_from(primitive),
                        _ => Err(ValueConvertError::new(
                            PrimitiveType::$ident,
                            value.value_type(),
                        )),
                    }
                }
            }

            impl<const N: usize> TryFrom<Value> for [$ty; N] {
                type Error = ValueConvertError;

                fn try_from(value: Value) -> Result<Self, Self::Error> {
                    match value {
                        Value::Array(array) => {
                            if array.elem_type() != PrimitiveType::$ident {
                                return Err(ValueConvertError::new(
                                    PrimitiveType::$ident,
                                    array.elem_type(),
                                ));
                            }

                            if array.len() != N {
                                return Err(ValueConvertError::new(
                                    ValueType::Array {
                                        ty: PrimitiveType::$ident,
                                        len: N,
                                    },
                                    array.value_type(),
                                ));
                            }

                            let mut iter = array.into_iter().map(TryInto::try_into);
                            let mut result = [<$ty>::default(); N];

                            for value in result.iter_mut() {
                                *value = iter.next().unwrap()?;
                            }

                            Ok(result)
                        }
                        _ => Err(ValueConvertError::new(
                            ValueType::Array {
                                ty: PrimitiveType::$ident,
                                len: N,
                            },
                            value.value_type(),
                        )),
                    }
                }
            }
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
