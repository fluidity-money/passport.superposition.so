use borsh::{BorshDeserialize, BorshSerialize};

pub use crate::{applicative::ApplicativeLabel, result::Res};

use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256},
    prelude::calls::errors::Error as StylusErr,
};

use alloc::{boxed::Box, vec::Vec};

#[cfg(feature = "std")]
use proptest::strategy::Strategy;

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, PartialEq, Debug)]
#[cfg_attr(
    feature = "std",
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary)
)]
pub enum ApplyContext {
    JoinLBalAmt,
    JoinRBalAmt,
    ApplyBalanceInline,
    ApplyBalanceCancel,
    ApplyCommit,
    ApplyCommitLeftAmtFilled,
    ApplyCommitRightAmtFilled,
    ApplyCommitBalanceAmount,
    ApplyOrder,
    ApplyWithdraw,
    AddLiq,
    Onboard,
}

/// ErrorDiscriminant is not shown to users, even if it contains any
/// information. It could have its contents printed while running on the
/// native host.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(
    feature = "std",
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary)
)]
#[repr(u8)]
pub enum ErrorDiscriminant {
    /// The error wasn't created properly.
    Unknown,

    /// The chain id during onboarding is different from ours.
    OnboardDifferentChainId,

    /// Different contract in use for onboarding.
    OnboardDifferentContract,

    /// Generic call error that we translated directly.
    BadCall,

    /// A token was taken that's inconsistent with the user's goal.
    GoalInconsistent,

    /// Goal not met for checking goal amounts.
    GoalNotMet,

    /// Incorrect Applicative transition during digesting. To and from.
    BadApplicativeTransitionDigest,

    /// Incorrect Applicative transition during validation. To and from.
    BadApplicativeTransitionValidate,

    /// The nonce was inconsistent with our local storage of it!
    BadNonce,

    /// The deadline is out of date.
    BadDeadline,

    /// An unpack returned errorneously.
    BadUnpack,

    /// Bad signer of a ecrecover call.
    BadEcrecoverSigner,

    /// Bad verifying key creation.
    BadVerifyingKey,

    /// Bad creation of a signature from what's in the applicative structure.
    BadSignatureCreation,

    /// Bad verifying of a signature using strict methods.
    BadStrictVerify,

    /// The signer wasn't found using their id.
    SignerNotFoundId,

    /// The signer wasn't found using a verifying key.
    SignerNotFoundKey,

    /// The convert stage couldn't find the ID given.
    AccountIdNotFound,

    /// The convert stage couldn't find the address.
    AccountKeyNotFound,

    /// A bad conversion from took place from the applicative form to the
    /// state machine form.
    BadConversionFrom,

    /// One side of the commit was asking for a side that doesn't match up
    /// with the other side.
    BadAssetComparison,

    /// Inconsistent owners for a multiple step operation.
    InconsistentOwners,

    /// Not enough from the derivative amount to fill.
    NotEnoughForDeriv,

    /// No left excess is available to the user from this commit!
    NoLeftExcess,

    /// Checked sub overflow in the math!
    CheckedSub,

    /// Checked add overflow in the math!
    CheckedAdd,

    /// Checked add overflow for a 64 bit somewhere. Not including 64 bit here saves us the
    /// encoding codesize.
    CheckedAdd64,

    /// Zero amount in the balance object.
    ZeroBalanceAmount,

    SameAssets,

    BadAssetAsks,

    Erc20TransferFromCall,

    Erc20TransferFromDecode,

    Erc20TransferFromFalse,

    Erc20BalanceOfCall,

    Erc20BalanceOfDecode,

    Erc20TransferCall,

    Erc20TransferDecode,

    Erc20TransferFalse,

    /// This happens if the amount that the user asked to transition from their balance to their
    /// order is incorrect during the Order stage.
    BalanceTransitionToOrderBad,

    /// The hash was already seen onchain!
    HashAlreadyOnchain,

    /// The token has no code!
    TokenNoCode,

    /// The signature given during onboarding was bad!
    BadOnboardingSig,

    /// It wasn't possible to verify a signature during a sig_two validate.
    BadStrictVerifyTwo,

    /// During testing, there wasn't enough balance for a transfer!
    TestNotEnoughBalForTransfer,

    /// During testing, there wasn't enough allowance!
    TestNotEnoughAllowance,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorTestContext {
    pub sender: Address,
    pub recipient: Address,
    pub asset: Address,
    pub amt: U256,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorTestInterimDetails {
    pub owner_l: Address,
    pub owner_r: Address,
    pub asset_l: Address,
    pub asset_r: Address,
    pub amt: u128,
    pub hash: FixedBytes<32>,
    pub thread_recorded_owner: Address,
    pub thread_recorded_asset: Address,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorInterimAccessContext {
    pub accessed_hash: FixedBytes<32>,
    pub interim_hashes: Vec<ErrorTestInterimDetails>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorInner {
    // Used to hint information about the app when an error happens if this
    // is tagged on.
    pub test_context: Option<ErrorTestContext>,
    pub test_interim: Option<ErrorInterimAccessContext>,
    pub context: Option<ApplyContext>,
    pub x: Option<u128>,
    pub y: Option<u128>,
    pub app: Option<ApplicativeLabel>,
    pub side: Option<u8>,
    pub app_to: Option<ApplicativeLabel>,
    pub hash: Option<[u8; 64]>,
}

#[derive(PartialEq)]
pub struct Error {
    pub typ: ErrorDiscriminant,
    pub inner: Box<ErrorInner>
}

impl Error {
    pub fn dis_u8(self) -> u8 {
        self.typ as u8
    }
}

#[cfg(feature = "std")]
impl<'a> arbitrary::Arbitrary<'a> for Error {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Error {
            typ: ErrorDiscriminant::arbitrary(u)?,
            test_context: None,
            test_interim: None,
            context: None,
            x: None,
            y: None,
            app: None,
            side: None,
            app_to: None,
            hash: None,
        })
    }
}

#[cfg(feature = "std")]
impl proptest::prelude::Arbitrary for Error {
    type Parameters = ();
    type Strategy = proptest::prelude::BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        proptest::prelude::any::<ErrorDiscriminant>()
            .prop_map(|typ| Error {
                typ,
                test_context: None,
                test_interim: None,
                context: None,
                x: None,
                y: None,
                app: None,
                side: None,
                app_to: None,
                hash: None,
            })
            .boxed()
    }
}

impl Error {
    pub fn test_context(mut self, e: ErrorTestContext) -> Self {
        self.inner.test_context = Some(e);
        self
    }

    pub fn test_interim(mut self, v: ErrorInterimAccessContext) -> Self {
        self.inner.test_interim = Some(v);
        self
    }

    pub fn ctx(mut self, c: ApplyContext) -> Self {
        self.inner.context = Some(c);
        self
    }

    pub fn x(mut self, x: u128) -> Self {
        self.inner.x = Some(x);
        self
    }

    pub fn y(mut self, y: u128) -> Self {
        self.inner.y = Some(y);
        self
    }

    pub fn app(mut self, from: ApplicativeLabel) -> Self {
        self.inner.app = Some(from);
        self
    }

    pub fn side(mut self, side: u8) -> Self {
        self.inner.side = Some(side);
        self
    }

    pub fn app_to(mut self, to: ApplicativeLabel) -> Self {
        self.inner.app_to = Some(to);
        self
    }

    pub fn hash(mut self, h: [u8; 64]) -> Self {
        self.inner.hash = Some(h);
        self
    }
}

impl Default for Error {
    fn default() -> Self {
        Error {
            typ: ErrorDiscriminant::Unknown,
            inner: Box::new(ErrorInner {
                test_context: None,
                test_interim: None,
                context: None,
                x: None,
                y: None,
                app: None,
                side: None,
                app_to: None,
                hash: None,
            }),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Error");
        d.field("typ", &self.typ);
        d.field("inner", &self.inner);
        d.finish()
    }
}

impl From<StylusErr> for Error {
    fn from(x: StylusErr) -> Error {
        map_stylus_err(ErrorDiscriminant::BadCall, ErrorDiscriminant::BadUnpack, x)
    }
}

pub type R = Result<Res, Error>;

pub fn err_cd(typ: ErrorDiscriminant) -> R {
    Err(Error::from(typ))
}

pub fn map_stylus_err(
    call_unp: ErrorDiscriminant,
    unpack_unp: ErrorDiscriminant,
    x: StylusErr,
) -> Error {
    match x {
        StylusErr::AbiDecodingFailed(_) => Error::from(unpack_unp),
        StylusErr::Revert(_) => Error::from(call_unp),
    }
}

impl From<ErrorDiscriminant> for Error {
    fn from(x: ErrorDiscriminant) -> Self {
        Error {
            typ: x,
            ..Error::default()
        }
    }
}

impl From<alloy_sol_types::Error> for Error {
    fn from(_: alloy_sol_types::Error) -> Self {
        // It's likely we're using this for a failed decoding, so that's what
        // we're always assuming.
        Error::from(ErrorDiscriminant::BadUnpack)
    }
}

pub fn err(x: ErrorDiscriminant) -> R {
    err_cd(x)
}

pub const NOOP: R = Ok(Res::Noop);

pub const DONE_UNIT: R = Ok(Res::DoneUnit);

pub fn done_u64(x: u64) -> R {
    Ok(Res::DoneU64(x))
}

#[macro_export]
macro_rules! require {
    ($cond:expr, $err:ident) => {
        if !($cond) {
            err(ErrorDiscriminant::$err)?;
        }
    };
}

#[cfg(feature = "std")]
mod test {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_encoding_decoding_errs(err in any::<super::Error>()) {
            assert_eq!(err, borsh::from_slice(&borsh::to_vec(&err).unwrap()).unwrap());
        }
    }
}
