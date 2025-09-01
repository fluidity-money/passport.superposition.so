use crate::{
    accounts::AccountsExpanded, applicative::Applicative, error::Error,
    state_machine::StateMachine, storage::StoragePassport,
};

/// Helper trait for operating on the core business logic of the passport.
/// Read the README for more.
pub trait Application {
    fn convert(&self, accounts: &AccountsExpanded, ap: Applicative) -> Result<StateMachine, Error>;
    fn application(&mut self, m: StateMachine) -> Result<(), Error>;
}

impl Application for StoragePassport {
    fn convert(&self, accounts: &AccountsExpanded, ap: Applicative) -> Result<StateMachine, Error> {
        // This step should also check if the user's balance is enough to service
        // the amounts.
        self.validate(accounts, ap)
    }

    fn application(&mut self, ap: StateMachine) -> Result<(), Error> {
        self.apply(ap)
    }
}
