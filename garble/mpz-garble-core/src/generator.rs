use std::{sync::Arc, marker::PhantomData};

use blake3::Hasher;

use crate::{
    encoding::{state, Delta, EncodedValue, Label}, EncryptedRow,
    mode::GarbleMode, Normal
};
use mpz_circuits::{types::TypeError, Circuit, CircuitError, Gate};
use mpz_core::{
    aes::{FixedKeyAes, FIXED_KEY_AES},
    hash::Hash,
    Block,
};

/// Errors that can occur during garbled circuit generation.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum GeneratorError {
    #[error(transparent)]
    TypeError(#[from] TypeError),
    #[error(transparent)]
    CircuitError(#[from] CircuitError),
    #[error("generator not finished")]
    NotFinished,
}

/// Computes half-gate garbled AND gate
/// 
/// Returns the output label and the encrypted of the generator and evaluator respectively.
#[inline]
pub(crate) fn and_gate(
    cipher: &FixedKeyAes,
    x_0: &Label,
    y_0: &Label,
    delta: &Delta,
    gid: usize,
    rows: &mut Vec<EncryptedRow>,
) -> Label {
    let delta = delta.into_inner();
    let x_0 = x_0.to_inner();
    let x_1 = x_0 ^ delta;
    let y_0 = y_0.to_inner();
    let y_1 = y_0 ^ delta;

    let p_a = x_0.lsb();
    let p_b = y_0.lsb();
    let j = Block::new((gid as u128).to_be_bytes());
    let k = Block::new(((gid + 1) as u128).to_be_bytes());

    let mut h = [x_0, y_0, x_1, y_1];
    cipher.tccr_many(&[j, k, j, k], &mut h);

    let [hx_0, hy_0, hx_1, hy_1] = h;

    // Garbled row of generator half-gate
    let t_g = hx_0 ^ hx_1 ^ (Block::SELECT_MASK[p_b] & delta);
    let w_g = hx_0 ^ (Block::SELECT_MASK[p_a] & t_g);

    // Garbled row of evaluator half-gate
    let t_e = hy_0 ^ hy_1 ^ x_0;
    let w_e = hy_0 ^ (Block::SELECT_MASK[p_b] & (t_e ^ x_0));

    rows.push(EncryptedRow(t_g));
    rows.push(EncryptedRow(t_e));

    Label::new(w_g ^ w_e)
}

/// Computes half-gate privacy-free garbled AND gate
#[inline]
pub(crate) fn and_gate_pf(
    cipher: &FixedKeyAes,
    x_0: &Label,
    y_0: &Label,
    delta: &Delta,
    gid: usize,
    rows: &mut Vec<EncryptedRow>,
) -> Label {
    let delta = delta.into_inner();
    let x_0 = x_0.to_inner();
    let x_1 = x_0 ^ delta;
    let y_0 = y_0.to_inner();

    let j = Block::new((gid as u128).to_be_bytes());

    let mut h = [x_0, x_1];
    cipher.tccr_many(&[j, j], &mut h);

    let [mut hx_0, mut hx_1] = h;
    hx_0.clear_lsb();
    hx_1.set_lsb();

    // Garbled row of evaluator half-gate
    let t_e = hx_0 ^ hx_1 ^ y_0;
    let z_0 = Label::new(hx_0);

    rows.push(EncryptedRow(t_e));

    z_0
}

/// Core generator type used to generate garbled circuits.
///
/// A generator is to be used as an iterator of encrypted gates. Each
/// iteration will return the next encrypted gate in the circuit until the
/// entire garbled circuit has been yielded.
pub struct Generator<M: GarbleMode = Normal> {
    /// Cipher to use to encrypt the gates
    cipher: &'static FixedKeyAes,
    /// Circuit to generate a garbled circuit for
    circ: Arc<Circuit>,
    /// Delta value to use while generating the circuit
    delta: Delta,
    /// The 0 bit labels for the garbled circuit
    low_labels: Vec<Option<Label>>,
    /// Current position in the circuit
    pos: usize,
    /// Current gate id
    gid: usize,
    /// Hasher to use to hash the encrypted gates
    hasher: Option<Hasher>,
    _mode: PhantomData<M>
}

impl<M: GarbleMode> Generator<M> {
    /// Creates a new generator for the given circuit.
    ///
    /// # Arguments
    ///
    /// * `circ` - The circuit to generate a garbled circuit for.
    /// * `delta` - The delta value to use.
    /// * `inputs` - The inputs to the circuit.
    pub fn new(
        circ: Arc<Circuit>,
        delta: Delta,
        inputs: &[EncodedValue<state::Full>],
    ) -> Result<Self, GeneratorError> {
        Self::new_with(circ, delta, inputs, None)
    }

    /// Creates a new generator for the given circuit. Generator will compute a hash
    /// of the encrypted gates while they are produced.
    ///
    /// # Arguments
    ///
    /// * `circ` - The circuit to generate a garbled circuit for.
    /// * `delta` - The delta value to use.
    /// * `inputs` - The inputs to the circuit.
    pub fn new_with_hasher(
        circ: Arc<Circuit>,
        delta: Delta,
        inputs: &[EncodedValue<state::Full>],
    ) -> Result<Self, GeneratorError> {
        Self::new_with(circ, delta, inputs, Some(Hasher::new()))
    }

    fn new_with(
        circ: Arc<Circuit>,
        delta: Delta,
        inputs: &[EncodedValue<state::Full>],
        hasher: Option<Hasher>,
    ) -> Result<Self, GeneratorError> {
        if inputs.len() != circ.inputs().len() {
            return Err(CircuitError::InvalidInputCount(
                circ.inputs().len(),
                inputs.len(),
            ))?;
        }

        let mut low_labels: Vec<Option<Label>> = vec![None; circ.feed_count()];
        for (encoded, input) in inputs.iter().zip(circ.inputs()) {
            if encoded.value_type() != input.value_type() {
                return Err(TypeError::UnexpectedType {
                    expected: input.value_type(),
                    actual: encoded.value_type(),
                })?;
            }

            for (label, node) in encoded.iter().zip(input.iter()) {
                low_labels[node.id()] = Some(*label);
            }
        }

        Ok(Self {
            cipher: &(*FIXED_KEY_AES),
            circ,
            delta,
            low_labels,
            pos: 0,
            gid: 1,
            hasher,
            _mode: PhantomData
        })
    }

    /// Returns whether the generator has finished generating the circuit.
    pub fn is_complete(&self) -> bool {
        self.pos >= self.circ.gates().len()
    }

    /// Returns the encoded outputs of the circuit.
    pub fn outputs(&self) -> Result<Vec<EncodedValue<state::Full>>, GeneratorError> {
        if !self.is_complete() {
            return Err(GeneratorError::NotFinished);
        }

        Ok(self
            .circ
            .outputs()
            .iter()
            .map(|output| {
                let labels: Vec<Label> = output
                    .iter()
                    .map(|node| self.low_labels[node.id()].expect("feed should be initialized"))
                    .collect();

                EncodedValue::<state::Full>::from_labels(output.value_type(), self.delta, &labels)
                    .expect("encoding should be correct")
            })
            .collect())
    }

    /// Returns the hash of the encrypted gates.
    pub fn hash(&self) -> Option<Hash> {
        self.hasher.as_ref().map(|hasher| {
            let hash: [u8; 32] = hasher.finalize().into();
            Hash::from(hash)
        })
    }

    /// Garbles the circuit and returns the encrypted rows.
    /// 
    /// # Arguments
    /// 
    /// * `limit` - The maximum number of encrypted rows to generate (rounded up to the nearest multiple of 2).
    ///             If set to 0, the generator will generate all the encrypted rows.
    pub fn generate(&mut self, limit: usize) -> Vec<EncryptedRow> {
        if self.is_complete() {
            return Vec::new();
        }

        let limit = if limit == 0 {
            self.circ.and_count() * M::ROWS_PER_AND_GATE
        } else {
            limit.next_power_of_two()
        };

        let low_labels = &mut self.low_labels;
        let mut encrypted_rows = Vec::with_capacity(limit);

        for gate in &self.circ.gates()[self.pos..] {
            match gate {
                Gate::Inv {
                    x: node_x,
                    z: node_z,
                } => {
                    let x_0 = low_labels[node_x.id()].expect("feed should be initialized");
                    low_labels[node_z.id()] = Some(x_0 ^ self.delta);
                }
                Gate::Xor {
                    x: node_x,
                    y: node_y,
                    z: node_z,
                } => {
                    let x_0 = low_labels[node_x.id()].expect("feed should be initialized");
                    let y_0 = low_labels[node_y.id()].expect("feed should be initialized");
                    low_labels[node_z.id()] = Some(x_0 ^ y_0);
                }
                Gate::And {
                    x: node_x,
                    y: node_y,
                    z: node_z,
                } => {
                    if encrypted_rows.len() >= limit {
                        break;
                    }

                    let x_0 = &low_labels[node_x.id()].expect("feed should be initialized");
                    let y_0 = &low_labels[node_y.id()].expect("feed should be initialized");
                    let z_0 = M::garble_and_gate(self.cipher, x_0, y_0, &self.delta, self.gid, &mut encrypted_rows);
                    low_labels[node_z.id()] = Some(z_0);
                    self.gid += 2;
                }
            }
            self.pos += 1;
        }

        if let Some(hasher) = &mut self.hasher {
            for row in &encrypted_rows {
                hasher.update(&row.0.to_bytes());
            }
        }

        encrypted_rows
    }
}

#[cfg(test)]
mod tests {
    use crate::{ChaChaEncoder, Encoder};
    use mpz_circuits::circuits::AES128;

    use super::*;

    #[test]
    fn test_generator() {
        let encoder = ChaChaEncoder::new([0; 32]);
        let inputs: Vec<_> = AES128
            .inputs()
            .iter()
            .map(|input| encoder.encode_by_type(0, &input.value_type()))
            .collect();

        let mut gen = Generator::<Normal>::new_with_hasher(AES128.clone(), encoder.delta(), &inputs).unwrap();

        let rows = gen.generate(0);

        assert!(gen.is_complete());
        assert_eq!(rows.len(), AES128.and_count() * Normal::ROWS_PER_AND_GATE);

        let _ = gen.outputs().unwrap();
        let _ = gen.hash().unwrap();
    }
}
