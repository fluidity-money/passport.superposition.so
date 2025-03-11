
use crate::encoding::*;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Res {
    /// This operation triggered a dummy interaction.
    NOTHING,

    /// The equivalent of returning a unit after a stateful action.
    DONE,
}
