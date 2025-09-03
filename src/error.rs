use alloc::{vec, vec::Vec};

use borsh::{BorshDeserialize, BorshSerialize};

use stylus_sdk::prelude::calls::errors::Error as StylusErr;

pub use crate::{applicative::ApplicativeLabel, result::Res};

/// ErrorDiscriminant is not shown to users, even if it contains any
/// information. It could have its contents printed while running on the
/// native host.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub enum ErrorDiscriminant {
    /// Bad call was made! It reverted.
    BadCall,

    /// A token was taken that's inconsistent with the user's goal.
    GoalInconsistent,

    /// Goal not met for checking goal amounts.
    GoalNotMet,

    /// The requests array isn't even.
    UnusualRequestsAmount,

    /// Incorrect Applicative transition. To and from.
    BadApplicativeTransition(ApplicativeLabel, ApplicativeLabel),

    /// We were unable to validate a signature.
    InvalidRequest,

    /// The nonce was inconsistent with our local storage of it!
    BadNonce,

    /// The deadline is out of date.
    BadDeadline,

    /// An unpack returned errorneously.
    BadUnpack,

    /// Bad signer of a ecrecover call.
    BadEcrecoverSigner,

    /// Bad keccak call.
    BadKeccakCall,

    /// Bad verifying key creation.
    BadVerifyingKey,

    /// Bad verifying of a signature using strict methods.
    BadStrictVerify,

    /// The signer wasn't found using their id.
    SignerNotFoundId([u8; 4]),

    /// The signer wasn't found using a verifying key.
    SignerNotFoundKey,

    /// Unable to sign a prehashed blob.
    UnableToSignPrehashed,

    /// The convert stage couldn't find the ID given.
    AccountIdNotFound([u8; 4]),

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
    CheckedSub(u128, u128),

    /// Checked add overflow in the math!
    CheckedAdd(u128, u128),

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
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Error {
    pub typ: ErrorDiscriminant,
    pub cd: Vec<u8>,
}

impl Error {
    pub fn is_typ(&self, x: ErrorDiscriminant) -> bool {
        self.typ == x
    }
}

impl From<ErrorDiscriminant> for Error {
    fn from(typ: ErrorDiscriminant) -> Self {
        Error { typ, cd: vec![] }
    }
}

pub type R = Result<Res, Error>;

pub fn err_cd(typ: ErrorDiscriminant, cd: Vec<u8>) -> R {
    Err(Error { typ, cd })
}

pub fn map_stylus_err(
    call_unp: ErrorDiscriminant,
    unpack_unp: ErrorDiscriminant,
    x: StylusErr,
) -> Error {
    match x {
        StylusErr::AbiDecodingFailed(_) => Error {
            typ: unpack_unp,
            cd: vec![],
        },
        StylusErr::Revert(x) => Error {
            typ: call_unp,
            cd: x,
        },
    }
}

impl From<StylusErr> for Error {
    fn from(x: StylusErr) -> Error {
        map_stylus_err(ErrorDiscriminant::BadCall, ErrorDiscriminant::BadUnpack, x)
    }
}

impl From<alloy_sol_types::Error> for Error {
    fn from(_: alloy_sol_types::Error) -> Error {
        // It's likely we're using this for a failed decoding, so that's what
        // we're always assuming.
        Error {
            typ: ErrorDiscriminant::BadUnpack,
            cd: vec![],
        }
    }
}

pub fn err(x: ErrorDiscriminant) -> R {
    err_cd(x, vec![])
}

pub const NOOP: R = Ok(Res::Noop);

pub const DONE_UNIT: R = Ok(Res::DoneUnit);

pub fn DONE_U128(x: u128) -> R {
    Ok(Res::DoneU128(x))
}

#[macro_export]
macro_rules! require {
    ($cond:expr, $err:ident) => {
        if !($cond) {
            err(ErrorDiscriminant::$err)?;
        }
    };
}
