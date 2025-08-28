use crate::{
    accounts::AccountsExpanded, applicative::Applicative, error::Error,
    storage::StoragePassport, state_machine::StateMachine,
};

/// Helper trait for operating on the core business logic of the passport.
/// Read the README for more.
pub trait Application {
    fn convert(&self, accounts: &AccountsExpanded, ap: Applicative) -> Result<StateMachine, Error>;
    fn apply(&mut self, m: StateMachine) -> Result<(), Error>;
}

impl Application for StoragePassport {
    fn convert(&self, accounts: &AccountsExpanded, ap: Applicative) -> Result<StateMachine, Error> {
        self.validate(accounts, ap)
    }

    fn apply(
        &mut self,
        _ap: StateMachine,
    ) -> Result<(), Error> {
        todo!()
    }
}
