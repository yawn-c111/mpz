use std::{
    fmt,
    ops::{Index, Range},
};

use itybity::IntoBits;
use rand::Rng;

use crate::{
    BitLength, Primitive, PrimitiveType, StaticPrimitiveType, StaticType, Value, ValueType,
};

/// An error for [`Array`].
#[derive(Debug, thiserror::Error)]
#[error("array error: {kind}")]
pub struct ArrayError {
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    InconsistentType,
    UnknownType,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::InconsistentType => write!(f, "elements have inconsistent types"),
            ErrorKind::UnknownType => write!(f, "could not determine the type of elements"),
        }
    }
}

/// An array.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "Vec<Primitive>")]
pub struct Array {
    /// Type of elements.
    ty: PrimitiveType,
    /// Elements.
    elems: Vec<Primitive>,
}

impl Array {
    /// Creates a new array.
    pub fn new<T: StaticPrimitiveType>(elems: Vec<T>) -> Self {
        Self {
            ty: T::TYPE,
            elems: elems.into_iter().map(Into::into).collect(),
        }
    }

    /// Creates a new array with a specific type.
    pub fn new_with_type(ty: PrimitiveType, elems: Vec<Primitive>) -> Result<Self, ArrayError> {
        if elems.iter().all(|elem| elem.primitive_type() == ty) {
            Ok(Self { ty, elems })
        } else {
            Err(ArrayError {
                kind: ErrorKind::InconsistentType,
            })
        }
    }

    /// Creates a new array from a vector of elements.
    pub fn try_from_vec(elems: Vec<Primitive>) -> Result<Self, ArrayError> {
        let Some(ty) = elems.first().map(|elem| elem.primitive_type()) else {
            return Err(ArrayError {
                kind: ErrorKind::UnknownType,
            });
        };

        if elems.iter().all(|elem| elem.primitive_type() == ty) {
            Ok(Self { ty, elems })
        } else {
            Err(ArrayError {
                kind: ErrorKind::InconsistentType,
            })
        }
    }

    /// Generates a random array.
    ///
    /// # Arguments
    ///
    /// * `ty` - The type of elements.
    /// * `len` - The length of the array.
    /// * `rng` - The random number generator.
    pub fn random<R: Rng + ?Sized>(ty: PrimitiveType, len: usize, rng: &mut R) -> Self {
        Self {
            ty,
            elems: (0..len).map(|_| ty.random(rng)).collect(),
        }
    }

    /// Returns the length of the array.
    pub fn len(&self) -> usize {
        self.elems.len()
    }

    /// Returns `true` if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }

    /// Returns the type of elements.
    pub fn elem_type(&self) -> PrimitiveType {
        self.ty
    }

    /// Returns the value type.
    pub fn value_type(&self) -> ValueType {
        ValueType::Array {
            ty: self.ty,
            len: self.elems.len(),
        }
    }

    /// Returns a reference to the element at the given index.
    pub fn get(&self, index: usize) -> Option<&Primitive> {
        self.elems.get(index)
    }

    /// Returns the elements as a slice.
    pub fn as_slice(&self) -> &[Primitive] {
        &self.elems
    }

    /// Returns an iterator over the elements.
    pub fn iter(&self) -> impl Iterator<Item = &Primitive> {
        self.elems.iter()
    }
}

impl BitLength for Array {
    #[inline]
    fn bit_length(&self) -> usize {
        self.ty.bit_length() * self.elems.len()
    }
}

impl Index<usize> for Array {
    type Output = Primitive;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.elems[index]
    }
}

impl Index<Range<usize>> for Array {
    type Output = [Primitive];

    #[inline]
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.elems[index]
    }
}

impl AsRef<[Primitive]> for Array {
    #[inline]
    fn as_ref(&self) -> &[Primitive] {
        self.as_slice()
    }
}

impl IntoIterator for Array {
    type Item = Primitive;
    type IntoIter = std::vec::IntoIter<Primitive>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.elems.into_iter()
    }
}

impl<T> FromIterator<T> for Array
where
    T: StaticPrimitiveType,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Array {
            ty: T::TYPE,
            elems: iter.into_iter().map(Into::into).collect(),
        }
    }
}

impl<const N: usize, T> StaticType for [T; N]
where
    T: StaticPrimitiveType,
{
    const TYPE: ValueType = ValueType::Array {
        ty: T::TYPE,
        len: N,
    };
}

impl<const N: usize, T> From<[T; N]> for Array
where
    T: StaticPrimitiveType,
{
    #[inline]
    fn from(value: [T; N]) -> Self {
        Array {
            ty: T::TYPE,
            elems: value.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a, T> From<&'a [T]> for Array
where
    T: StaticPrimitiveType + Clone,
{
    #[inline]
    fn from(value: &'a [T]) -> Self {
        Array {
            ty: T::TYPE,
            elems: value.iter().cloned().map(Into::into).collect(),
        }
    }
}

impl<T> From<Vec<T>> for Array
where
    T: StaticPrimitiveType,
{
    #[inline]
    fn from(value: Vec<T>) -> Self {
        Array {
            ty: T::TYPE,
            elems: value.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<Vec<Primitive>> for Array {
    type Error = ArrayError;

    fn try_from(value: Vec<Primitive>) -> Result<Self, Self::Error> {
        Array::try_from_vec(value)
    }
}

impl<T> From<T> for Value
where
    T: Into<Array>,
{
    #[inline]
    fn from(value: T) -> Self {
        Value::Array(value.into())
    }
}

impl From<Array> for Vec<Primitive> {
    #[inline]
    fn from(value: Array) -> Self {
        value.elems
    }
}

impl IntoBits for Array {
    type IterLsb0 = std::vec::IntoIter<bool>;
    type IterMsb0 = std::vec::IntoIter<bool>;

    fn into_iter_lsb0(self) -> Self::IterLsb0 {
        self.elems
            .into_iter()
            .flat_map(|elem| elem.into_iter_lsb0())
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn into_iter_msb0(self) -> Self::IterMsb0 {
        self.elems
            .into_iter()
            .flat_map(|elem| elem.into_iter_msb0())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_inconsistent_types() {
        let elems = vec![Primitive::U8(1), Primitive::U16(2)];
        let err = Array::try_from_vec(elems).unwrap_err();

        assert!(matches!(err.kind, ErrorKind::InconsistentType));
    }

    #[test]
    fn test_array_unknown_type() {
        let elems = vec![];
        let err = Array::try_from_vec(elems).unwrap_err();

        assert!(matches!(err.kind, ErrorKind::UnknownType));
    }
}
