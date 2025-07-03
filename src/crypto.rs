use crate::{accounts::Accounts, applicative::*, error::*};

use borsh::BorshSerialize;

use arrayvec::ArrayVec;

use ed25519_dalek::{Signature, VerifyingKey};

use sha2::{digest::Digest, Sha512};

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

pub type ValidateCarry = Result<[u8; 64], Error>;

fn conv_key(x: &[u8; 32]) -> Result<VerifyingKey, Error> {
    VerifyingKey::from_bytes(x).map_err(|err| err_verifying_key(format!("{err}")))
}

fn check_sig(
    verifying_key: &VerifyingKey,
    sig: &[u8; 64],
    msg: &[u8],
    prev_digest: &[u8],
) -> ValidateCarry {
    let d = Sha512::default()
        .chain_update(msg)
        .chain_update(prev_digest);
    verifying_key
        .verify_prehashed_strict(
            d.clone(),
            None,
            &Signature::from_slice(sig).map_err(|err| err_sig(format!("{err}")))?,
        )
        .map_err(|err| {
            // When it comes to returning the error here, we can do so since the
            // caller will revert so we can be excessive with the penalties of
            // encoding a message.
            err_sig(format!("{err}"))
        })?;
    Ok(d.finalize().into())
}

fn check_sig_two(
    verifying_key1: &VerifyingKey,
    sig1: &[u8; 64],
    verifying_key2: &VerifyingKey,
    sig2: &[u8; 64],
    msg: &[u8],
    prev_digest: &[u8],
) -> ValidateCarry {
    let d = Sha512::default()
        .chain_update(msg)
        .chain_update(prev_digest);
    verifying_key1
        .verify_prehashed_strict(
            d.clone(),
            None,
            &Signature::from_slice(sig1).map_err(|err| err_sig(format!("{err}")))?,
        )
        .map_err(|err| err_sig(format!("{err}")))?;
    verifying_key2
        .verify_prehashed_strict(
            d.clone(),
            None,
            &Signature::from_slice(sig2).map_err(|err| err_sig(format!("{err}")))?,
        )
        .map_err(|err| err_sig(format!("{err}")))?;
    Ok(d.finalize().into())
}

/// Validate the Balance against the signature given using an array on the stack.
pub fn validate_balance(
    accounts: &Accounts,
    signer_key: &VerifyingKey,
    sig: &[u8; 64],
    ap: &ArgsBalance,
) -> ValidateCarry {
    let mut b = ArrayVec::<u8, { size_of::<ArgsBalance>() }>::new();
    ap.serialize(&mut b).unwrap();
    check_sig(&signer_key, sig, &b, &[])
}

fn err_bad_ap_transition() -> Error {
    Error {
        typ: ErrorDiscriminant::BadApplicativeTransition,
        cd: Vec::new(),
    }
}

fn chain_digests(x: [u8; 64], y: [u8; 64]) -> [u8; 64] {
    Sha512::default()
        .chain_update(&x)
        .chain_update(&y)
        .finalize()
        .into()
}

pub fn validate_order(
    accounts: &Accounts,
    solver_key: &VerifyingKey,
    sig: &[u8; 64],
    args: &ArgsOrder,
    ap: &Applicative,
) -> ValidateCarry {
    check_sig(
        solver_key,
        sig,
        &[],
        &match ap {
            Applicative::Balance(sig, args) => todo!(),
            Applicative::CommitLeftFilledToBalance(args) => {
                validate_commit_left_filled_out(accounts, solver_key, args)
            }
            Applicative::CommitRightFilledToBalance(args) => {
                validate_commit_right_filled_out(accounts, solver_key, args)
            }
            Applicative::CommitLeftExcessToBalance(sig, args) => {
                validate_commit_left_excess_out(accounts, solver_key, args)
            }
            Applicative::CommitRightExcessToBalance(sig, args) => {
                validate_commit_right_excess_out(accounts, solver_key, args)
            }
            _ => Err(err_bad_ap_transition()),
        }?,
    )
}

/// Validate a commit using the solver's signature. This function only
/// validates the signature and the state transition, allowing the state
/// machine to do the checking of the amounts and constraints.
pub fn validate_commit(
    accounts: &Accounts,
    solver_key: &VerifyingKey,
    sig: &[u8; 64],
    left: &Applicative,
    right: &Applicative,
) -> ValidateCarry {
    check_sig(
        solver_key,
        sig,
        &[],
        &match (left, right) {
            (Applicative::Order(args1, sig1, ap1), Applicative::Order(args2, sig2, ap2)) => {
                let left_digest = validate_order(accounts, solver_key, args1, sig1, ap1)?;
                let right_digest = validate_order(accounts, solver_key, args2, sig2, ap2)?;
                Ok(chain_digests(left_digest, right_digest))
            }
            (_, _) => Err(err_bad_ap_transition()),
        }?,
    )
}

/// Validate the left commit's filled out amount. Does not contain any
/// values itself, but it does contain information on how the liquidity
/// contained within the commit should be reused by virtue of its typing
/// system. So the translation function knows how to manipulate this.
/// Does not do any validation except validate the contained value.
pub fn validate_commit_left_filled_out(
    accounts: &Accounts,
    solver_key: &VerifyingKey,
    args: &Applicative,
) -> ValidateCarry {
    match args {
        Applicative::Commit(sig, left, right) => {
            validate_commit(accounts, solver_key, sig, left, right)
        }
        _ => Err(err_bad_ap_transition()),
    }
}

pub fn validate_commit_right_filled_out(
    accounts: &Accounts,
    solver_key: &VerifyingKey,
    args: &Applicative,
) -> ValidateCarry {
    match args {
        Applicative::Commit(sig, left, right) => {
            validate_commit(accounts, solver_key, sig, left, right)
        }
        _ => Err(err_bad_ap_transition()),
    }
}

pub fn validate_commit_left_excess_out(
    accounts: &Accounts,
    solver_key: &VerifyingKey,
    args: &Applicative,
) -> ValidateCarry {
    match args {
        Applicative::Commit(sig, left, right) => {
            validate_commit(accounts, solver_key, sig, left, right)
        }
        _ => Err(err_bad_ap_transition()),
    }
}

pub fn validate_commit_right_excess_out(
    accounts: &Accounts,
    solver_key: &VerifyingKey,
    args: &Applicative,
) -> ValidateCarry {
    match args {
        Applicative::Commit(sig, left, right) => {
            validate_commit(accounts, solver_key, sig, left, right)
        }
        _ => Err(err_bad_ap_transition()),
    }
}

/// Validate the state transition of the Withdrawal applicator and the
/// signature. The Withdraw applicator can go from
/// CommitLeftExcessToBalance, CommitRightExcessToBalance, and Balance to
/// an amount that should be redeemed to the user by the contract.
pub fn validate_withdraw(
    accounts: &Accounts,
    solver_key: &VerifyingKey,
    solver_sig: &[u8; 64],
    signer_key: &VerifyingKey,
    signer_sig: &[u8; 64],
    args: &Applicative,
) -> ValidateCarry {
    // Since the argument to the right isn't known in the type here, we
    // validate the signature, and we feed the computed digest into a
    // concatenation here. Very stack expensive.
    check_sig_two(
        solver_key,
        solver_sig,
        signer_key,
        signer_sig,
        &[],
        &match args {
            Applicative::Balance(sig, args) => validate_balance(accounts, signer_key, sig, args),
            Applicative::CommitLeftFilledToBalance(args) => {
                validate_commit_left_filled_out(accounts, solver_key, args)
            }
            Applicative::CommitRightFilledToBalance(args) => {
                validate_commit_right_filled_out(accounts, solver_key, args)
            }
            Applicative::CommitLeftExcessToBalance(sig, args) => {
                validate_commit_left_excess_out(accounts, solver_key, args)
            }
            Applicative::CommitRightExcessToBalance(sig, args) => {
                validate_commit_right_excess_out(accounts, solver_key, args)
            }
            _ => Err(err_bad_ap_transition()),
        }?,
    )
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod test_proptest {
    use super::*;
    use proptest::prelude::*;

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
            let d = Sha512::default().chain_update(&b);
            let a = Accounts::default();
            validate_balance(&a, &vk, &k.sign_prehashed(d, None).unwrap().to_bytes(), &args_bal)
                .unwrap();
            // Test that someone can't break things:
            sign_key[31] = sign_key[31].wrapping_add(1);
            let k2 = SigningKey::from_bytes(&sign_key);
            assert!(
                validate_balance(&a, &vk, &k2.sign(&b).to_bytes(), &args_bal)
                    .unwrap_err()
                    .is_typ(ErrorDiscriminant::BadStrictVerify)
            );
        }
    }
}
