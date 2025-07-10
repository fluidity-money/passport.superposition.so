// Applicative form of the state machine, that gets converted to an
// internal representation during the program's simulation.

use stylus_sdk::alloy_primitives::{Address, U256};

use borsh::{BorshDeserialize, BorshSerialize};

use alloc::boxed::Box;

use crate::{encoding::*, error::Error};

// Concatenated form of the ed25519 r and s values for use with
// ed25519_dalek.
pub type EdSig = [u8; 64];

/// Ed25519 signature. Referenced according to its place in the accounts
/// vector.
pub type EdId = usize;

/// User provided signature. Needs a lookup in the accounts table.
pub type UserSig = (EdId, EdSig);

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
    pub asset: BAddress,
    pub chain: u128,
    pub amount: BU256,
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
    pub from_amt: BU256,
    pub desired_asset: BAddress,
    pub desired_chain: u128,
    pub desired_amt: BU256,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary)
)]
pub struct ArgsCommit {
    pub ms_timestamp: u128,
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
    /// When this step is used, it's only admissable if the user has
    /// uncommitted amounts they've deposited that haven't been converted to a
    /// Balance.
    Balance(UserSig, ArgsBalance),
    /// Withdraw a Balance from the system. The solver signature is needed
    /// alongside the user's signature to be able to testify there are no
    /// unspent UTXOs. The signature only needs to be the signed value
    /// of the concatenation of the hash of the
    /// (Balance|CommitLeftFilledToBalance|CommitRightFilledToBalance)
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
    Join(UserSig, Box<Applicative>, Box<Applicative>),
}

/// User friendly trait for construction of Applicative with types
/// included in a side effectful way.
pub trait UserApplicative {
    fn balance(&self, asset: Address, chain: u128, amount: U256, ms_timestamp: u128)
        -> Applicative;

    fn withdraw(&self, solver_sig: [u8; 64], ap: Applicative) -> Result<Applicative, Error>;

    fn order(
        &self,
        from_amt: U256,
        desired_asset: Address,
        desired_chain: u128,
        desired_amt: U256,
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

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod test_proptest {
    use proptest::prelude::*;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestBalanceInside {
        args: ArgsBalance,
    }

    // Flattened testing structure that if the excess form is used, assume
    // that the input is a Commit properly structured.
    #[derive(Debug, PartialEq)]
    enum TestOrder {
        Balance(Box<TestBalance>),
        CommitLeftExcessToOrder(Box<TestCommit>),
        CommitRightExcessToOrder(Box<TestCommit>),
    }

    #[derive(Debug, PartialEq)]
    struct TestCommitInside {
        args: ArgsCommit,
        left: Box<TestOrder>,
        right: Box<TestOrder>,
    }

    #[derive(Debug, PartialEq)]
    enum TestCommit {
        Commit(Box<TestCommitInside>),
    }

    #[derive(Debug, PartialEq)]
    enum TestBalance {
        Balance(TestBalanceInside),
        CommitLeftFilledToBalance(Box<TestCommit>),
        CommitRightFilledToBalance(Box<TestCommit>),
    }

    #[derive(Debug, PartialEq)]
    enum Entry {
        Balance(TestBalance),
        Withdraw(TestBalance),
        Order(TestOrder),
        Cancel(TestOrder),
        Commit(TestCommit),
        CommitLeftFilledToBalance(TestCommit),
        CommitRightFilledToBalance(TestCommit),
        CommitLeftExcessToOrder(TestCommit),
        CommitRightExcessToOrder(TestCommit),
    }

    // ChatGPT generated arbitrary type in lieu of using the generator.
    impl Arbitrary for Entry {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            let bal_leaf = any::<ArgsBalance>()
                .prop_map(|args| TestBalance::Balance(TestBalanceInside { args }))
                .boxed();

            let ord_leaf = any::<ArgsBalance>()
                .prop_map(|args| {
                    TestOrder::Balance(Box::new(TestBalance::Balance(TestBalanceInside { args })))
                })
                .boxed();
            let commit_leaf = (any::<ArgsCommit>(), ord_leaf.clone(), ord_leaf.clone())
                .prop_map(|(args, left, right)| {
                    TestCommit::Commit(Box::new(TestCommitInside {
                        args,
                        left: Box::new(left),
                        right: Box::new(right),
                    }))
                })
                .boxed();
            let commit_strat = commit_leaf.prop_recursive(4, 64, 4, |inner| {
                (any::<ArgsCommit>(), inner.clone(), inner)
                    .prop_map(|(args, left_c, right_c)| {
                        TestCommit::Commit(Box::new(TestCommitInside {
                            args,
                            left: Box::new(TestOrder::CommitLeftExcessToOrder(Box::new(left_c))),
                            right: Box::new(TestOrder::CommitRightExcessToOrder(Box::new(right_c))),
                        }))
                    })
                    .boxed()
            });
            let c_ord_l = commit_strat.clone();
            let c_ord_r = commit_strat.clone();
            let ord_strat = ord_leaf.prop_recursive(4, 64, 4, move |inner| {
                prop_oneof![
                    c_ord_l
                        .clone()
                        .prop_map(|c| TestOrder::CommitLeftExcessToOrder(Box::new(c))),
                    c_ord_r
                        .clone()
                        .prop_map(|c| TestOrder::CommitRightExcessToOrder(Box::new(c))),
                    inner,
                ]
                .boxed()
            });
            let c_bal = commit_strat.clone();
            let bal_strat = bal_leaf.prop_recursive(4, 64, 4, move |inner| {
                prop_oneof![
                    c_bal
                        .clone()
                        .prop_map(|c| TestBalance::CommitLeftFilledToBalance(Box::new(c))),
                    c_bal
                        .clone()
                        .prop_map(|c| TestBalance::CommitRightFilledToBalance(Box::new(c))),
                    inner,
                ]
                .boxed()
            });
            prop_oneof![
                bal_strat.clone().prop_map(Entry::Balance),
                bal_strat.clone().prop_map(Entry::Withdraw),
                ord_strat.clone().prop_map(Entry::Order),
                ord_strat.clone().prop_map(Entry::Cancel),
                commit_strat.clone().prop_map(Entry::Commit),
                commit_strat
                    .clone()
                    .prop_map(Entry::CommitLeftFilledToBalance),
                commit_strat
                    .clone()
                    .prop_map(Entry::CommitRightFilledToBalance),
                commit_strat
                    .clone()
                    .prop_map(Entry::CommitLeftExcessToOrder),
                commit_strat.prop_map(Entry::CommitRightExcessToOrder),
            ]
            .boxed()
        }
    }

    // Convert an Entry to the Applicative form, setting signatures and ids to 0.
    fn entry_to_applicative(e: Entry) -> Applicative {
    }
}
