// Applicative form of the state machine, that gets converted to an
// internal representation during the program's simulation.

use borsh::{BorshSerialize, BorshDeserialize};

use alloc::boxed::Box;

use crate::encoding::*;

pub type Sig = [u8; 32];

/// Balance should be the amount that the user has uncommitted in
/// their entirety.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct ArgsBalance {
    owner: BAddress,
    asset: BAddress,
    chain: u32,
    amount: BU256,
}

/// In the Applicative form, the arguments for the Order are slightly
/// different to also include the amount the user wants to liquidate.
/// Since the Balance should be entirely spent, during the indirection
/// stage to the more fleshed out type, a SplitBalance operation is
/// created.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct ArgsOrder {
    from_asset: BAddress,
    from_amt: BU256,
    desired_asset: BAddress,
    desired_chain: u32,
    desired_amt: BU256,
}

/// It's difficult to use Applicative correctly owing to the lack of
/// type control here. Developers must be cautious when constructing this
/// and going to the state machine internal type!
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub enum Applicative {
    // When this step is used, it's only admissable if the user has
    // uncommitted amounts they've deposited that haven't been converted to a
    // Balance.
    Balance(ArgsBalance),
    /// Withdraw a Balance from the system. The solver signature is needed
    /// alongside the user's signature to be able to testify there are no
    /// unspent UTXOs.
    Withdraw(Sig, Sig, Box<Applicative>),
    // Only (Balance | CommitToBalance*) => Order as the argument here.
    Order(ArgsOrder, Sig, Box<Applicative>),
    /// Cancel an order using a user's signature as well as the matching
    /// engine's signature.
    Cancel(Sig, Sig, Box<Applicative>),
    /// When this type is used, the internal balances are converted to the
    /// derived type that we use to generate the rebalancing of amounts to users.
    /// The first signature argument is the user's signature, and the second is the
    /// matching engine's signature.
    // Order * Order => Commit
    Commit(Sig, Box<Applicative>, Box<Applicative>),
    // Convert the left side of a Commit to a balance, to be reused.
    CommitToBalanceLeft(Sig, Box<Applicative>),
    // Commit the right side of the Commit results to a balance.
    CommitToBalanceRight(Sig, Box<Applicative>)
}

/*
(fulfilled
  (order USDC 55244 10 (balance (alex ETH 55244 1)))
  (order ETH 55244 1 (balance (shahmeer USDC 55244 15))))

Becomes internally before having the machine fleshed out:

  Emissions {
    commit: Commit {
      left: OrderCreated {
        from: StateBalance::Single(
          CreateBalance { owner: Alex, asset: ETH, chain: 55244, amount: 1 }
        ),
        desired_asset: USDC,
        desired_chain: 55244,
        desired_amt: 10
      },
      left_fulfilled: OrderCreated {
        from: StateBalance::Single(
          CreateBalance { owner: Alex, asset: ETH, chain: 55244, amount: 1 }
        ),
        desired_asset: USDC,
        desired_chain: 55244,
        desired_amt: 10
      },
      right: OrderCreated {
        from: StateBalance::Single(
          CreateBalance { owner: Shahmeer, asset: USDC, chain: 55244, amount: 15 }
        ),
        desired_asset: ETH,
        desired_chain: 55244,
        desired_amt: 10
      },
      right_fulfilled: OrderCreated {
        from: StateBalance::Single(
          CreateBalance { owner: Shahmeer, asset: USDC, chain: 55244, amount: 15 }
        ),
        desired_asset: ETH,
        desired_chain: 55244,
        desired_amt: 10
      },
      excess: None
    },
    left_outcome: Outcome {
      asset: USDC,
      recipient: Alex,
      amt: 10
    },
    left_outcome: Outcome {
      asset: ETH,
      recipient: Shahmeer,
      amt: 1
    }
  }
*/
