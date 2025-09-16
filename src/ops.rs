// Using the Solver is the process of generating the applicative form of
// the state machine to translate, then an internal translation
// converting to a derived state, which the program interrogates to
// transfer ownership and partial amounts.

use crate::applicative::Applicative;

use alloc::vec::Vec;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
pub enum OpSolver {
    /// Dummy operation.
    Dummy,
    // The recursive datatype entrypoint that represents the rolled up form
    // of every interaction. Each excess value creates a new value that could be
    // spent in the same transaction, or committed to the reusable pool of
    // on-chain state. The first argument to the Solve function is the location
    // of the VerifyingKey in the mapping of the keys on-chain.
    Solve(Vec<u64>, Applicative),
}

#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
pub enum OpSetter {
    /// Dummy operation.
    Dummy,
    // Add liquidity to a position, allowing the user to use it later during a balance
    // creation. Verifying key => ed25519 sig => nonce => owner => onboard v
    // => onboard r => onboard s => token => permit blob for moving money to
    // the contract. The verifying key is used to check with the signature if the user
    // created this key for the association, and the other part of the signature is
    // used to check if the user's wallet authorises the signature. The permit blob is used
    // to onramp the user.
    Onboard(
        [u8; 32], // Verifying key
        [u8; 64], // Verifying signature
        u16,      // Nonce
        [u8; 20], // Owner
        u8,       // Onboard v
        [u8; 32], // Onboard r
        [u8; 32], // Onboard s
        [u8; 20], // Token
        u128,     // Value
        [u8; 32], // Deadline
        u8,       // Permit V
        [u8; 32], // Permit R
        [u8; 32], // Permit S
    ),
    // Add liquidity to a user's address without completing any onboarding.
    AddLiquidity([u8; 20], [u8; 20], u128, [u8; 32], u8, [u8; 32], [u8; 32]),
}
