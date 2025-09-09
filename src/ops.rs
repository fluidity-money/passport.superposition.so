
// Using the Solver is the process of generating the applicative form of
// the state machine to translate, then an internal translation
// converting to a derived state, which the program interrogates to
// transfer ownership and partial amounts.

use crate::{applicative::Applicative, accounts::AccountsList};

use borsh::{BorshDeserialize};

#[cfg(not(target_arch = "wasm32"))]
use borsh::BorshSerialize;

#[derive(BorshDeserialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, BorshSerialize))]
pub enum Op {
    /// Dummy operation.
    Dummy,
    // The recursive datatype entrypoint that represents the rolled up form
    // of every interaction. Each excess value creates a new value that could be
    // spent in the same transaction, or committed to the reusable pool of
    // on-chain state.
    Solve(AccountsList, Applicative)
}
