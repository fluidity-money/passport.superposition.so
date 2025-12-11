use ed25519_dalek::SigningKey;

use crate::{
    applicative::{Applicative, ArgsCommit, EdSig, SolverApplicative},
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
        sign_withdraw(&self.signer, ap)
    }

    fn cancel(&self, ap: &Applicative) -> Result<EdSig, Error> {
        sign_cancel(&self.signer, ap)
    }

    fn commit(
        &self,
        ms_timestamp: u32,
        left: &Applicative,
        right: &Applicative,
    ) -> Result<EdSig, Error> {
        sign_commit(
            &self.signer,
            &ArgsCommit {
                ms_timestamp,
            },
            left,
            right,
        )
    }
}
