
// Creates a spendable Balance UTXO, without the sending of the Permit
// blob to provide the liquidity to create this type.
#[derive(Clone, PartialEq, Debug)]
pub struct CreateBalance {
    owner: BAddress,
    asset: BAddress,
    chain: u32,
    amount: BU256,
}

// Gets translated into from ArgsBalance compared to what we know about
// the user.
pub enum CreateBalanceOrigin {
    /// Smply creating a balance using a Permit signature on-chain!
    Single(CreateBalance),
    /// The right side commitment here is the result of the Commit on the left.
    Commit(Commit, CreateBalance),
}

#[derive(Clone, PartialEq, Debug)]
pub struct SplitBalance {
    /// The input balance.
    input: CreateBalanceOrigin,
    /// The balance that would be spent in a recursive use of this state.
    spendable: CreateBalance,
    /// The excess balance that should be reused later.
    excess: CreateBalance,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StateBalance {
    Split(SplitBalance),
    Single(CreateBalance),
}

#[derive(Clone, PartialEq, Debug)]
pub struct OrderCreated {
    from: StateBalance,
    desired_asset: BAddress,
    desired_chain: u32,
    desired_amt: BU256,
}

/// The committed outcome for one side of a trade for a user.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct Commit {
    /// The input order that filled the left side.
    left: OrderCreated,
    /// The input order that filled the right side.
    right: OrderCreated,
    /// A copied order that represents a fulfilled order for the left side.
    left_fulfilled: OrderCreated,
    /// A copied order that represents a fulfilled order for the right side.
    right_fulfilled: OrderCreated,
    /// The excess order, if any, for the left side.
    excess_left: Option<OrderCreated>,
    /// The excess order, if any, for the right side.
    excess_right: Option<OrderCreated>,
}

/// Simple view of the Commit structure to send recipients of the trade outcomes.
pub struct Outcome {
    asset: BAddress,
    recipient: BAddress,
    amt: BU256,
}

/// The emissions of the trade that took place here, in the form of a view.
/// Aka the inverted form of the request, or just the left side's fulfilled balance
/// flipped around slash the right side.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct Emissions {
    commit: Commit,
    left_outcome: Outcome,
    right_outcome: Outcome,
}
