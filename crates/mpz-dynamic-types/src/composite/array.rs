extern crate alloc;

use core::{fmt, ops::Index};
use std::error::Error;

use crate::{
    primitive::{PrimitiveType, StaticPrimitiveType},
    repr::Repr,
    ConvertError, MemoryAlloc, MemoryGet, MemoryMut, MemoryReserve,
};

/// Array elements have inconsistent types.
#[derive(Debug, thiserror::Error)]
#[error("elements have an inconsistent type")]
pub struct InconsistentType;

/// An array type.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ArrayType<P> {
    ty: Option<P>,
    len: usize,
}

impl<P> ArrayType<P> {
    /// Creates a new array type.
    pub const fn new(ty: P, len: usize) -> Self {
        Self { ty: Some(ty), len }
    }

    /// Creates a new empty array type.
    pub const fn new_empty() -> Self {
        Self { ty: None, len: 0 }
    }

    /// Returns the length of the array.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }
}

impl<P: Copy> ArrayType<P> {
    /// Returns the element type.
    #[inline]
    pub const fn primitive_type(&self) -> Option<P> {
        self.ty
    }
}

impl<P: Copy> fmt::Display for ArrayType<P>
where
    P: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ty) = self.primitive_type() {
            write!(f, "[{}; {}]", ty, self.len)
        } else {
            write!(f, "[]")
        }
    }
}

/// An array.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Array<P> {
    elems: Vec<P>,
}

impl<P> Array<P> {
    /// Returns the elements of the array.
    #[inline]
    pub fn into_inner(self) -> Vec<P> {
        self.elems
    }

    /// Returns the length of the array.
    #[inline]
    pub fn len(&self) -> usize {
        self.elems.len()
    }

    /// Returns `true` if the array is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }

    /// Returns the element at the given index, or `None` if the index is out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&P> {
        self.elems.as_slice().get(index)
    }

    /// Returns the array as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[P] {
        self.elems.as_slice()
    }

    /// Returns an iterator over the elements.
    #[inline]
    pub fn iter(&self) -> ArrayIter<'_, P> {
        ArrayIter(self.elems.iter())
    }

    /// Returns an iterator that allows mutating each element.
    ///
    /// # Internal
    ///
    /// This method is intended for internal use only.
    #[inline]
    #[doc(hidden)]
    pub fn iter_mut(&mut self) -> ArrayIterMut<'_, P> {
        ArrayIterMut(self.elems.iter_mut())
    }
}

impl<P: PrimitiveType> Array<P> {
    /// Creates a new array.
    #[inline]
    pub fn new(elems: Vec<P>) -> Result<Self, InconsistentType> {
        if let Some(ty) = elems.first().map(|elem| elem.primitive_type()) {
            if elems.iter().all(|elem| elem.primitive_type() == ty) {
                Ok(Array { elems })
            } else {
                Err(InconsistentType)
            }
        } else {
            Ok(Array { elems })
        }
    }

    /// Returns the primitive type of the elements, or `None` if the array is empty.
    #[inline]
    pub fn primitive_type(&self) -> Option<P::Type> {
        self.elems.first().map(|elem| elem.primitive_type())
    }

    /// Returns the array type.
    #[inline]
    pub fn array_type(&self) -> ArrayType<P::Type> {
        if let Some(ty) = self.primitive_type() {
            ArrayType::new(ty, self.elems.len())
        } else {
            ArrayType::new_empty()
        }
    }

    /// Reverses the array in place.
    pub fn reverse(&mut self) {
        self.elems.reverse();
    }

    /// Appends a value to the array. Returns an error if the value has a different type than the other elements
    /// in the array.
    pub fn append(&mut self, value: P) -> Result<(), InconsistentType> {
        if let Some(ty) = self.primitive_type() {
            if value.primitive_type() == ty {
                self.elems.push(value);
            } else {
                return Err(InconsistentType);
            }
        } else {
            self.elems.push(value);
        }

        Ok(())
    }

    /// Extends the array with the given iterator. Returns an error if the iterator contains elements with a
    /// different type than the other elements in the array.
    pub fn extend(&mut self, iter: impl IntoIterator<Item = P>) -> Result<(), InconsistentType> {
        if let Some(ty) = self.primitive_type() {
            for value in iter.into_iter() {
                if value.primitive_type() != ty {
                    return Err(InconsistentType);
                }
                self.elems.push(value);
            }
        } else {
            self.elems.extend(iter);
        }

        Ok(())
    }
}

impl<P> Default for Array<P> {
    #[inline]
    fn default() -> Self {
        Self { elems: Vec::new() }
    }
}

impl<P> Index<usize> for Array<P> {
    type Output = P;

    fn index(&self, index: usize) -> &Self::Output {
        &self.elems[index]
    }
}

impl<'a, T, P> From<&'a [T]> for Array<P>
where
    T: StaticPrimitiveType<Type = P::Type> + Into<P> + Clone,
    P: PrimitiveType,
{
    #[inline]
    fn from(elems: &[T]) -> Self {
        Array::new(elems.into_iter().map(|e| e.clone().into()).collect())
            .expect("array contains consistent types")
    }
}

impl<const N: usize, T, P> From<[T; N]> for Array<P>
where
    T: StaticPrimitiveType<Type = P::Type> + Into<P>,
    P: PrimitiveType,
{
    #[inline]
    fn from(elems: [T; N]) -> Self {
        Array::new(elems.map(|e| e.into()).into_iter().collect())
            .expect("array contains consistent types")
    }
}

impl<'a, const N: usize, T, P> From<&'a [T; N]> for Array<P>
where
    T: StaticPrimitiveType<Type = P::Type> + Into<P> + Clone,
    P: PrimitiveType,
{
    #[inline]
    fn from(elems: &'a [T; N]) -> Self {
        (elems.as_slice()).into()
    }
}

impl<T, P> From<Vec<T>> for Array<P>
where
    T: StaticPrimitiveType<Type = P::Type> + Into<P>,
    P: PrimitiveType,
{
    #[inline]
    fn from(elems: Vec<T>) -> Self {
        Array::new(elems.into_iter().map(|e| e.into()).collect())
            .expect("array contains consistent types")
    }
}

impl<T, P> FromIterator<T> for Array<P>
where
    T: StaticPrimitiveType<Type = P::Type> + Into<P>,
    P: PrimitiveType,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Array::new(iter.into_iter().map(|e| e.into()).collect())
            .expect("array contains consistent types")
    }
}

impl<P> IntoIterator for Array<P> {
    type Item = P;
    type IntoIter = alloc::vec::IntoIter<P>;

    fn into_iter(self) -> Self::IntoIter {
        self.elems.into_iter()
    }
}

impl<const N: usize, P> TryFrom<Array<P>> for [P; N] {
    type Error = <[P; N] as TryFrom<Vec<P>>>::Error;

    fn try_from(value: Array<P>) -> Result<Self, Self::Error> {
        value.elems.try_into()
    }
}

impl<P, T> TryFrom<Array<P>> for Vec<T>
where
    T: TryFrom<P>,
    T::Error: Into<Box<dyn Error + Send + Sync>>,
{
    type Error = ConvertError;

    fn try_from(value: Array<P>) -> Result<Self, Self::Error> {
        value
            .elems
            .into_iter()
            .map(T::try_from)
            .collect::<Result<_, _>>()
            .map_err(ConvertError::new)
    }
}

impl<V, R, M> Repr<Array<V>, M> for Array<R>
where
    V: PrimitiveType,
    R: PrimitiveType<Type = V::Type> + Repr<V, M>,
    <R as Repr<V, M>>::Type: Copy,
{
    type Type = ArrayType<<R as Repr<V, M>>::Type>;

    fn get(&self, mem: &M) -> Option<Array<V>>
    where
        M: MemoryGet,
    {
        Some(
            Array::new(
                self.elems
                    .iter()
                    .map(|elem| elem.get(mem))
                    .collect::<Option<Vec<_>>>()?,
            )
            .expect("array repr points to consistent types"),
        )
    }

    fn set(&self, mem: &mut M, value: Array<V>)
    where
        M: MemoryMut,
    {
        for (elem, elem_v) in self.elems.iter().zip(value.elems) {
            elem.set(mem, elem_v);
        }
    }

    fn alloc(mem: &mut M, value: Array<V>) -> Self
    where
        Self: Sized,
        M: MemoryAlloc,
    {
        Array::new(
            value
                .elems
                .into_iter()
                .map(|elem| R::alloc(mem, elem))
                .collect(),
        )
        .expect("array contains consistent types")
    }

    fn reserve(mem: &mut M, ty: Self::Type) -> Self
    where
        Self: Sized,
        M: MemoryReserve,
    {
        if let Some(elem_ty) = ty.primitive_type() {
            Array::new((0..ty.len()).map(|_| R::reserve(mem, elem_ty)).collect())
                .expect("array contains consistent types")
        } else {
            Array::default()
        }
    }
}

/// An iterator for [`Array`].
#[derive(Debug)]
pub struct ArrayIter<'a, P>(core::slice::Iter<'a, P>);

impl<'a, P> Iterator for ArrayIter<'a, P> {
    type Item = &'a P;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// A mutating iterator for [`Array`].
#[derive(Debug)]
#[doc(hidden)]
pub struct ArrayIterMut<'a, P>(core::slice::IterMut<'a, P>);

impl<'a, P> Iterator for ArrayIterMut<'a, P> {
    type Item = &'a mut P;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[cfg(feature = "serde")]
impl<'de, P> serde::Deserialize<'de> for Array<P>
where
    P: PrimitiveType + for<'a> serde::de::Deserialize<'a>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let elems = Vec::<P>::deserialize(deserializer)?;
        Array::new(elems).map_err(serde::de::Error::custom)
    }
}
