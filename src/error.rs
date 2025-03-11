pub use crate::result::Res;

use crate::encoding::*;

use borsh::{BorshDeserialize, BorshSerialize};

use stylus_sdk::alloy_primitives::*;

#[derive(BorshSerialize, BorshDeserialize, PartialEq)]
pub enum ErrorDiscriminant {
    Dummy,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Error {
    typ: ErrorDiscriminant,
}

pub type R = Result<Res, Error>;

pub fn err_singl(x: ErrorDiscriminant) -> Error {
    Error { typ: x }
}

pub const NOTHING: R = Ok(Res::NOTHING);

pub const DONE: R = Ok(Res::DONE);
