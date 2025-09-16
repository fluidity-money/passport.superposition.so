use crate::{
    error::{Error, ErrorDiscriminant},
    Storage, done_u64, R,
};

use ed25519_dalek::{Signature, VerifyingKey};

use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U64},
    prelude::HostAccess,
};

impl Storage {
    pub fn onboard(
        &mut self,
        key: [u8; 32],
        sig: [u8; 64],
        nonce: u16,
        token: [u8; 20],
        value: u128,
        deadline: [u8; 32],
        permit_v: u8,
        permit_r: [u8; 32],
        permit_s: [u8; 32],
    ) -> R {
        // Check the user's signature first. Concatenate the address with the
        // nonce, then feed it into the hashing function. Then use that to check
        // the signature that's given. Use that to set the state, then following
        // that, we need to add their liquidity using the permit blob that we
        // were given, and an external-facing function to the address they gave
        // us.
        let owner = self.vm().msg_sender();
        let mut addr_and_nonce = [0u8; 20 + 2];
        addr_and_nonce[..20].copy_from_slice(owner.as_slice());
        addr_and_nonce[20..].copy_from_slice(&nonce.to_be_bytes());
        VerifyingKey::from_bytes(&key)
            .map_err(|_| Error {
                typ: ErrorDiscriminant::BadVerifyingKey,
            })?
            .verify_strict(&addr_and_nonce, &Signature::from_bytes(&sig))
            .map_err(|_| Error {
                typ: ErrorDiscriminant::BadStrictVerify,
            })?;
        let key_count = u64::from_le_bytes(self.ed25519_count.get().to_le_bytes());
        self.ed25519_count
            .update_check_add(U64::from(1))
            .ok_or(Error {
                typ: ErrorDiscriminant::CheckedAdd,
            })?;
        let key = FixedBytes(key);
        self.ed25519_keys.setter(key_count).set(key);
        self.ed25519_owners
            .setter(key_count)
            .set(Address::from(owner));
        self.add_liq(
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
