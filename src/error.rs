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
    msg: u32
}

pub type R = Result<Res, Error>;

pub fn err_singl(x: ErrorDiscriminant) -> Error {
    Error {typ: x, msg: 0}
}

pub const DONE: R = Ok(Res::DONE);

pub fn ok_count(x: U256) -> R {
    Ok(Res::COUNT(BU256{x}))
}
