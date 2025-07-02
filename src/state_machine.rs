use crate::encoding::*;

use alloc::boxed::Box;

// Creates a spendable Balance UTXO, without the sending of the Permit
// blob to provide the liquidity to create this type.
#[derive(Clone, PartialEq, Debug)]
pub struct CreateBalance {
    owner: BAddress,
    asset: BAddress,
    chain: u32,
    amount: BU256,
    ms_ts: u128,
}

/// Balances are identifiable in their descended form using the
/// concatenation of the previous hash, the timestamp of the change, and
/// the nonce here.
#[repr(C)]
pub enum SnowflakeNonce {
    CreateBalance,
    SPLIT_BALANCE_EXCESS,
    COMMIT_FULFILLED_LEFT,
    COMMIT_FULFILLED_RIGHT,
    COMMIT_EXCESS_LEFT,
    COMMIT_EXCESS_RIGHT,
    JOIN_BALANCE,
}

// Gets translated into from ArgsBalance compared to what we know about
// the user.
#[derive(Clone, PartialEq, Debug)]
pub enum CreateBalanceOrigin {
    /// Smply creating a balance using a Permit signature on-chain!
    Single(CreateBalance),
    /// A commit took place and we're transforming the left filled side of the result.
    CommitSpendableLeft(Commit, CreateBalance),
    /// A commit took place and we're transforming the right filled side of the result.
    CommitSpendableRight(Commit, CreateBalance),
    /// A commit took place, and we're taking the excess order on the left.
    CommitExcessLeft(Commit, CreateBalance),
    /// A commit took place, and we're taking the excess order on the right.
    CommitExcessRight(Commit, CreateBalance),
    /// A join took place from a pair of balances!
    Join(Box<CreateBalanceOrigin>, Box<CreateBalanceOrigin>),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Withdrawal {
    from: CreateBalanceOrigin,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SplitBalance {
    /// The input balance.
    pub input: Box<CreateBalanceOrigin>,
    /// The balance that would be spent in a recursive use of this state.
    pub spendable: CreateBalance,
    /// The excess balance that should be reused later.
    pub excess: CreateBalance,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SplitBalanceSpendable {
    pub from: SplitBalance,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SplitBalanceSpendExcess {
    pub from: SplitBalance,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StateBalance {
    SplitExcess(SplitBalanceSpendExcess),
    SplitSpendable(SplitBalanceSpendable),
    Single(CreateBalance),
}

#[derive(Clone, PartialEq, Debug)]
pub struct OrderCreated {
    desired_asset: BAddress,
    desired_chain: u32,
    desired_amt: BU256,
    from: StateBalance,
}

/// The committed outcome for one side of a trade for a user.
#[derive(Clone, PartialEq, Debug)]
pub struct Commit {
    /// The input order that filled the left side.
    left: OrderCreated,
    /// The input order that filled the right side.
    right: OrderCreated,
    /// A copied order that represents a fulfilled order for the left side.
    left_fulfilled: CreateBalance,
    /// A copied order that represents a fulfilled order for the right side.
    right_fulfilled: CreateBalance,
    /// The excess order, if any, for the left side.
    excess_left: Option<CreateBalance>,
    /// The excess order, if any, for the right side.
    excess_right: Option<CreateBalance>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct JoinBalance {
    left: CreateBalance,
    right: CreateBalance,
}
