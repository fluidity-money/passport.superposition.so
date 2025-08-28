use ed25519_dalek::SigningKey;

use crate::{
    applicative::{Applicative, ArgsCommit, SolverApplicative},
    conversion::{sign_cancel, sign_commit, sign_withdraw},
    error::Error,
};

pub struct SolverContext {
    pub signer: SigningKey,
}

impl SolverContext {
    pub fn new_from_bytes(x: [u8; 32]) -> Self {
        SolverContext {
            signer: SigningKey::from_bytes(&x),
        }
    }
}

impl SolverApplicative for SolverContext {
    fn withdraw(&self, ap: Applicative) -> Result<[u8; 64], Error> {
        sign_withdraw(&self.signer, &ap)
    }

    fn cancel(&self, ap: Applicative) -> Result<[u8; 64], Error> {
        sign_cancel(&self.signer, &ap)
    }

    fn commit(
        &self,
        ms_timestamp: u128,
        left: Applicative,
        right: Applicative,
    ) -> Result<[u8; 64], Error> {
        sign_commit(&self.signer, &ArgsCommit { ms_timestamp }, &left, &right)
    }
}
