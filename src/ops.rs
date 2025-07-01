
// Using the Solver is the process of generating the applicative form of
// the state machine to translate, then an internal translation
// converting to a derived state, which the program interrogates to
// transfer ownership and partial amounts.

use crate::{applicative::Applicative, encoding::*};

use borsh::{BorshDeserialize, BorshSerialize};

/// The Permit part of the operation. The sender is not included in this
/// input, as it's supplied during a part of the operation instead of this
/// part of the structure.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct Permit {
    pub owner: BAddress,
    pub value: BU256,
    pub deadline: BU256,
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub v: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct DepositUnusedLiquidity {
    pub asset: BAddress,
    pub amount: BU256,
    pub permit: Option<Permit>,
    pub ed_association_addr: EdAddr
}


#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum Op {
    /// Dummy operation.
    Dummy,
    /// Deposit liquidity that can be consumed by the Solve feature in the form
    /// of Balance creation. Makes it possible to roll up the balance creation later.
    DepositUnusedLiquidity(DepositUnusedLiquidity),
    /// Get unused liquidity that can be consumed.
    QueryUnusedLiquidity(BAddress),
    // The recursive datatype entrypoint that represents the rolled up form
    // of every interaction. Each excess value creates a new value that could be
    // spent in the same transaction, or committed to the reusable pool of
    // on-chain state.
    Solve(Applicative)
}
