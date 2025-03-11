
use crate::encoding::*;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Res {
    DONE,
    COUNT(BU256)
}
