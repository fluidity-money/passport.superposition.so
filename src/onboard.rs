use crate::{
    done_u64,
    error::{Error, ErrorDiscriminant},
    sigs::make_onboarding_sig,
    storage::*,
    add_liq::add_liq,
    R,
};

use ed25519_dalek::{Signature, VerifyingKey};

use bobcat_sdk::{
    entry::{chain_id, contract_address, msg_sender},
    maths::U,
};

pub fn onboard(
    key: [u8; 32],
    sig: [u8; 64],
    contract: [u8; 20],
    nonce: u16,
    chain: u64,
    token: [u8; 20],
    value: u128,
    deadline: U,
    permit_v: u8,
    permit_r: U,
    permit_s: U,
) -> R {
    // Check the user's signature first. Concatenate the address with the
    // nonce, then feed it into the hashing function. Then use that to check
    // the signature that's given. Use that to set the state, then following
    // that, we need to add their liquidity using the permit blob that we
    // were given, and an external-facing function to the address they gave
    // us.
    #[cfg(not(feature = "dryrun"))]
    if chain_id() != chain {
        return Err(Error::from(ErrorDiscriminant::OnboardDifferentChainId));
    }
    if contract_address() != contract {
        return Err(Error::from(ErrorDiscriminant::OnboardDifferentContract));
    }
    let owner = msg_sender();
    let addr_nonce_chain = make_onboarding_sig(&owner, &contract, nonce, chain);
    VerifyingKey::from_bytes(&key)
        .map_err(|_| Error::from(ErrorDiscriminant::BadVerifyingKey))?
        .verify_strict(&addr_nonce_chain, &Signature::from_bytes(&sig))
        .map_err(|_| Error::from(ErrorDiscriminant::BadOnboardingSig))?;
    let key_count = ed25519_count::get();
    ed25519_count::add(&U::ONE);
    ed25519_keys::set(&key_count, &U::from(key));
    ed25519_owners::set(&key_count, &U::from(owner));
    add_liq(token, owner, value, deadline, permit_v, permit_r, permit_s)?;
    done_u64(key_count.into())
}
