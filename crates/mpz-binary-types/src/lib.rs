//! Dynamic typing support for binary types.

pub(crate) mod collections;
pub(crate) mod primitive;

use core::fmt;

pub use collections::Array;
use itybity::IntoBits;
pub use primitive::{Primitive, PrimitiveType};

use rand::Rng;

/// A type with a bit length.
pub trait BitLength {
    /// Returns the bit length of the value.
    fn bit_length(&self) -> usize;
}

/// A type with a static bit length.
pub trait StaticBitLength {
    /// The bit length of the value.
    const BIT_LENGTH: usize;
}

/// A static binary type.
pub trait StaticPrimitiveType: StaticBitLength + Into<Primitive> {
    /// The binary type.
    const TYPE: PrimitiveType;
}

/// A static type.
pub trait StaticType: Into<Value> {
    /// The type.
    const TYPE: ValueType;
}

/// A value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ValueType {
    /// A primitive value.
    Primitive(PrimitiveType),
    /// An array value.
    Array {
        /// Type of elements.
        ty: PrimitiveType,
        /// The length of the array.
        len: usize,
    },
}

impl ValueType {
    /// Generates a random value.
    pub fn random<R: Rng + ?Sized>(&self, rng: &mut R) -> Value {
        match self {
            ValueType::Primitive(primitive_type) => Value::Primitive(primitive_type.random(rng)),
            ValueType::Array { ty, len } => Value::Array(Array::random(*ty, *len, rng)),
        }
    }
}

impl BitLength for ValueType {
    #[inline]
    fn bit_length(&self) -> usize {
        match self {
            ValueType::Primitive(primitive_type) => primitive_type.bit_length(),
            ValueType::Array { ty, len } => ty.bit_length() * len,
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Primitive(primitive_type) => write!(f, "{}", primitive_type),
            ValueType::Array { ty, len } => write!(f, "[{}; {}]", ty, len),
        }
    }
}

/// A value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Value {
    /// A primitive value.
    Primitive(Primitive),
    /// An array value.
    Array(Array),
}

impl Value {
    /// Returns the type of the value.
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Primitive(primitive) => ValueType::Primitive(primitive.primitive_type()),
            Value::Array(array) => ValueType::Array {
                ty: array.elem_type(),
                len: array.len(),
            },
        }
    }
}

impl BitLength for Value {
    #[inline]
    fn bit_length(&self) -> usize {
        match self {
            Value::Primitive(primitive) => primitive.bit_length(),
            Value::Array(array) => array.bit_length(),
        }
    }
}

impl IntoBits for Value {
    type IterLsb0 = std::vec::IntoIter<bool>;
    type IterMsb0 = std::vec::IntoIter<bool>;

    fn into_iter_lsb0(self) -> Self::IterLsb0 {
        match self {
            Value::Primitive(primitive) => primitive.into_iter_lsb0(),
            Value::Array(array) => array.into_iter_lsb0(),
        }
    }

    fn into_iter_msb0(self) -> Self::IterMsb0 {
        match self {
            Value::Primitive(primitive) => primitive.into_iter_msb0(),
            Value::Array(array) => array.into_iter_msb0(),
        }
    }
}

/// A value conversion error.
#[derive(Debug, thiserror::Error)]
#[error("attempted to convert a {actual} to a {expected}")]
pub struct ValueConvertError {
    expected: ValueType,
    actual: ValueType,
}

impl ValueConvertError {
    pub(crate) fn new(expected: impl Into<ValueType>, actual: impl Into<ValueType>) -> Self {
        Self {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Returns the expected value type.
    pub fn expected(&self) -> ValueType {
        self.expected
    }

    /// Returns the actual value type.
    pub fn actual(&self) -> ValueType {
        self.actual
    }
}
