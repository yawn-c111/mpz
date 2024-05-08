//! This crate provides types for representing computation as binary circuits.

#![deny(missing_docs, unreachable_pub, unused_must_use)]

extern crate self as mpz_circuits;

mod builder;
mod circuit;
pub mod circuits;
pub(crate) mod components;
pub mod ops;
#[cfg(feature = "parse")]
mod parse;
pub mod repr;
mod tracer;

#[doc(hidden)]
pub use builder::BuilderState;
pub use builder::{BuilderError, CircuitBuilder};
pub use circuit::{Circuit, CircuitError};
pub use components::binary::{Gate, GateType};
#[doc(hidden)]
pub use components::{Feed, Node, Sink};
pub use tracer::Tracer;

pub use once_cell;

pub use mpz_binary_types::{Array, Primitive, PrimitiveType, Value, ValueType};

/// Evaluates a circuit and attempts to coerce the output into the specified return type
/// indicated in the function signature.
///
/// # Returns
///
/// The macro returns a `Result` with the output of the circuit or a [`TypeError`](crate::types::TypeError) if the
/// output could not be coerced into the specified return type.
///
/// `Result<T, TypeError>`
///
/// # Example
///
/// ```
/// # let circ = {
/// #    use mpz_circuits::{CircuitBuilder, ops::WrappingAdd};
/// #
/// #    let builder = CircuitBuilder::new();
/// #    let a = builder.add_input::<u8>();
/// #    let b = builder.add_input::<u8>();
/// #    let c = a.wrapping_add(b);
/// #    builder.add_output(c);
/// #    builder.build().unwrap()
/// # };
/// use mpz_circuits::evaluate;
///
/// let output: u8 = evaluate!(circ, fn(1u8, 2u8) -> u8).unwrap();
///
/// assert_eq!(output, 1u8 + 2u8);
/// ```
pub use mpz_circuits_macros::evaluate;

/// Helper macro for testing that a circuit evaluates to the expected value.
///
/// # Example
///
/// ```
/// # let circ = {
/// #    use mpz_circuits::{CircuitBuilder, ops::WrappingAdd};
/// #
/// #    let builder = CircuitBuilder::new();
/// #    let a = builder.add_input::<u8>();
/// #    let b = builder.add_input::<u8>();
/// #    let c = a.wrapping_add(b);
/// #    builder.add_output(c);
/// #    builder.build().unwrap()
/// # };
/// use mpz_circuits::test_circ;
///
/// fn wrapping_add(a: u8, b: u8) -> u8 {
///    a.wrapping_add(b)
/// }
///
/// test_circ!(circ, wrapping_add, fn(1u8, 2u8) -> u8);
/// ```
pub use mpz_circuits_macros::test_circ;
