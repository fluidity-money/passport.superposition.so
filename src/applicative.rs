// Applicative form of the state machine, that gets converted to an
// internal representation during the program's simulation.

#[cfg(feature = "std")]
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

use borsh::{BorshDeserialize, BorshSerialize};

use alloc::boxed::Box;

use crate::error::Error;

pub type Address = [u8; 20];

// Concatenated form of the ed25519 r and s values for use with
// ed25519_dalek.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(
    feature = "std",
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary,)
)]
pub struct EdSig(pub [u8; 64]);

#[derive(Clone, Debug, Copy)]
pub enum ErrFromStrApplicative {
    Unknown,
    BadSigDecode,
    BadSigLength,
}

#[cfg(feature = "std")]
impl std::fmt::Display for ErrFromStrApplicative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "std")]
impl serde::ser::StdError for ErrFromStrApplicative {}

#[cfg(feature = "std")]
impl core::str::FromStr for Applicative {
    type Err = ErrFromStrApplicative;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_sexpr::from_str(s).map_err(|_| ErrFromStrApplicative::Unknown)
    }
}

#[cfg(feature = "std")]
impl<'de> serde::de::Deserialize<'de> for EdSig {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(
            const_hex::decode_to_array(&<std::string::String as serde::Deserialize>::deserialize(
                d,
            )?)
            .map_err(serde::de::Error::custom)?
            .into(),
        )
    }
}

#[cfg(feature = "std")]
impl serde::Serialize for EdSig {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&const_hex::encode(&self.0))
    }
}

impl From<EdSig> for [u8; 64] {
    fn from(x: EdSig) -> Self {
        x.0
    }
}

impl From<[u8; 64]> for EdSig {
    fn from(x: [u8; 64]) -> Self {
        EdSig(x)
    }
}

impl<'a> From<&'a EdSig> for &'a [u8] {
    fn from(x: &'a EdSig) -> Self {
        &x.0
    }
}

#[cfg(feature = "std")]
impl core::str::FromStr for EdSig {
    type Err = ErrFromStrApplicative;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let x: [u8; 64] = const_hex::decode(s)
            .map_err(|_| ErrFromStrApplicative::BadSigDecode)?
            .try_into()
            .map_err(|_| ErrFromStrApplicative::BadSigLength)?;
        Ok(x.into())
    }
}

/// User provided signature. Needs a lookup in the accounts table.
pub type UserSig = (u8, EdSig);

// User implemented u128 type for Serde sexp encoding reasons (the
// upstream crate lacks this).
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(
    feature = "std",
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary,)
)]
pub struct U128(pub u128);

#[cfg(feature = "std")]
impl<'de> serde::de::Deserialize<'de> for U128 {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        use std::str::FromStr;
        // This is necessary since serde sexpr doesn't support u128.
        let s = &<std::string::String as serde::Deserialize>::deserialize(d)?;
        Ok(U128(u128::from_str(s).map_err(serde::de::Error::custom)?))
    }
}

#[cfg(feature = "std")]
impl serde::Serialize for U128 {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.0.to_string())
    }
}

/// Solver provided signature.
pub type SolverSig = EdSig;

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(
    feature = "std",
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary,)
)]
pub struct Asset(pub [u8; 20]);

#[cfg(feature = "std")]
impl<'de> serde::de::Deserialize<'de> for Asset {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Asset(
            const_hex::decode_to_array(&<std::string::String as serde::Deserialize>::deserialize(
                d,
            )?)
            .map_err(serde::de::Error::custom)?,
        ))
    }
}

#[cfg(feature = "std")]
impl serde::Serialize for Asset {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&const_hex::encode(&self.0))
    }
}

/// Vault provided signature. The location of the provider is the type
/// of vault that was used here. The signature is screened to check if the
/// vault arguments are consistent with the filling of the liquidity here.
/// The vault will enforce restrictions on the type of asset that was used,
/// including the amount.
pub type VaultSig = EdSig;

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
    feature = "std",
    derive(
        arbitrary::Arbitrary,
        proptest_derive::Arbitrary,
        SerdeDeserialize,
        SerdeSerialize
    )
)]
pub struct ArgsBalance {
    pub asset: Asset,
    pub chain: u64,
    pub amount: U128,
    pub ms_timestamp: U128,
}

/// In the Applicative form, the arguments for the Order are slightly
/// different to also include the amount the user wants to liquidate.
/// Since the Balance should be entirely spent, during the indirection
/// stage to the more fleshed out type, a SplitBalance operation is
/// created.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(
    feature = "std",
    derive(
        arbitrary::Arbitrary,
        proptest_derive::Arbitrary,
        SerdeDeserialize,
        SerdeSerialize
    )
)]
pub struct ArgsOrder {
    /// From amount that the user is willing to consume from the
    /// previous balance on this operation.
    pub from_amt: U128,
    pub desired_asset: Asset,
    pub desired_chain: U128,
    /// Desired amount of the other asset to fill for.
    pub desired_amt: U128,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(
    feature = "std",
    derive(
        arbitrary::Arbitrary,
        proptest_derive::Arbitrary,
        SerdeDeserialize,
        SerdeSerialize
    )
)]
pub struct ArgsCommit {
    pub ms_timestamp: U128,
}

/// Simple label for debugging purposes when a contextual error takes
/// place during a form conversion or validation.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug, Copy)]
#[cfg_attr(
    feature = "std",
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary)
)]
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

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(SerdeDeserialize, SerdeSerialize))]
pub struct DppmNonce;

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(SerdeDeserialize, SerdeSerialize))]
pub struct DppmMintArgs;

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(SerdeDeserialize, SerdeSerialize))]
pub struct DppmBurnArgs;

/// User friendly higher level form of the state_machine internal type
/// that does conversions to the internal type in a way that's more
/// consistent with the user journey. Also consumed by the contract to
/// verify signatures, since this form is easier to interact with.
/// constructed, but the consumer of the Balance opts to spend less than
/// they otherwise could've, to the state machine it's implicitly a split
/// balance operation. The Applicative user must rejoin balances when it
/// suits them, but it's not important for them to split balances.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(SerdeDeserialize, SerdeSerialize))]
pub enum Applicative {
    /// Starting point of the conversion to the other types.
    Balance(UserSig, ArgsBalance),
    /// Withdraw a Balance from the system. The solver signature is needed
    /// alongside the user's signature to be able to testify there are no
    /// unspent UTXOs. The signature only needs to be the signed value
    /// of the concatenation of the hash of the
    /// (Balance|CommitLeftFilledToBalance|CommitRightFilledToBalance|Cancel)
    /// input from the solver and the user.
    Withdraw(SolverSig, UserSig, Option<VaultSig>, Box<Applicative>),
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
            Applicative::Withdraw(_, _, _, _) => ApplicativeLabel::Withdraw,
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

#[cfg(feature = "std")]
impl std::fmt::Display for Applicative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_sexpr::to_string(self).unwrap())
    }
}

/// User friendly trait for construction of Applicative with types
/// included in a side effectful way.
pub trait UserApplicative {
    fn balance(&self, asset: Address, chain: u64, amount: u128, ms_timestamp: u128)
        -> Applicative;

    fn withdraw(
        &self,
        solver_sig: EdSig,
        ap: Applicative,
        vault_sig: Option<EdSig>,
    ) -> Result<Applicative, Error>;

    fn order(
        &self,
        from_amt: u128,
        desired_asset: Address,
        desired_chain: u128,
        desired_amt: u128,
        ap: Applicative,
    ) -> Result<Applicative, Error>;

    fn cancel(&self, solver_sig: EdSig, ap: Applicative) -> Result<Applicative, Error>;

    fn commit_left_filled_to_balance(&self, ap: Applicative) -> Result<Applicative, Error>;

    fn commit_right_filled_to_balance(&self, ap: Applicative) -> Result<Applicative, Error>;

    fn commit_left_excess_to_order(&self, ap: Applicative) -> Result<Applicative, Error>;

    fn commit_right_excess_to_order(&self, ap: Applicative) -> Result<Applicative, Error>;

    fn join(&self, left: Applicative, right: Applicative) -> Result<Applicative, Error>;
}

pub trait SolverApplicative {
    fn withdraw(&self, ap: &Applicative) -> Result<SolverSig, Error>;

    fn cancel(&self, ap: &Applicative) -> Result<SolverSig, Error>;

    fn commit(
        &self,
        ms_timestamp: u128,
        left: &Applicative,
        right: &Applicative,
    ) -> Result<SolverSig, Error>;
}
