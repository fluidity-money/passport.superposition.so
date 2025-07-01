use alloc::{vec, vec::Vec};

use borsh::{BorshDeserialize, BorshSerialize};

use stylus_sdk::prelude::calls::errors::Error as StylusErr;

pub use crate::result::Res;

#[derive(BorshSerialize, BorshDeserialize, PartialEq)]
pub enum ErrorDiscriminant {
    /// Bad call was made! It reverted.
    BadCall,

    /// A token was taken that's inconsistent with the user's goal.
    GoalInconsistent,

    /// Goal not met for checking goal amounts.
    GoalNotMet,

    /// The requests array isn't even.
    UnusualRequestsAmount,

    /// Incorrect Applicative transition.
    BadApplicativeTransition,

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
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Error {
    pub typ: ErrorDiscriminant,
    pub cd: Vec<u8>,
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

pub const NOTHING: R = Ok(Res::NOTHING);

pub const DONE: R = Ok(Res::DONE);

#[macro_export]
macro_rules! require {
    ($cond:expr, $err:ident) => {
        if !($cond) {
            err(ErrorDiscriminant::$err)?;
        }
    };
}
