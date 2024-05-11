//! Primitive types.

pub mod binary;

/// A primitive type.
pub trait PrimitiveType {
    /// The primitive type.
    type Type: Copy + PartialEq;

    /// Returns the primitive type.
    fn primitive_type(&self) -> Self::Type;
}

/// A static primitive type.
pub trait StaticPrimitiveType: PrimitiveType {
    /// The primitive type.
    const TYPE: Self::Type;
}
