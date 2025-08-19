use crate::{
    accounts::AccountsExpanded, applicative::Applicative, crypto, error::Error,
    state_machine::Bucket, storage::StoragePassport,
};

/// Helper trait for operating on the core business logic of the passport.
/// Read the README for more.
pub trait Application {
    fn validate(&self, accounts: &AccountsExpanded, ap: &Applicative) -> Result<(), Error>;
    fn convert(&self, accounts: AccountsExpanded, ap: Applicative) -> Result<Bucket, Error>;
    fn apply(&mut self, bucket: Bucket) -> Result<(), Error>;
}

impl Application for StoragePassport {
    fn validate(&self, accounts: &AccountsExpanded, ap: &Applicative) -> Result<(), Error> {
        crypto::validate(accounts, ap)?;
        Ok(())
    }

    fn convert(&self, _: AccountsExpanded, _: Applicative) -> Result<Bucket, Error> {
        todo!()
    }

    fn apply(&mut self, _: Bucket) -> Result<(), Error> {
        todo!()
    }
}
