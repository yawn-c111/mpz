//! Circuit Module
//!
//! Main circuit module.

use crate::{model::Component, Node};
use thiserror::Error;

/// The Circuit Builder assembles a collection of gates into a circuit.
///
/// The built output is ensured to be a directed acyclic graph (DAG).
///
/// The gates are topologically sorted.
#[derive(Debug)]
pub struct CircuitBuilder<T> {
    current_node: Node,
    inputs: Vec<Node>,
    outputs: Vec<Node>,
    gates: Vec<T>,
    stack_size: usize,
}

impl<T> Default for CircuitBuilder<T> {
    fn default() -> Self {
        Self {
            current_node: Node(0),
            inputs: Default::default(),
            outputs: Default::default(),
            gates: Default::default(),
            stack_size: 0,
        }
    }
}

/// Returns the next node.
#[derive(Debug)]
pub struct Next<'a>(&'a mut Node);

impl<'a> Next<'a> {
    /// Returns the next node.
    pub fn next(&mut self) -> Node {
        self.0.next()
    }
}

impl<T> CircuitBuilder<T>
where
    T: Component,
{
    /// Creates a new circuit builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an input to the circuit.
    pub fn add_input(&mut self) -> Node {
        let input = self.current_node.next();
        self.inputs.push(input);
        self.stack_size += 1;
        input
    }

    /// Adds an output to the circuit.
    pub fn add_output(&mut self, node: Node) {
        self.outputs.push(node);
    }

    /// Adds a gate to the circuit.
    ///
    /// This method receives a function for constructing the gate. The input argument,
    /// [`Next`], provides a method for defining the output nodes of the gate.
    pub fn add_gate<F>(&mut self, f: F) -> Result<&T, CircuitBuilderError>
    where
        F: FnOnce(&mut Next) -> T,
    {
        let gate = f(&mut Next(&mut self.current_node));

        let output_count = gate.get_outputs().count();

        if output_count == 0 || gate.get_inputs().count() == 0 {
            return Err(CircuitBuilderError::DisconnectedGate);
        }

        self.stack_size += output_count;

        self.gates.push(gate);

        Ok(self.gates.last().unwrap())
    }

    /// Builds the circuit.
    pub fn build(self) -> Result<Circuit<T>, CircuitBuilderError> {
        if self.gates.is_empty() {
            return Err(CircuitBuilderError::EmptyCircuit);
        }

        let mut gate_inputs = std::collections::HashSet::new();
        let mut gate_outputs = std::collections::HashSet::new();

        for gate in &self.gates {
            for input in gate.get_inputs() {
                if input.0 as usize >= self.stack_size {
                    return Err(CircuitBuilderError::NodeOutOfIndex);
                }
                gate_inputs.insert(*input);
            }

            for output in gate.get_outputs() {
                if output.0 as usize >= self.stack_size {
                    return Err(CircuitBuilderError::NodeOutOfIndex);
                }
                gate_outputs.insert(*output);
            }
        }

        // Verify that output nodes are not inputs to any gate
        if self
            .outputs
            .iter()
            .any(|output| gate_inputs.contains(output))
        {
            return Err(CircuitBuilderError::OutputValidationFailed);
        }

        Ok(Circuit::new(
            self.inputs.len(),
            self.outputs.len(),
            self.gates,
        ))
    }
}

/// A circuit constructed from a collection of gates.
///
/// - Each node in the circuit is an indexed point within an external array.
/// - Each gate acts as a unit of logic that connects these nodes.
#[derive(Debug)]
pub struct Circuit<T> {
    input_count: usize,
    output_count: usize,
    gates: Vec<T>,
}

impl<T> Circuit<T> {
    /// Creates a new circuit.
    fn new(input_count: usize, output_count: usize, gates: Vec<T>) -> Self {
        Self {
            input_count,
            output_count,
            gates,
        }
    }

    /// Returns the number of inputs.
    pub fn input_count(&self) -> usize {
        self.input_count
    }

    /// Returns the number of outputs.
    pub fn output_count(&self) -> usize {
        self.output_count
    }

    /// Returns the gates.
    pub fn gates(&self) -> &[T] {
        &self.gates
    }
}

/// Circuit errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CircuitBuilderError {
    #[error("Disconnected gate")]
    DisconnectedGate,
    #[error("Empty circuit")]
    EmptyCircuit,
    #[error("Output validation failed")]
    OutputValidationFailed,
    #[error("Node out of index")]
    NodeOutOfIndex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Gate {
        inputs: Vec<Node>,
        output: Node,
    }

    impl Component for Gate {
        fn get_inputs(&self) -> impl Iterator<Item = &Node> {
            self.inputs.iter()
        }

        fn get_outputs(&self) -> impl Iterator<Item = &Node> {
            std::iter::once(&self.output)
        }
    }

    #[test]
    fn test_circuit_builder() {
        // Setup circuit builder
        let mut builder = CircuitBuilder::<Gate>::new();

        let (in_0, in_1) = (builder.add_input(), builder.add_input());

        let &Gate { output, .. } = builder
            .add_gate(|next| Gate {
                inputs: vec![in_0, in_1],
                output: next.next(),
            })
            .unwrap();

        let &Gate { output, .. } = builder
            .add_gate(|next| Gate {
                inputs: vec![in_0, output],
                output: next.next(),
            })
            .unwrap();

        let &Gate { output, .. } = builder
            .add_gate(|next| Gate {
                inputs: vec![output, in_1],
                output: next.next(),
            })
            .unwrap();

        builder.add_output(output);

        // Build circuit
        let circuit = builder.build();
        assert!(
            circuit.is_ok(),
            "Failed to build circuit: {:?}",
            circuit.err()
        );
        let circuit = circuit.unwrap();
        let gates = circuit.gates();

        // Verify topological order
        assert_eq!(
            gates[0].get_outputs().collect::<Vec<_>>(),
            vec![&Node(2)],
            "First gate outputs mismatch" // Gate 1
        );
        assert_eq!(
            gates[1].get_outputs().collect::<Vec<_>>(),
            vec![&Node(3)],
            "Second gate outputs mismatch" // Gate 2
        );
        assert_eq!(
            gates[2].get_outputs().collect::<Vec<_>>(),
            vec![&Node(4)],
            "Third gate outputs mismatch" // Gate 3
        );
    }

    #[test]
    fn test_builder_add_gate() {
        // Setup circuit builder
        let mut builder = CircuitBuilder::<Gate>::new();

        let (in_0, in_1) = (builder.add_input(), builder.add_input());

        // Add a valid gate
        let &Gate { .. } = builder
            .add_gate(|next| Gate {
                inputs: vec![in_0, in_1],
                output: next.next(),
            })
            .unwrap();

        // Add a disconnected gate
        let gate_result = builder.add_gate(|next| Gate {
            inputs: Vec::new(),
            output: next.next(),
        });

        assert!(gate_result.is_err(), "Expected disconnected gate error");
        assert_eq!(
            gate_result.unwrap_err(),
            CircuitBuilderError::DisconnectedGate,
            "Unexpected error type"
        );
    }

    #[test]
    fn test_empty_circuit() {
        let builder = CircuitBuilder::<Gate>::new();

        let circuit = builder.build();

        assert!(circuit.is_err(), "Expected empty circuit error");
        assert_eq!(
            circuit.unwrap_err(),
            CircuitBuilderError::EmptyCircuit,
            "Unexpected error type"
        );
    }

    #[test]
    fn test_node_out_of_index() {
        let mut builder = CircuitBuilder::<Gate>::new();

        let input = builder.add_input();

        // Add a gate with an out-of-index node
        builder
            .add_gate(|next| Gate {
                inputs: vec![input, Node(100)],
                output: next.next(),
            })
            .unwrap();

        let circuit = builder.build();

        assert!(circuit.is_err(), "Expected node out of index error");
        assert_eq!(
            circuit.unwrap_err(),
            CircuitBuilderError::NodeOutOfIndex,
            "Unexpected error type"
        );
    }

    #[test]
    fn test_output_validation() {
        let mut builder = CircuitBuilder::<Gate>::new();

        let in_0 = builder.add_input();
        let in_1 = builder.add_input();

        let &Gate { output, .. } = builder
            .add_gate(|next| Gate {
                inputs: vec![in_0, in_1],
                output: next.next(),
            })
            .unwrap();

        builder.add_output(output);

        // Use the output node as an input to a new gate
        let &Gate {
            output: new_output, ..
        } = builder
            .add_gate(|next| Gate {
                inputs: vec![output, in_0],
                output: next.next(),
            })
            .unwrap();

        builder.add_output(new_output);

        let circuit = builder.build();

        assert!(circuit.is_err(), "Expected output validation error");
        assert_eq!(
            circuit.unwrap_err(),
            CircuitBuilderError::OutputValidationFailed,
            "Unexpected error type"
        );
    }
}
