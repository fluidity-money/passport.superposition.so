use borsh::{BorshDeserialize, BorshSerialize};

/// End result return results of the user-facing kind.
#[derive(BorshSerialize, BorshDeserialize)]
pub enum Res {
    /// This operation triggered a dummy interaction.
    NOTHING,

    /// The equivalent of returning a unit after a stateful action.
    DONE,
}
