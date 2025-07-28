use crate::{
    accounts::AccountsExpanded, applicative::Applicative, crypto, emissions::Emissions,
    error::Error, state_machine, storage::StoragePassport,
};

pub enum StateMachine {
    CreateBalance(state_machine::CreateBalance),
    Withdrawal(state_machine::Withdrawal),
    SplitBalance(state_machine::SplitBalance),
    StateBalance(state_machine::StateBalance),
    OrderCreated(state_machine::OrderCreated),
    Commit(state_machine::Commit),
}

/// Helper trait for operating on the core business logic of the passport.
/// Read the README for more.
pub trait Application {
    fn validate(accounts: &AccountsExpanded, ap: &Applicative) -> Result<(), Error>;
    fn convert(accounts: AccountsExpanded, ap: Applicative) -> Result<StateMachine, Error>;
    fn apply(state_machine: StateMachine) -> Result<Emissions, Error>;
}

impl Application for StoragePassport {
    fn validate(accounts: &AccountsExpanded, ap: &Applicative) -> Result<(), Error> {
        crypto::validate(accounts, ap)?;
        Ok(())
    }

    fn convert(_: AccountsExpanded, _: Applicative) -> Result<StateMachine, Error> {
        todo!()
    }

    fn apply(_: StateMachine) -> Result<Emissions, Error> {
        todo!()
    }
}
