//! Model module.
//!
//! This module contains the main traits and structures used to represent the circuits.

/// A `Component` defines a block with inputs and outputs.
pub trait Component {
    /// Returns an iterator over the input node indices.
    fn get_inputs(&self) -> impl Iterator<Item = &Node>;

    /// Returns an iterator over the output node indices.
    fn get_outputs(&self) -> impl Iterator<Item = &Node>;
}

/// A circuit node.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Node(pub(crate) u32);

impl Node {
    pub(crate) fn next(&mut self) -> Self {
        let prev = self.0;
        self.0 += 1;
        Self(prev)
    }
}
