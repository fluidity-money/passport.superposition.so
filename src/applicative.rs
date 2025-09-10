// Applicative form of the state machine, that gets converted to an
// internal representation during the program's simulation.

use stylus_sdk::alloy_primitives::Address;

use borsh::{BorshDeserialize, BorshSerialize};

use alloc::boxed::Box;

use crate::error::Error;

// Concatenated form of the ed25519 r and s values for use with
// ed25519_dalek.
pub type EdSig = [u8; 64];

/// User provided signature. Needs a lookup in the accounts table.
pub type UserSig = (u8, EdSig);

/// Solver provided signature.
pub type SolverSig = EdSig;

/// For situations where the conversion between the type lacks an
/// argument, we include this in the supplied bytes to differentiate
/// things so a signature can't be misused. This is like the state machine
/// equivalent. It's not strictly needed to have a type for the left and right
/// applicative forms since the signer is going to be different for both
/// sides in a normal operation.
#[repr(u8)]
pub enum Nonce {
    Withdraw,
    Cancel,
    Join,
    CommitLeftFilledToBalance,
    CommitRightFilledToBalance,
    CommitLeftExcessToOrder,
    CommitRightExcessToOrder,
}

impl From<Nonce> for u8 {
    fn from(v: Nonce) -> Self {
        unsafe { *<*const _>::from(&v).cast::<u8>() }
    }
}

/// Balance should be the amount that the user has uncommitted in
/// their entirety.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary)
)]
pub struct ArgsBalance {
    pub asset: [u8; 20],
    pub chain: u128,
    pub amount: u128,
    // Owner and timestamp (milliseconds) are combined to create a snowflake.
    pub ms_timestamp: u128,
}

/// In the Applicative form, the arguments for the Order are slightly
/// different to also include the amount the user wants to liquidate.
/// Since the Balance should be entirely spent, during the indirection
/// stage to the more fleshed out type, a SplitBalance operation is
/// created.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary)
)]
pub struct ArgsOrder {
    /// From amount that the user is willing to consume from the
    /// previous balance on this operation.
    pub from_amt: u128,
    pub desired_asset: [u8; 20],
    pub desired_chain: u128,
    /// Desired amount of the other asset to fill for.
    pub desired_amt: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary)
)]
pub struct ArgsCommit {
    pub ms_timestamp: u128,
}

/// Simple label for debugging purposes when a contextual error takes
/// place during a form conversion or validation.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug, Copy)]
pub enum ApplicativeLabel {
    Balance,
    Withdraw,
    Order,
    Cancel,
    Commit,
    CommitLeftFilledToBalance,
    CommitRightFilledToBalance,
    CommitLeftExcessToOrder,
    CommitRightExcessToOrder,
    Join,
}

/// User friendly higher level form of the state_machine internal type
/// that does conversions to the internal type in a way that's more
/// consistent with the user journey. Also consumed by the contract to
/// verify signatures, since this form is easier to interact with.
/// constructed, but the consumer of the Balance opts to spend less than
/// they otherwise could've, to the state machine it's implicitly a split
/// balance operation. The Applicative user must rejoin balances when it
/// suits them, but it's not important for them to split balances.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub enum Applicative {
    /// Starting point of the conversion to the other types.
    Balance(UserSig, ArgsBalance),
    /// Withdraw a Balance from the system. The solver signature is needed
    /// alongside the user's signature to be able to testify there are no
    /// unspent UTXOs. The signature only needs to be the signed value
    /// of the concatenation of the hash of the
    /// (Balance|CommitLeftFilledToBalance|CommitRightFilledToBalance|Cancel)
    /// input from the solver and the user.
    Withdraw(SolverSig, UserSig, Box<Applicative>),
    /// Only (Balance | CommitToBalance*) => Order as the argument here.
    /// Consumes a Balance. The signature is the concatenation of the inputs,
    /// and the hash from the balance.
    /// (Balance|CommitLeftFilledToBalance|CommitRightFilledToBalance|Join)
    Order(UserSig, ArgsOrder, Box<Applicative>),
    /// Cancel an order using a user's signature as well as the matching
    /// engine's signature.
    Cancel(SolverSig, UserSig, Box<Applicative>),
    /// When this type is used, the internal balances are converted to the
    /// derived type that we use to generate the rebalancing of amounts to users.
    /// The first signature argument is the user's signature, and the second is the
    /// matching engine's signature.
    // Args * Order * Order => Commit
    Commit(SolverSig, ArgsCommit, Box<Applicative>, Box<Applicative>),
    // Convert the left side of a Commit to a balance, to be reused.
    CommitLeftFilledToBalance(Box<Applicative>),
    // Commit the right side of the Commit results to a balance.
    CommitRightFilledToBalance(Box<Applicative>),
    // Accessor for the excess left side order to a balance.
    CommitLeftExcessToOrder(Box<Applicative>),
    // Accessor for the excess right side order to a balance.
    CommitRightExcessToOrder(Box<Applicative>),
    /// Join two balances together.
    /// (Balance|Cancel|CommitLeftFilledToBalance|CommitRightFilledToBalance).
    Join(UserSig, Box<Applicative>, Box<Applicative>),
}

impl From<&Applicative> for ApplicativeLabel {
    fn from(x: &Applicative) -> Self {
        match x {
            Applicative::Balance(_, _) => ApplicativeLabel::Balance,
            Applicative::Withdraw(_, _, _) => ApplicativeLabel::Withdraw,
            Applicative::Order(_, _, _) => ApplicativeLabel::Order,
            Applicative::Cancel(_, _, _) => ApplicativeLabel::Cancel,
            Applicative::Commit(_, _, _, _) => ApplicativeLabel::Commit,
            Applicative::CommitLeftFilledToBalance(_) => {
                ApplicativeLabel::CommitLeftFilledToBalance
            }
            Applicative::CommitRightFilledToBalance(_) => {
                ApplicativeLabel::CommitRightFilledToBalance
            }
            Applicative::CommitLeftExcessToOrder(_) => ApplicativeLabel::CommitLeftExcessToOrder,
            Applicative::CommitRightExcessToOrder(_) => ApplicativeLabel::CommitRightExcessToOrder,
            Applicative::Join(_, _, _) => ApplicativeLabel::Join,
        }
    }
}

/// User friendly trait for construction of Applicative with types
/// included in a side effectful way.
pub trait UserApplicative {
    fn balance(&self, asset: Address, chain: u128, amount: u128, ms_timestamp: u128)
        -> Applicative;

    fn withdraw(&self, solver_sig: [u8; 64], ap: Applicative) -> Result<Applicative, Error>;

    fn order(
        &self,
        from_amt: u128,
        desired_asset: Address,
        desired_chain: u128,
        desired_amt: u128,
        ap: Applicative,
    ) -> Result<Applicative, Error>;

    fn cancel(&self, solver_sig: [u8; 64], ap: Applicative) -> Result<Applicative, Error>;

    fn commit(
        &self,
        solver_sig: [u8; 64],
        ms_timestamp: u128,
        left: Applicative,
        right: Applicative,
    ) -> Result<Applicative, Error>;

    fn commit_left_filled_to_balance(&self, ap: Applicative) -> Result<Applicative, Error>;

    fn commit_right_filled_to_balance(&self, ap: Applicative) -> Result<Applicative, Error>;

    fn commit_left_excess_to_order(&self, ap: Applicative) -> Result<Applicative, Error>;

    fn commit_right_excess_to_order(&self, ap: Applicative) -> Result<Applicative, Error>;

    fn join(&self, left: Applicative, right: Applicative) -> Result<Applicative, Error>;
}

pub trait SolverApplicative {
    fn withdraw(&self, ap: Applicative) -> Result<SolverSig, Error>;

    fn cancel(&self, ap: Applicative) -> Result<SolverSig, Error>;

    fn commit(
        &self,
        ms_timestamp: u128,
        left: Applicative,
        right: Applicative,
    ) -> Result<SolverSig, Error>;
}
