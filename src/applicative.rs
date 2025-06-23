// Applicative form of the state machine, that gets converted to an
// internal representation during the program's simulation.

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
    from: StateBalance,
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
    // Only (Balance | CommitToBalance*) => Order as the argument here.
    Order(ArgsOrder, Sig, Box<Applicative>),
    /// When this type is used, the internal balances are converted to the
    /// derived type that we use to generate the rebalancing of amounts to users.
    // Order * Order => Commit
    Commit(Sig, Box<Applicative>, Box<Applicative>),
    /// A degraded case. This happens if the user does not submit their calldata
    /// in a while. We need to begin anew using another form here that resets
    /// a Commit to a Balance, but only for the left side. In practice, this is
    /// deriving CreateBalanceOrigin for the left side.
    CommitToBalanceLeft(Sig, Box<Applicative>),
    // Let's convert the Commit on the right to a temporarily spendable balance
    // here.
    CommitToBalanceRight(Sig, Box<Applicative>),
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
