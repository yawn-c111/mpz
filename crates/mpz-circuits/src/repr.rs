//! Representations used in circuits.

use crate::components::{Feed, Node};

/// Binary representations.
pub mod binary {
    use super::*;

    /// Value representation used in binary circuits.
    pub type ValueRepr = mpz_memory::repr::binary::ValueRepr<Node<Feed>>;
    /// Primitive type used in binary circuits.
    pub type PrimitiveRepr = mpz_memory::repr::binary::PrimitiveRepr<Node<Feed>>;
    /// Array type used in binary circuits.
    pub type ArrayRepr = mpz_memory::repr::binary::ArrayRepr<Node<Feed>>;

    /// Bit representation.
    pub type Bit = mpz_memory::repr::binary::Bit<Node<Feed>>;
    /// u8 representation.
    pub type U8 = mpz_memory::repr::binary::U8<Node<Feed>>;
    /// u16 representation.
    pub type U16 = mpz_memory::repr::binary::U16<Node<Feed>>;
    /// u32 representation.
    pub type U32 = mpz_memory::repr::binary::U32<Node<Feed>>;
    /// u64 representation.
    pub type U64 = mpz_memory::repr::binary::U64<Node<Feed>>;
    /// u128 representation.
    pub type U128 = mpz_memory::repr::binary::U128<Node<Feed>>;
}
