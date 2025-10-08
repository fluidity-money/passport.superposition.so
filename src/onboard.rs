#[cfg(feature = "storage-gen-apply")]
use crate::{
    done_u64,
    error::{ApplyContext, Error, ErrorDiscriminant},
    sigs::make_onboarding_sig,
    Storage, R,
};

#[cfg(feature = "storage-gen-apply")]
use ed25519_dalek::{Signature, VerifyingKey};

#[cfg(feature = "storage-gen-apply")]
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U64},
    prelude::HostAccess,
};

#[cfg(feature = "storage-gen-apply")]
use stylus_panic::harness_dbg;

#[cfg(feature = "storage-gen-apply")]
impl Storage {
    fn chain_id(&self) -> u128 {
        self.vm().chain_id() as u128
    }

    pub fn onboard(
        &mut self,
        key: [u8; 32],
        sig: [u8; 64],
        contract: [u8; 20],
        nonce: u16,
        chain: u128,
        token: [u8; 20],
        value: u128,
        deadline: [u8; 32],
        permit_v: u8,
        permit_r: [u8; 32],
        permit_s: [u8; 32],
    ) -> R {
        harness_dbg!("Inside the onboard function");
        // Check the user's signature first. Concatenate the address with the
        // nonce, then feed it into the hashing function. Then use that to check
        // the signature that's given. Use that to set the state, then following
        // that, we need to add their liquidity using the permit blob that we
        // were given, and an external-facing function to the address they gave
        // us.
        #[cfg(not(feature = "dryrun"))]
        if self.chain_id() != chain {
            return Err(Error::from(ErrorDiscriminant::OnboardDifferentChainId));
        }
        harness_dbg!("After the chain id");
        if self.vm().contract_address().0 != contract {
            return Err(Error::from(ErrorDiscriminant::OnboardDifferentContract));
        }
        harness_dbg!("Contract address");
        let owner = self.vm().msg_sender();
        harness_dbg!("Owner is found");
        let addr_nonce_chain = make_onboarding_sig(&owner.into_array(), &contract, nonce, chain);
        harness_dbg!("Making onboarding sig");
        VerifyingKey::from_bytes(&key)
            .map_err(|_| Error::from(ErrorDiscriminant::BadVerifyingKey))?
            .verify_strict(&addr_nonce_chain, &Signature::from_bytes(&sig))
            .map_err(|_| Error::from(ErrorDiscriminant::BadOnboardingSig))?;
        harness_dbg!("Verifyingkey done");
        let key_count = u64::from_le_bytes(self.app.validation.ed25519_count.get().to_le_bytes());
        self.app
            .validation
            .ed25519_count
            .update_check_add(U64::from(1))
            .ok_or(Error::from(ErrorDiscriminant::CheckedAdd64).ctx(ApplyContext::Onboard))?;
        harness_dbg!("Set the validation info");
        let key = FixedBytes(key);
        self.app.validation.ed25519_keys.setter(key_count).set(key);
        self.app
            .validation
            .ed25519_owners
            .setter(key_count)
            .set(Address::from(owner));
        harness_dbg!("about to call the add_liq function");
        self.app.add_liq(
            token,
            owner.into_array(),
            value,
            deadline,
            permit_v,
            permit_r,
            permit_s,
        )?;
        done_u64(key_count)
    }
}
