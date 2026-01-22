// Using the Solver is the process of generating the applicative form of
// the state machine to translate, then an internal translation
// converting to a derived state, which the program interrogates to
// transfer ownership and partial amounts.

use crate::applicative::{Applicative, Asset, CompressedApplicative};

use alloc::vec::Vec;

use borsh::{BorshDeserialize, BorshSerialize};

use bobcat_sdk::maths::U;

#[cfg(feature = "std")]
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(SerdeDeserialize, SerdeSerialize))]
pub enum OpSolver {
    // The recursive datatype entrypoint that represents the rolled up form
    // of every interaction. Each excess value creates a new value that could be
    // spent in the same transaction, or committed to the reusable pool of
    // on-chain state. The first argument to the Solve function is the location
    // of the VerifyingKey in the mapping of the keys on-chain.
    Solve(Vec<u64>, Applicative),
}

#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(SerdeDeserialize, SerdeSerialize))]
pub enum CompressedOpSolver {
    Solve(Vec<Asset>, Vec<u64>, CompressedApplicative),
}

impl TryFrom<CompressedOpSolver> for OpSolver {
    type Error = crate::error::Error;
    fn try_from(value: CompressedOpSolver) -> Result<Self, Self::Error> {
        match value {
            CompressedOpSolver::Solve(assets, accounts, c_appl) => Ok(OpSolver::Solve(
                accounts,
                CompressedApplicative::decompress(c_appl, &assets)?,
            )),
        }
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for OpSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_sexpr::to_string(self).unwrap())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SolverFromSexp;

#[cfg(feature = "std")]
impl serde::ser::StdError for SolverFromSexp {}

#[cfg(feature = "std")]
impl std::fmt::Display for SolverFromSexp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "std")]
impl std::str::FromStr for OpSolver {
    type Err = SolverFromSexp;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_sexpr::from_str(s).map_err(|_| SolverFromSexp)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
pub enum OpSetter {
    /// Dummy operation.
    Dummy,
    // Add liquidity to a position, allowing the user to use it later
    // during a balance creation. Verifying key => ed25519 sig => nonce
    // => token => permit blob for moving money to the contract. The
    // verifying key is used to check with the signature if the user
    // created this key for the association. We use the user's sender
    // address to know if they're legitimate.
    Onboard(
        U,        // Verifying key
        [u8; 64], // Verifying signature
        [u8; 20], // Contract
        u16,      // Nonce
        u64,      // Chain
        [u8; 20], // Token
        u128,     // Value
        U,        // Deadline
        u8,       // Permit V
        U,        // Permit R
        U,        // Permit S
    ),
    // Add liquidity to a user's address without completing any onboarding.
    AddLiquidity(
        [u8; 20], // Token
        [u8; 20], // Recipient
        u128,     // Value
        U,        // Deadline
        u8,       // V
        U,        // R
        U,        // S
    ),
    ViewWithdrawable(
        U, // Owner
        U, // Asset
    ),
    ViewOwner(
        U, // Index
    ),
}

#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
pub enum OpAdmin {
    Upgrade(
        [u8; 20], // Solver
        [u8; 20], // Setter
        [u8; 20], // Admin
        [u8; 20], // Vault
    ),
}

#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
pub enum OpVault {
    MoveLiquidity,
}
