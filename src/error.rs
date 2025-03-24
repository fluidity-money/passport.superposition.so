use alloc::{vec, vec::Vec};

use borsh::{BorshDeserialize, BorshSerialize};

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

    /// We were unable to validate a signature.
    InvalidRequest,

    /// The nonce was inconsistent with our local storage of it!
    BadNonce,

    /// The deadline is out of date.
    BadDeadline,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Error {
    pub typ: ErrorDiscriminant,
    pub cd: Vec<u8>,
}

pub type R = Result<Res, Error>;

pub fn err_cd(typ: ErrorDiscriminant, cd: Vec<u8>) -> R {
    Err(Error { typ, cd })
}

pub fn err_cd_curry(typ: ErrorDiscriminant) -> impl FnOnce(Vec<u8>) -> R {
    move |cd| Err(Error { typ, cd })
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
