use stylus_sdk::alloy_primitives::*;

// Colour is the previous source of the Bucket item, so that the storage
// for balances may be arranged correctly.
#[derive(Clone, PartialEq, Debug)]
pub enum Colour {
    /// In the process of compressing the calldata, this Bucket lost its
    /// identity after being provided on-chain.
    Identityless,
    /// The amount in this step of the pool of operations comes from a
    /// balance that was created as an interim step.
    Ephereal,
    /// The amount in this step of the operation came from another user's
    /// order.
    OtherUser,
    /// The amount comes from the spending user's unfilled side of a commit.
    Unfilled,
    /// The amount comes from the user's order that was cancelled.
    Order,
}

/// Creates a spendable UTXO. The conversion does the checking for the
/// validity of the state transformation locally. This type explicitly
/// states the conversion of the balances until their final destination,
/// with an extra field designating where the funds must come from. The
/// convert stage validates that the user has enough funds for each step
/// of the bucket transformation. The apply stage makes the final SSTORE
/// allocations to amounts in each bucket that we track. This step also
/// validates that the counterparty in a trade themeslves have enough
/// balance for their trade, so that it's safe to designate the source of
/// the results of a trade as being from the Ephereal pool.
#[derive(Clone, PartialEq, Debug)]
pub struct Bucket {
    /// The type of "Colour" this bucket is, aka, where the funds come from.
    pub colour: Colour,
    /// Where this bucket came from. We traverse this tree to find the amounts
    /// to emit, and to validate that the user has enough for their deposit during
    /// the apply stage.
    pub from: Option<Box<Bucket>>,
    /// The amount "Spent" by this bucket for the next step to consume.
    pub spent: u128,
    /// The amount "Saved" into the storage from the bucket that can't be
    /// consumed. The idea is that this was inadvertendly saved, by use of the
    /// spend of an amount converted, or not. Note that a Withdrawal still
    /// needs to consume the amount and will record itself as a spent amount.
    /// The point of saved is to prevent accidental respending of amounts that
    /// were committed partially on-chain when someone has submitted calldata.
    /// During a situation where a Commit has taken place, we can't store that
    /// someone has recorded a saved amount here unless we're explicitly accessing
    /// it since our bucket lacks the identity in these situations. When this happens,
    /// we record the saved amount as zero.
    pub saved: u128,
    /// The timestamp, if any, that this operation originated from. This
    /// might be used validate if an amount at a state has the funds to service
    /// an operation.
    pub ms_ts: u128,
    /// The destination chain of these funds for reconciliation at the final
    /// step of the operation. TODO.
    pub chain: u128,
    /// The asset that this bucket is for.
    pub asset: Address,
    /// The end owner of the saved funds in this bucket.
    pub owner: Address,
}
