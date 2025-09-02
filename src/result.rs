use borsh::{BorshDeserialize, BorshSerialize};

/// End result return results of the user-facing kind.
#[derive(BorshSerialize, BorshDeserialize)]
pub enum Res {
    /// This operation triggered a dummy interaction.
    Noop,

    /// The equivalent of returning a unit after a stateful action.
    DoneUnit,

    /// A number was returned alongside correct execution.
    DoneU128(u128)
}
