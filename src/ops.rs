
// Using the Solver is the process of generating the applicative form of
// the state machine to translate, then an internal translation
// converting to a derived state, which the program interrogates to
// transfer ownership and partial amounts.

use crate::{applicative::Applicative};

use alloc::vec::Vec;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
pub struct PermitBlob {
    // The value is copied to the EIP20 contract and used to extract the
    // amount
    pub value: [u8; 32],
    // The deadline of the permit signature.
    pub deadline: [u8; 32],
    pub v: u8,
    pub r: [u8; 32],
    pub s: [u8; 32]
}

#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
pub enum Op {
    /// Dummy operation.
    Dummy,
    // The recursive datatype entrypoint that represents the rolled up form
    // of every interaction. Each excess value creates a new value that could be
    // spent in the same transaction, or committed to the reusable pool of
    // on-chain state. The first argument to the Solve function is the location
    // of the VerifyingKey in the mapping of the keys on-chain.
    Solve(Vec<u64>, Applicative),
    // Add liquidity to a position, allowing the user to use it later during a balance
    // creation. Verifying key => owner => permit blob for moving money to
    // the contract. The amount to spend is taken from the Permit blob, and
    // should not exceed u128.
    Onboard([u8; 32], [u8; 20], PermitBlob),
}
