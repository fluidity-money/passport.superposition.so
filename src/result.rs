use borsh::{BorshDeserialize, BorshSerialize};

use alloc::vec::Vec;

/// End result return results of the user-facing kind.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum Res {
    /// This operation triggered a dummy interaction.
    Noop,

    /// The equivalent of returning a unit after a stateful action.
    DoneUnit,

    /// A u64 number was returned alongside correct execution.
    DoneU64(u64),

    /// A number was returned alongside correct execution.
    DoneU128(u128)
}

impl From<Res> for Vec<u8> {
    fn from(r: Res) -> Self {
        borsh::to_vec(&r).unwrap()
    }
}

impl core::fmt::Display for Res {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}