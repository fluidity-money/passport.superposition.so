// Applicative form of the state machine, that gets converted to an
// internal representation during the program's simulation.

use borsh::{BorshDeserialize, BorshSerialize};

use alloc::boxed::Box;

use crate::encoding::*;

// Concatenated form of the ed25519 r and s values for use with
// ed25519_dalek.
pub type EdSig = [u8; 64];

/// Ed25519 signature. Referenced according to its place in the accounts
/// vector.
pub type EdAddr = usize;

/// User provided signature. Needs a lookup in the accounts table.
pub type UserSig = (EdAddr, EdSig);

/// Solver provided signature.
pub type SolverSig = EdSig;

/// Balance should be the amount that the user has uncommitted in
/// their entirety.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary)
)]
pub struct ArgsBalance {
    asset: BAddress,
    chain: u32,
    amount: BU256,
    // Owner and timestamp (milliseconds) are combined to create a snowflake.
    owner: EdAddr,
    ms_timestamp: u128,
}

/// In the Applicative form, the arguments for the Order are slightly
/// different to also include the amount the user wants to liquidate.
/// Since the Balance should be entirely spent, during the indirection
/// stage to the more fleshed out type, a SplitBalance operation is
/// created.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct ArgsOrder {
    from_amt: BU256,
    desired_asset: BAddress,
    desired_chain: u32,
    desired_amt: BU256,
}

/// It's difficult to use Applicative correctly owing to the lack of
/// type control here. Developers must be cautious when constructing this
/// and going to the state machine internal type! When a Balance is
/// constructed, but the consumer of the Balance opts to spend less than
/// they otherwise could've, to the state machine it's implicitly a split
/// balance operation. The Applicative user must rejoin balances when
/// it suits them, but it's not important for them to split balances.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub enum Applicative {
    /// When this step is used, it's only admissable if the user has
    /// uncommitted amounts they've deposited that haven't been converted to a
    /// Balance.
    Balance(EdSig, ArgsBalance),
    /// Withdraw a Balance from the system. The solver signature is needed
    /// alongside the user's signature to be able to testify there are no
    /// unspent UTXOs. The signature only needs to be the signed value
    /// of the concatenation of the hash of the
    /// (Balance|CommitLeftFilledToBalance|CommitRightFilledToBalance|CommitLeftExcessToBalance|CommitRightExcessToBalance)
    /// input from the solver and the user.
    Withdraw(EdSig, EdSig, Box<Applicative>),
    /// Only (Balance | CommitToBalance*) => Order as the argument here.
    /// Consumes a Balance. The signature is the concatenation of the inputs,
    /// and the hash from the balance.
    /// (Balance|CommitLeftFilledToBalance|CommitRightFilledToBalance|CommitLeftExcessToBalance|CommitRightExcessToBalance)
    Order(SolverSig, ArgsOrder, Box<Applicative>),
    /// Cancel an order using a user's signature as well as the matching
    /// engine's signature.
    Cancel(SolverSig, UserSig, Box<Applicative>),
    /// When this type is used, the internal balances are converted to the
    /// derived type that we use to generate the rebalancing of amounts to users.
    /// The first signature argument is the user's signature, and the second is the
    /// matching engine's signature.
    // Order * Order => Commit
    Commit(SolverSig, Box<Applicative>, Box<Applicative>),
    // Convert the left side of a Commit to a balance, to be reused.
    CommitLeftFilledToBalance(Box<Applicative>),
    // Commit the right side of the Commit results to a balance.
    CommitRightFilledToBalance(Box<Applicative>),
    // Convert the leftover amount on the left side of a partial order match to a
    // Balance. This is useful if the order doesn't fill properly! Internally,
    // this has the identifier of a balance created using the snowflake function
    // of the original Balance identifier, incremented by one.
    CommitLeftExcessToBalance(UserSig, Box<Applicative>),
    // Convert the leftover amount on the right side to a Balance. Internally,
    // this has the identifier of a balance created using the snowflake function
    // of the original Balance identifier, incremented by one.
    CommitRightExcessToBalance(UserSig, Box<Applicative>),
    /// Join two balances together.
    Join(UserSig, Box<Applicative>, Box<Applicative>),
}

/*
Simplified explainer of the state transition:

digraph {
  Balance -> Withdraw
  Balance -> Order -> Cancel
  Order -> Commit -> {
      CommitToBalanceLeft
      CommitToBalanceRight
      ExcessToBalanceLeft
      ExcessToBalanceRight
  } -> Balance
}
*/
