mod unsigned;

use std::marker::PhantomData;

use mpz_core::Block;
use rand::distributions::{Distribution, Standard};

/// An encoded bit.
#[derive(Debug, Clone, Copy)]
pub struct Bit<S: State>(pub(crate) Block, PhantomData<S>);

impl<S: State> Bit<S> {
    #[inline]
    pub(crate) fn new(block: Block) -> Self {
        Bit(block, PhantomData)
    }

    /// The label of the encoded bit.
    pub(crate) fn label(&self) -> &Block {
        &self.0
    }

    /// The encodings pointer bit from the point-and-permute technique.
    pub(crate) fn pointer_bit(&self) -> bool {
        self.0.lsb() == 1
    }
}

impl Distribution<Bit<Inactive>> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Bit<Inactive> {
        Bit(self.sample(rng), PhantomData)
    }
}

/// A bag of encoded bits.
#[derive(Debug, Clone, Copy)]
pub struct Bits<const N: usize, S: State>(pub(crate) [Bit<S>; N]);

impl<const N: usize, S: State> Bits<N, S> {
    #[inline]
    pub(crate) fn new(bits: [Bit<S>; N]) -> Self {
        Bits(bits)
    }
}

/// Global binary offset used by the Free-XOR technique to create label
/// pairs where W_1 = W_0 ^ Delta.
///
/// In accordance with the (p&p) Point-and-Permute technique, the LSB of Delta is set to 1, so that
/// the pointer bit LSB(W_1) = LSB(W_0) ^ 1
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Delta(Block);

impl Distribution<Delta> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Delta {
        let mut block: Block = self.sample(rng);
        block.set_lsb();
        Delta(block)
    }
}

mod state {
    mod sealed {
        /// Sealed trait.
        pub trait Sealed {}
    }

    /// Bit state.
    pub trait State: sealed::Sealed {}

    /// Active encoding state.
    ///
    /// An encoded value which has been assigned.
    #[derive(Debug, Clone, Copy)]
    pub struct Active;

    impl State for Active {}
    impl sealed::Sealed for Active {}

    /// Inactive encoding state.
    ///
    /// An encoded value which has not been assigned.
    #[derive(Debug, Clone, Copy)]
    pub struct Inactive;

    impl State for Inactive {}
    impl sealed::Sealed for Inactive {}
}

pub use state::{Active, Inactive, State};
