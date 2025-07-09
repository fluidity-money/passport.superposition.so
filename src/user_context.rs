use ed25519_dalek::SigningKey;

use stylus_sdk::alloy_primitives::{Address, U256};

use crate::{
    accounts::Accounts,
    applicative::{Applicative, ArgsBalance, UserApplicative},
    crypto::*,
    encoding::{BAddress, BU256},
};

/// This code implements crypto's UserApplicative trait, to provide a
/// user-friendly vehicle to construct the Applicative type including
/// signing. For the life of the construction it maintains the account
/// table, which it provides during the conversion of this type to
/// the state machine type.
pub struct UserContext {
    pub accounts: Accounts,
    pub signer: SigningKey,
}

impl UserContext {
    pub fn new_from_bytes(_accounts: Accounts, _signer_b: [u8; 32]) -> UserContext {
        todo!()
    }

    pub fn find_signer(&self) -> usize {
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
    fn balance(&self, asset: Address, chain: u32, amount: U256, ms_timestamp: u128) -> Applicative {
        let b = ArgsBalance {
            asset: BAddress { x: asset },
            chain,
            amount: BU256 { x: amount },
            ms_timestamp,
        };
        Applicative::Balance((self.find_signer(), sign_balance(&self.signer, &b)), b)
    }

    fn withdraw(&self, _solver_sig: [u8; 64], _ap: Applicative) -> Applicative {
        //Applicative::Withdraw(solver_sig, make_sig(self.signer,
        todo!()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod test_proptest {}
