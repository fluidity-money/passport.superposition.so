
// Using the Solver is the process of generating the applicative form of
// the state machine to translate, then an internal translation
// converting to a derived state, which the program interrogates to
// transfer ownership and partial amounts.

use crate::encoding::*;

use alloc::vec::Vec;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct Sig {
    pub r: [u8; 32],
    pub r: [u8; 32],
    pub v: u8,
}

pub type EdAddr = [u8; 32];

/// The Permit part of the operation. The sender is not included in this
/// input, as it's supplied during a part of the operation instead of this
/// part of the structure.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct Permit {
    pub owner: BAddress,
    pub value: BU256,
    pub deadline: BU256,
    pub sig: SigPermit,
}

/// Applicative form to derive the previous types with.

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum Op {
    /// Dummy operation.
    Dummy,
    /// Create a spendable Balance that can be consumed by the Solve feature.
    /// Makes it possible to roll up the balance creation later.
    CreateBalance(CreateBalance),
    // The recursive datatype entrypoint that represents the rolled up form
    // of every interaction. Each excess value creates a new value that could be
    // spent in the same transaction, or committed to the reusable pool of
    // on-chain state.
    Solve(Vec<Applicative>)
}
