use std::ops::{BitAnd, BitXor, BitXorAssign};

use crate::{
    encoding::{Bit, Bits, Inactive, State},
    EncryptedRow,
};

impl Bit<Inactive> {
    /// Perform a bitwise AND operation with another bit.
    pub fn and(self, rhs: Bit<Inactive>) -> (Bit<Inactive>, [EncryptedRow; 2]) {
        todo!()
    }
}

impl<S: State> BitXor<Bit<S>> for Bit<S> {
    type Output = Bit<S>;

    fn bitxor(self, rhs: Bit<S>) -> Self::Output {
        Bit::new(self.0 ^ rhs.0)
    }
}

impl<S: State> BitXor<&Bit<S>> for &Bit<S> {
    type Output = Bit<S>;

    fn bitxor(self, rhs: &Bit<S>) -> Self::Output {
        Bit::new(self.0 ^ rhs.0)
    }
}

impl<S: State> BitXorAssign<Bit<S>> for Bit<S> {
    fn bitxor_assign(&mut self, rhs: Bit<S>) {
        self.0 ^= rhs.0;
    }
}

impl<const N: usize, S: State> BitXor<Bits<N, S>> for Bits<N, S> {
    type Output = Bits<N, S>;

    fn bitxor(mut self, rhs: Bits<N, S>) -> Self::Output {
        for (i, bit) in rhs.0.into_iter().rev().enumerate() {
            self.0[N - i] ^= bit;
        }
        self
    }
}
