use ed25519_dalek::SigningKey;

use stylus_sdk::alloy_primitives::Address;

use crate::{
    accounts::AccountsList,
    applicative::{Applicative, ArgsBalance, ArgsCommit, ArgsOrder, UserApplicative},
    conversion::*,
    error::Error,
};

use alloc::boxed::Box;

/// This code implements crypto's UserApplicative trait, to provide a
/// user-friendly vehicle to construct the Applicative type including
/// signing. For the life of the construction it maintains the account
/// table, which it provides during the conversion of this type to
/// the state machine type.
pub struct UserContext {
    pub accounts: AccountsList,
    pub signer: SigningKey,
}

impl UserContext {
    /// Create a new Accounts and register the Signer given.
    pub fn new_from_bytes(accounts: AccountsList, signer_b: &[u8; 32]) -> Self {
        let key = SigningKey::from_bytes(signer_b);
        UserContext {
            accounts: accounts.register(key.verifying_key()),
            signer: key,
        }
    }

    pub fn find_signer(&self) -> [u8; 4] {
        // We can panic here since this shouldn't be possible to construct
        // without the signing key in the Accounts map.
        self.accounts
            .find_place(&self.signer.verifying_key())
            .unwrap()
    }

    pub fn combine(_user: UserContext) -> UserContext {
        todo!()
    }
}

impl UserApplicative for UserContext {
    fn balance(
        &self,
        asset: Address,
        chain: u128,
        amount: u128,
        ms_timestamp: u128,
    ) -> Applicative {
        let args = ArgsBalance {
            asset: *asset.0,
            chain,
            amount,
            ms_timestamp,
        };
        Applicative::Balance(
            (self.find_signer(), sign_balance(&self.signer, &args)),
            args,
        )
    }

    fn withdraw(&self, solver_sig: [u8; 64], ap: Applicative) -> Result<Applicative, Error> {
        Ok(Applicative::Withdraw(
            solver_sig,
            (self.find_signer(), sign_withdraw(&self.signer, &ap)?),
            Box::new(ap),
        ))
    }

    fn order(
        &self,
        from_amt: u128,
        desired_asset: Address,
        desired_chain: u128,
        desired_amt: u128,
        ap: Applicative,
    ) -> Result<Applicative, Error> {
        let args = ArgsOrder {
            from_amt,
            desired_asset: *desired_asset.0,
            desired_chain,
            desired_amt,
        };
        Ok(Applicative::Order(
            (self.find_signer(), sign_order(&self.signer, &args, &ap)?),
            args,
            Box::new(ap),
        ))
    }

    fn cancel(&self, solver_sig: [u8; 64], ap: Applicative) -> Result<Applicative, Error> {
        Ok(Applicative::Cancel(
            solver_sig,
            (self.find_signer(), sign_cancel(&self.signer, &ap)?),
            Box::new(ap),
        ))
    }

    fn commit(
        &self,
        solver_sig: [u8; 64],
        ms_timestamp: u128,
        left: Applicative,
        right: Applicative,
    ) -> Result<Applicative, Error> {
        let args = ArgsCommit { ms_timestamp };
        Ok(Applicative::Commit(
            solver_sig,
            args,
            Box::new(left),
            Box::new(right),
        ))
    }

    fn commit_left_filled_to_balance(&self, ap: Applicative) -> Result<Applicative, Error> {
        Ok(Applicative::CommitLeftFilledToBalance(Box::new(ap)))
    }

    fn commit_right_filled_to_balance(&self, ap: Applicative) -> Result<Applicative, Error> {
        Ok(Applicative::CommitRightFilledToBalance(Box::new(ap)))
    }

    fn commit_left_excess_to_order(&self, ap: Applicative) -> Result<Applicative, Error> {
        Ok(Applicative::CommitLeftExcessToOrder(Box::new(ap)))
    }

    fn commit_right_excess_to_order(&self, ap: Applicative) -> Result<Applicative, Error> {
        Ok(Applicative::CommitRightExcessToOrder(Box::new(ap)))
    }

    fn join(&self, left: Applicative, right: Applicative) -> Result<Applicative, Error> {
        Ok(Applicative::Join(
            (self.find_signer(), sign_join(&self.signer, &left, &right)?),
            Box::new(left),
            Box::new(right),
        ))
    }
}
