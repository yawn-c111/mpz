use mpz_core::Block;

mod arithmetic;
mod encoding;

/// An encrypted row of a garbled truth table.
#[derive(Debug, Clone, Copy)]
pub struct EncryptedRow(pub(crate) Block);
