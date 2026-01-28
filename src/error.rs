use borsh::{BorshDeserialize, BorshSerialize};

pub use crate::{applicative::ApplicativeLabel, result::Res};

use bobcat_sdk::maths::U;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use alloc::{string::String, vec::Vec};

#[cfg(feature = "errors-extra-context")]
use alloc::boxed::Box;

#[cfg(feature = "std")]
use proptest::strategy::Strategy;

type Address = [u8; 20];

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
#[derive(
    Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive, BorshSerialize, BorshDeserialize,
)]
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

    /// Zero amount in the balance object.
    ZeroBalanceAmount,

    SameAssets,

    /// A pair of assets were matched that weren't consistent with the desired amounts.
    BadAssetAsks,

    Erc20Invoke,

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

    /// There wasn't enough liquidity for the from ask.
    NotEnoughFromAmount,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct ErrorInner {
    pub context: Option<ApplyContext>,
    pub x: Option<U>,
    pub y: Option<U>,
    pub app: Option<ApplicativeLabel>,
    pub side: Option<u8>,
    pub app_to: Option<ApplicativeLabel>,
    pub asset_left: Option<String>,
    pub asset_right: Option<String>,
    pub desired_left: Option<String>,
    pub desired_right: Option<String>,
}

impl Default for ErrorInner {
    fn default() -> Self {
        Self {
            context: None,
            x: None,
            y: None,
            app: None,
            side: None,
            app_to: None,
            asset_left: None,
            asset_right: None,
            desired_left: None,
            desired_right: None,
        }
    }
}

#[derive(PartialEq, Clone, BorshSerialize, BorshDeserialize)]
pub struct Error {
    pub typ: ErrorDiscriminant,
    pub hash: Option<[u8; 64]>,
    #[cfg(feature = "errors-extra-context")]
    pub inner: Box<ErrorInner>,
}

impl From<Error> for Vec<u8> {
    fn from(x: Error) -> Self {
        borsh::to_vec(&x).unwrap()
    }
}

impl TryFrom<u8> for Error {
    type Error = u8;

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        Ok(Self {
            typ: ErrorDiscriminant::try_from(x).map_err(|_| x)?,
            ..Error::default()
        })
    }
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
            #[cfg(feature = "errors-extra-context")]
            inner: Box::new(ErrorInner {
                context: None,
                x: None,
                y: None,
                app: None,
                side: None,
                app_to: None,
                hash: None,
                asset_left: None,
                asset_right: None,
                desired_left: None,
                desired_right: None,
            }),
        })
    }
}

#[cfg(feature = "std")]
impl proptest::prelude::Arbitrary for Error {
    type Parameters = ();
    type Strategy = proptest::prelude::BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        proptest::prelude::any::<ErrorDiscriminant>()
            .prop_map(|typ| Error::from(typ))
            .boxed()
    }
}

impl Error {
    #[allow(unused_mut)]
    pub fn ctx(mut self, _c: ApplyContext) -> Self {
        #[cfg(feature = "errors-extra-context")]
        {
            self.inner.context = Some(_c);
        }
        self
    }

    #[allow(unused_mut)]
    pub fn x(mut self, _x: U) -> Self {
        #[cfg(feature = "errors-extra-context")]
        {
            self.inner.x = Some(_x);
        }
        self
    }

    #[allow(unused_mut)]
    pub fn y(mut self, _y: U) -> Self {
        #[cfg(feature = "errors-extra-context")]
        {
            self.inner.y = Some(_y);
        }
        self
    }

    #[allow(unused_mut)]
    pub fn app(mut self, _from: ApplicativeLabel) -> Self {
        #[cfg(feature = "errors-extra-context")]
        {
            self.inner.app = Some(_from);
        }
        self
    }

    #[allow(unused_mut)]
    pub fn side(mut self, _side: u8) -> Self {
        #[cfg(feature = "errors-extra-context")]
        {
            self.inner.side = Some(_side);
        }
        self
    }

    #[allow(unused_mut)]
    pub fn app_to(mut self, _to: ApplicativeLabel) -> Self {
        #[cfg(feature = "errors-extra-context")]
        {
            self.inner.app_to = Some(_to);
        }
        self
    }

    #[allow(unused_mut)]
    pub fn hash(mut self, _h: [u8; 64]) -> Self {
        #[cfg(feature = "errors-extra-context")]
        {
            self.inner.hash = Some(_h);
        }
        self
    }

    #[allow(unused_mut)]
    pub fn asset_left(mut self, _a: Address) -> Self {
        #[cfg(feature = "errors-extra-context")]
        {
            self.inner.asset_left = Some(const_hex::encode(_a));
        }
        self
    }

    #[allow(unused_mut)]
    pub fn asset_right(mut self, _a: Address) -> Self {
        #[cfg(feature = "errors-extra-context")]
        {
            self.inner.asset_right = Some(const_hex::encode(_a));
        }
        self
    }

    #[allow(unused_mut)]
    pub fn desired_left(mut self, _a: Address) -> Self {
        #[cfg(feature = "errors-extra-context")]
        {
            self.inner.desired_left = Some(const_hex::encode(_a));
        }
        self
    }

    #[allow(unused_mut)]
    pub fn desired_right(mut self, _a: Address) -> Self {
        #[cfg(feature = "errors-extra-context")]
        {
            self.inner.desired_right = Some(const_hex::encode(_a));
        }
        self
    }
}

impl Default for Error {
    fn default() -> Self {
        Error {
            typ: ErrorDiscriminant::Unknown,
            #[cfg(feature = "errors-extra-context")]
            inner: Box::new(ErrorInner::default()),
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
        #[cfg(feature = "errors-extra-context")]
        d.field("inner", &self.inner);
        d.finish()
    }
}

pub type R = Result<Res, Error>;

pub fn err_cd(typ: ErrorDiscriminant) -> R {
    Err(Error::from(typ))
}

impl From<ErrorDiscriminant> for Error {
    fn from(x: ErrorDiscriminant) -> Self {
        Error {
            typ: x,
            ..Error::default()
        }
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
