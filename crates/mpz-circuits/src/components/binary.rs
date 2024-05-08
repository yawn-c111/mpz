//! Binary circuit components.

use crate::components::{Feed, Node, Sink};

/// A binary logic gate.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Gate {
    /// XOR gate.
    Xor {
        x: Node<Sink>,
        y: Node<Sink>,
        z: Node<Feed>,
    },
    /// AND gate.
    And {
        x: Node<Sink>,
        y: Node<Sink>,
        z: Node<Feed>,
    },
    /// Inverter gate.
    Inv { x: Node<Sink>, z: Node<Feed> },
}

impl Gate {
    /// Returns the type of the gate.
    pub fn gate_type(&self) -> GateType {
        match self {
            Gate::Xor { .. } => GateType::Xor,
            Gate::And { .. } => GateType::And,
            Gate::Inv { .. } => GateType::Inv,
        }
    }

    /// Returns the x input of the gate.
    pub fn x(&self) -> Node<Sink> {
        match self {
            Gate::Xor { x, .. } => *x,
            Gate::And { x, .. } => *x,
            Gate::Inv { x, .. } => *x,
        }
    }

    /// Returns the y input of the gate.
    pub fn y(&self) -> Option<Node<Sink>> {
        match self {
            Gate::Xor { y, .. } => Some(*y),
            Gate::And { y, .. } => Some(*y),
            Gate::Inv { .. } => None,
        }
    }

    /// Returns the z output of the gate.
    pub fn z(&self) -> Node<Feed> {
        match self {
            Gate::Xor { z, .. } => *z,
            Gate::And { z, .. } => *z,
            Gate::Inv { z, .. } => *z,
        }
    }

    /// Shifts all the node IDs of the gate by the given offset.
    #[inline]
    pub(crate) fn shift_left(&mut self, offset: usize) {
        match self {
            Gate::Xor { x, y, z } => {
                x.id -= offset;
                y.id -= offset;
                z.id -= offset;
            }
            Gate::And { x, y, z } => {
                x.id -= offset;
                y.id -= offset;
                z.id -= offset;
            }
            Gate::Inv { x, z } => {
                x.id -= offset;
                z.id -= offset;
            }
        }
    }
}

/// The type of a binary gate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GateType {
    /// XOR gate.
    Xor,
    /// AND gate.
    And,
    /// Inverter gate.
    Inv,
}
