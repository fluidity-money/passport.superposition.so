use borsh::{BorshDeserialize, BorshSerialize};

use stylus_sdk::prelude::calls::errors::Error as StylusErr;

pub use crate::{applicative::ApplicativeLabel, result::Res};

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub enum MathContext {
    ApplyCommitLeftAmtFilled,
    ApplyCommitRightAmtFilled,
    IncreaseInterim,
    DecreaseInterim,
    IncreaseWithdrawal,
    DecreaseWithdrawal,
    IncreaseOrder,
    DecreaseOrder,
    ApplyCommitBalanceAmount,
    ApplyCommitLeftAmtUnfilled,
    ApplyCommitRightAmtUnfilled,
    AddLiq,
    Onboard,
}

/// ErrorDiscriminant is not shown to users, even if it contains any
/// information. It could have its contents printed while running on the
/// native host.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub enum ErrorDiscriminant {
    /// Generic call error that we translated directly.
    BadCall,

    /// A token was taken that's inconsistent with the user's goal.
    GoalInconsistent,

    /// Goal not met for checking goal amounts.
    GoalNotMet,

    /// Incorrect Applicative transition. To and from.
    BadApplicativeTransition,

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
    BadSignatureCreation(ApplicativeLabel),

    /// Bad verifying of a signature using strict methods.
    BadStrictVerify(ApplicativeLabel),

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
    CheckedSub(MathContext),

    /// Checked add overflow in the math!
    CheckedAdd(MathContext),

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
    HashAlreadyOnchain([u8; 64]),

    /// The token has no code!
    TokenNoCode,

    /// The signature given during onboarding was bad!
    BadOnboardingSig,

    /// It wasn't possible to verify a signature during a sig_two validate.
    BadStrictVerifyTwo(ApplicativeLabel, u8)
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Error {
    pub typ: ErrorDiscriminant,
}

impl From<StylusErr> for Error {
    fn from(x: StylusErr) -> Error {
        map_stylus_err(ErrorDiscriminant::BadCall, ErrorDiscriminant::BadUnpack, x)
    }
}

pub type R = Result<Res, Error>;

pub fn err_cd(typ: ErrorDiscriminant) -> R {
    Err(Error { typ })
}

pub fn map_stylus_err(
    call_unp: ErrorDiscriminant,
    unpack_unp: ErrorDiscriminant,
    x: StylusErr,
) -> Error {
    match x {
        StylusErr::AbiDecodingFailed(_) => Error { typ: unpack_unp },
        StylusErr::Revert(_) => Error { typ: call_unp },
    }
}

impl From<alloy_sol_types::Error> for Error {
    fn from(_: alloy_sol_types::Error) -> Error {
        // It's likely we're using this for a failed decoding, so that's what
        // we're always assuming.
        Error {
            typ: ErrorDiscriminant::BadUnpack,
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
