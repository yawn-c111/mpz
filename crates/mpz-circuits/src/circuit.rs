use mpz_binary_types::{Value, ValueType};
use mpz_memory::repr::Repr;

use crate::{
    components::{binary::Gate, Registers},
    repr::binary::ValueRepr,
};

/// An error that can occur when performing operations with a circuit.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum CircuitError {
    #[error("Invalid number of inputs: expected {0}, got {1}")]
    InvalidInputCount(usize, usize),
    #[error("Invalid number of outputs: expected {0}, got {1}")]
    InvalidOutputCount(usize, usize),
    #[error("Invalid input type {id}: expected {expected}, got {actual}")]
    InvalidInputType {
        id: usize,
        expected: ValueType,
        actual: ValueType,
    },
    #[error("Invalid output type {id}: expected {expected}, got {actual}")]
    InvalidOutputType {
        id: usize,
        expected: ValueType,
        actual: ValueType,
    },
}

/// A binary circuit.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Circuit {
    pub(crate) inputs: Vec<ValueRepr>,
    pub(crate) outputs: Vec<ValueRepr>,
    pub(crate) gates: Vec<Gate>,
    pub(crate) feed_count: usize,

    pub(crate) and_count: usize,
    pub(crate) xor_count: usize,
}

impl Circuit {
    /// Returns a reference to the inputs of the circuit.
    pub fn inputs(&self) -> &[ValueRepr] {
        &self.inputs
    }

    /// Returns a reference to the outputs of the circuit.
    pub fn outputs(&self) -> &[ValueRepr] {
        &self.outputs
    }

    /// Returns a reference to the gates of the circuit.
    pub fn gates(&self) -> &[Gate] {
        &self.gates
    }

    /// Returns the number of feeds in the circuit.
    pub fn feed_count(&self) -> usize {
        self.feed_count
    }

    /// Returns the number of AND gates in the circuit.
    pub fn and_count(&self) -> usize {
        self.and_count
    }

    /// Returns the number of XOR gates in the circuit.
    pub fn xor_count(&self) -> usize {
        self.xor_count
    }

    /// Reverses the order of the inputs.
    pub fn reverse_inputs(mut self) -> Self {
        self.inputs.reverse();
        self
    }

    /// Reverses endianness of the input at the given index.
    ///
    /// This only has an effect on array inputs.
    ///
    /// # Arguments
    ///
    /// * `idx` - The index of the input to reverse.
    ///
    /// # Returns
    ///
    /// The circuit with the input reversed.
    pub fn reverse_input(mut self, idx: usize) -> Self {
        if let Some(ValueRepr::Array(arr)) = self.inputs.get_mut(idx) {
            arr.reverse();
        }
        self
    }

    /// Reverses the order of the outputs.
    pub fn reverse_outputs(mut self) -> Self {
        self.outputs.reverse();
        self
    }

    /// Reverses endianness of the output at the given index.
    ///
    /// This only has an effect on array outputs.
    ///
    /// # Arguments
    ///
    /// * `idx` - The index of the output to reverse.
    ///
    /// # Returns
    ///
    /// The circuit with the output reversed.
    pub fn reverse_output(mut self, idx: usize) -> Self {
        if let Some(ValueRepr::Array(arr)) = self.outputs.get_mut(idx) {
            arr.reverse();
        }
        self
    }

    /// Evaluate the circuit with the given inputs.
    ///
    /// # Arguments
    ///
    /// * `values` - The inputs to the circuit
    ///
    /// # Returns
    ///
    /// The outputs of the circuit.
    pub fn evaluate(&self, values: &[Value]) -> Result<Vec<Value>, CircuitError> {
        if values.len() != self.inputs.len() {
            return Err(CircuitError::InvalidInputCount(
                self.inputs.len(),
                values.len(),
            ));
        }

        let mut registers = Registers::new(self.feed_count);

        for (id, (input, value)) in self.inputs.iter().zip(values).enumerate() {
            if input.value_type() != value.value_type() {
                return Err(CircuitError::InvalidInputType {
                    id,
                    expected: input.value_type(),
                    actual: value.value_type(),
                });
            }

            input.set(&mut registers, value.clone());
        }

        for gate in self.gates.iter() {
            match *gate {
                Gate::Xor { x, y, z } => {
                    let x = registers[x];
                    let y = registers[y];

                    registers[z] = x ^ y;
                }
                Gate::And { x, y, z } => {
                    let x = registers[x];
                    let y = registers[y];

                    registers[z] = x & y;
                }
                Gate::Inv { x, z } => {
                    let x = registers[x];

                    registers[z] = !x;
                }
            }
        }

        let outputs = self
            .outputs
            .iter()
            .map(|output| output.get(&registers).expect("output is present"))
            .collect();

        Ok(outputs)
    }
}

impl IntoIterator for Circuit {
    type Item = Gate;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.gates.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use mpz_circuits_macros::evaluate;

    use crate::{ops::WrappingAdd, CircuitBuilder};

    use super::*;

    fn build_adder() -> Circuit {
        let builder = CircuitBuilder::new();

        let a = builder.add_input::<u8>();
        let b = builder.add_input::<u8>();

        let c = a.wrapping_add(b);

        builder.add_output(c);

        builder.build().unwrap()
    }

    #[test]
    fn test_evaluate() {
        let circ = build_adder();

        let out = evaluate!(circ, fn(1u8, 2u8) -> u8).unwrap();

        assert_eq!(out, 3u8);
    }
}
