//! Messages used in garbled circuit protocols.

use mpz_core::{commit::Decommitment, hash::Hash};
use serde::{Deserialize, Serialize};

use crate::{
    circuit::EncryptedGate, encoding_state, Decoding, Delta, EncodedValue, EncodingCommitment,
    EqualityCheck,
};

/// A status sent from the generator to the evaluator to indicate whether the garbling is done.
#[derive(Debug, Serialize, Deserialize)]
pub enum Status<T> {
    /// Garbling is still active.
    Active(T),
    /// Garbling is done.
    Done,
}

/// Top-level message type encapsulating all messages used in garbled circuit protocols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum GarbleMessage {
    ActiveValue(Box<EncodedValue<encoding_state::Active>>),
    ActiveValues(Vec<EncodedValue<encoding_state::Active>>),
    EncryptedGates(Vec<EncryptedGate>),
    EncodingCommitments(Vec<EncodingCommitment>),
    ValueDecoding(Box<Decoding>),
    ValueDecodings(Vec<Decoding>),
    EqualityCheck(EqualityCheck),
    HashCommitment(Hash),
    EqualityCheckDecommitment(Decommitment<EqualityCheck>),
    EqualityCheckDecommitments(Vec<Decommitment<EqualityCheck>>),
    ProofDecommitments(Vec<Decommitment<Hash>>),
    Delta(Delta),
    EncoderSeed(Vec<u8>),
}
