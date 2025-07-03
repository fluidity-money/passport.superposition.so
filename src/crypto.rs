use crate::{applicative::*, error::*};

use borsh::BorshSerialize;

use arrayvec::ArrayVec;

use ed25519_dalek::{Signature, VerifyingKey};

use alloc::{format, string::String};

fn err_str(d: ErrorDiscriminant, msg: String) -> Error {
    Error {
        typ: d,
        cd: msg.as_bytes().to_vec(),
    }
}

fn err_sig(msg: String) -> Error {
    err_str(ErrorDiscriminant::BadStrictVerify, msg)
}

fn err_verifying_key(msg: String) -> Error {
    err_str(ErrorDiscriminant::BadVerifyingKey, msg)
}

pub fn check_sig(verifying_key: &[u8; 32], sig: &[u8; 64], msg: &[u8]) -> Result<bool, Error> {
    let sig = Signature::from_slice(sig).map_err(|err| err_sig(format!("{err}")))?;
    VerifyingKey::from_bytes(verifying_key)
        .map_err(|err| err_verifying_key(format!("{err}")))?
        .verify_strict(&msg, &sig)
        .map_err(|err| {
            // When it comes to returning the error here, we can do so since the
            // caller will revert so we can be excessive with the penalties of
            // encoding a message.
            err_sig(format!("{err}"))
        })?;
    Ok(true)
}

// Validate the Balance against the signature given using an array on the stack.
pub fn validate_balance(
    signer_key: &[u8; 32],
    sig: &[u8; 64],
    ap: &ArgsBalance,
) -> Result<bool, Error> {
    let mut b = ArrayVec::<u8, { size_of::<ArgsBalance>() }>::new();
    ap.serialize(&mut b).unwrap();
    check_sig(signer_key, sig, &b)
}

/// Validate the state transition of the Withdrawal applicator and the
/// signature. The Withdraw applicator can go from
/// CommitLeftExcessToBalance, CommitRightExcessToBalance, and Balance to
/// an amount that should be redeemed to the user by the contract.
pub fn validate_withdraw(
    solver_key: &[u8; 32],
    solver_sig: &[u8; 64],
    signer_key: &[u8; 32],
    signer_sig: &[u8; 64],
    args: &Applicative
) -> Result<bool, Error> {
    Ok(true)
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod test_proptest {
    use proptest::prelude::*;
    use super::*;

    use ed25519_dalek::{Signer, SigningKey};

    proptest! {
        #[test]
        fn test_validate_bal(
            mut sign_key in any::<[u8; 32]>(),
            args_bal in any::<ArgsBalance>()
        ) {
            let k = SigningKey::from_bytes(&sign_key);
            let mut b = Vec::new();
            args_bal.serialize(&mut b).unwrap();
            let vk = k.verifying_key();
            let validation = validate_balance(&vk.to_bytes(), &k.sign(&b).to_bytes(), &args_bal)
                .unwrap();
            assert!(validation);
            // Test that someone can't break things:
            sign_key[31] = sign_key[31].wrapping_add(1);
            let k2 = SigningKey::from_bytes(&sign_key);
            assert!(
                validate_balance(&vk.to_bytes(), &k2.sign(&b).to_bytes(), &args_bal)
                    .unwrap_err()
                    .is_typ(ErrorDiscriminant::BadStrictVerify)
            );
        }
    }
}
