use std::cell::RefCell;

use mpz_dynamic_types::primitive::{PrimitiveType, StaticPrimitiveType};

use crate::{
    builder::BuilderState,
    repr::binary::{Bit, PrimitiveRepr, ValueRepr},
    Feed, Node,
};

/// A wrapper type for tracing operations applied to a value.
///
/// This type is used to track the operations applied to a value, which
/// is used to build a circuit via a [`CircuitBuilder`](crate::CircuitBuilder).
#[derive(Debug, Clone, Copy)]
pub struct Tracer<'a, T> {
    pub(crate) value: T,
    pub(crate) state: &'a RefCell<BuilderState>,
}

impl<'a, T> Tracer<'a, T> {
    /// Create a new tracer.
    pub fn new(state: &'a RefCell<BuilderState>, value: T) -> Self {
        Self { value, state }
    }

    /// Return the inner value.
    pub fn to_inner(self) -> T {
        self.value
    }
}

impl<'a, T> PrimitiveType for Tracer<'a, T>
where
    T: PrimitiveType,
{
    type Type = T::Type;

    fn primitive_type(&self) -> Self::Type {
        self.value.primitive_type()
    }
}

impl<'a, T> StaticPrimitiveType for Tracer<'a, T>
where
    T: StaticPrimitiveType,
{
    const TYPE: Self::Type = T::TYPE;
}

impl<'a, T> From<Tracer<'a, T>> for PrimitiveRepr
where
    T: Into<PrimitiveRepr>,
{
    fn from(value: Tracer<'a, T>) -> Self {
        value.value.into()
    }
}

impl<'a, T> From<Tracer<'a, T>> for ValueRepr
where
    T: Into<ValueRepr>,
{
    fn from(tracer: Tracer<'a, T>) -> Self {
        tracer.value.into()
    }
}

impl<'a> Tracer<'a, Bit> {
    /// Returns the single node associated with the bit.
    pub fn node(&self) -> Node<Feed> {
        *self.to_inner().id()
    }
}
