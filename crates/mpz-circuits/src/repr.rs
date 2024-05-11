//! Representations used in circuits.

use crate::components::{Feed, Node};

/// Binary representations.
pub mod binary {
    use super::*;

    /// Primitive type used in binary circuits.
    pub type PrimitiveRepr = mpz_dynamic_types::repr::binary::BinaryRepr<Node<Feed>>;
    /// Array type used in binary circuits.
    pub type ArrayRepr = mpz_dynamic_types::composite::Array<PrimitiveRepr>;
    /// Value representation used in binary circuits.
    pub type ValueRepr = mpz_dynamic_types::composite::Composite<PrimitiveRepr>;

    /// Bit representation.
    pub type Bit = mpz_dynamic_types::repr::binary::Bit<Node<Feed>>;
    /// u8 representation.
    pub type U8 = mpz_dynamic_types::repr::binary::U8<Node<Feed>>;
    /// u16 representation.
    pub type U16 = mpz_dynamic_types::repr::binary::U16<Node<Feed>>;
    /// u32 representation.
    pub type U32 = mpz_dynamic_types::repr::binary::U32<Node<Feed>>;
    /// u64 representation.
    pub type U64 = mpz_dynamic_types::repr::binary::U64<Node<Feed>>;
    /// u128 representation.
    pub type U128 = mpz_dynamic_types::repr::binary::U128<Node<Feed>>;
}
