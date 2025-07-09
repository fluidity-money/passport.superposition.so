use ed25519_dalek::SigningKey;

use crate::{
    applicative::{Applicative, ArgsCommit},
    crypto::{sign_cancel, sign_commit, sign_withdraw},
    error::Error,
};

pub struct SolverContext {
    pub signer: SigningKey,
}

impl SolverContext {
    pub fn withdraw(&self, ap: Applicative) -> Result<[u8; 64], Error> {
        sign_withdraw(&self.signer, &ap)
    }

    pub fn cancel(&self, ap: Applicative) -> Result<[u8; 64], Error> {
        sign_cancel(&self.signer, &ap)
    }

    pub fn commit(
        &self,
        ms_timestamp: u128,
        left: Applicative,
        right: Applicative,
    ) -> Result<[u8; 64], Error> {
        sign_commit(&self.signer, &ArgsCommit { ms_timestamp }, &left, &right)
    }
}
