use ed25519_dalek::SigningKey;

use crate::{
    applicative::{Applicative, U128, EdSig, ArgsCommit, SolverApplicative},
    conversion::{sign_cancel, sign_commit, sign_withdraw},
    error::Error,
};

pub struct SolverContext {
    pub signer: SigningKey,
}

impl SolverContext {
    pub fn new(k: SigningKey) -> Self {
        SolverContext { signer: k }
    }
}

impl SolverApplicative for SolverContext {
    fn withdraw(&self, ap: &Applicative) -> Result<EdSig, Error> {
        Ok(sign_withdraw(&self.signer, ap)?.into())
    }

    fn cancel(&self, ap: &Applicative) -> Result<EdSig, Error> {
        Ok(sign_cancel(&self.signer, ap)?.into())
    }

    fn commit(
        &self,
        ms_timestamp: u128,
        left: &Applicative,
        right: &Applicative,
    ) -> Result<EdSig, Error> {
        Ok(sign_commit(
            &self.signer,
            &ArgsCommit { ms_timestamp: U128(ms_timestamp) },
            left,
            right,
        )?.into())
    }
}
