use crate::{accounts::Accounts, applicative::*, error::*};

use borsh::BorshSerialize;

use arrayvec::ArrayVec;

use ed25519_dalek::{Signature, VerifyingKey, SigningKey};

use sha2::{digest::Digest, Sha512};

use alloc::{format, string::String, vec::Vec};

use stylus_sdk::alloy_primitives::*;

fn err_str(d: ErrorDiscriminant, msg: String) -> Error {
    Error {
        typ: d,
        cd: msg.as_bytes().to_vec(),
    }
}

fn err_sig(msg: String) -> Error {
    err_str(ErrorDiscriminant::BadStrictVerify, msg)
}

pub type ValidateCarry = Result<[u8; 64], Error>;

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

fn serialise_inplace<'a, T: BorshSerialize, const CAP: usize>(x: T) -> ArrayVec<u8, CAP> {
    let mut b = ArrayVec::<u8, CAP>::new();
    x.serialize(&mut b).unwrap();
    b
}

/// Validate the Balance against the signature given using an array on the stack.
pub fn validate_balance(
    accounts: &Accounts,
    (owner_id, owner_sig): &UserSig,
    ap: &ArgsBalance,
) -> ValidateCarry {
    check_sig(
        accounts.find(*owner_id)?,
        owner_sig,
        &serialise_inplace::<_, { size_of::<ArgsBalance>() }>(ap),
        &[],
    )
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

pub fn validate_wrapped_balance(accounts: &Accounts, ap: &Applicative) -> ValidateCarry {
    match ap {
        Applicative::Balance(sig, args) => validate_balance(accounts, sig, args),
        Applicative::CommitLeftFilledToBalance(ap)
        | Applicative::CommitRightFilledToBalance(ap) => validate_wrapped_balance(accounts, ap),
        _ => Err(err_bad_ap_transition()),
    }
}

pub fn validate_order(
    accounts: &Accounts,
    (owner_id, owner_sig): &UserSig,
    args: &ArgsOrder,
    ap: &Applicative,
) -> ValidateCarry {
    check_sig(
        accounts.find(*owner_id)?,
        owner_sig,
        &serialise_inplace::<_, { size_of::<ArgsOrder>() }>(args),
        &match ap {
            Applicative::CommitLeftFilledToBalance(ap)
            | Applicative::CommitRightFilledToBalance(ap) => validate_wrapped_commit(accounts, ap),
            ap => validate_wrapped_balance(accounts, ap),
        }?,
    )
}

pub fn validate_wrapped_order(accounts: &Accounts, ap: &Applicative) -> ValidateCarry {
    match ap {
        Applicative::Order(sig, args, ap) => validate_order(accounts, sig, args, ap),
        Applicative::CommitLeftExcessToOrder(ap) | Applicative::CommitRightExcessToOrder(ap) => {
            validate_wrapped_commit(accounts, ap)
        }
        _ => Err(err_bad_ap_transition()),
    }
}

/// Validate a commit using the solver's signature. This function only
/// validates the signature and the state transition, allowing the state
/// machine to do the checking of the amounts and constraints.
pub fn validate_commit(
    accounts: &Accounts,
    solver_sig: &[u8; 64],
    args: &ArgsCommit,
    left: &Applicative,
    right: &Applicative,
) -> ValidateCarry {
    check_sig(
        &accounts.solver,
        solver_sig,
        &serialise_inplace::<_, { size_of::<ArgsCommit>() }>(args),
        &chain_digests(
            validate_wrapped_order(accounts, left)?,
            validate_wrapped_order(accounts, right)?,
        ),
    )
}

/// Validate the interior commit. Does not contain any
/// values itself, but it does contain information on how the liquidity
/// contained within the commit should be reused by virtue of its typing
/// system. So the translation function knows how to manipulate this.
/// Does not do any validation except validate the contained value.
pub fn validate_wrapped_commit(accounts: &Accounts, ap: &Applicative) -> ValidateCarry {
    match ap {
        Applicative::Commit(sig, args, left, right) => {
            validate_commit(accounts, sig, args, left, right)
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
    solver_sig: &[u8; 64],
    (owner_id, owner_sig): &UserSig,
    ap: &Applicative,
) -> ValidateCarry {
    // Since the argument to the right isn't known in the type here, we
    // validate the signature, and we feed the computed digest into a
    // concatenation here. Very stack expensive.
    check_sig_two(
        &accounts.solver,
        solver_sig,
        accounts.find(*owner_id)?,
        owner_sig,
        &[],
        &match ap {
            Applicative::Balance(sig, args) => validate_balance(accounts, sig, args),
            Applicative::CommitLeftFilledToBalance(args)
            | Applicative::CommitRightFilledToBalance(args) => {
                validate_wrapped_commit(accounts, args)
            }
            _ => Err(err_bad_ap_transition()),
        }?,
    )
}

pub fn validate_cancel(
    accounts: &Accounts,
    solver_sig: &[u8; 64],
    (owner_id, owner_sig): &UserSig,
    ap: &Applicative,
) -> ValidateCarry {
    check_sig_two(
        &accounts.solver,
        solver_sig,
        accounts.find(*owner_id)?,
        owner_sig,
        &[],
        &match ap {
            Applicative::Order(user_sig, args, ap) => validate_order(accounts, user_sig, args, ap),
            // We only handle the excess amount cancellation since that's
            // the type aside from Commit that's implicitly turned into a
            // Order if it's not filled.
            Applicative::CommitLeftExcessToOrder(args)
            | Applicative::CommitRightExcessToOrder(args) => validate_wrapped_commit(accounts, args),
            _ => Err(err_bad_ap_transition()),
        }?,
    )
}

pub fn validate_join(
    accounts: &Accounts,
    (owner_id, owner_sig): &UserSig,
    left: &Applicative,
    right: &Applicative,
) -> ValidateCarry {
    check_sig(
        accounts.find(*owner_id)?,
        owner_sig,
        &[],
        &chain_digests(
            validate_wrapped_balance(accounts, left)?,
            validate_wrapped_balance(accounts, right)?,
        ),
    )
}

/// Entrypoint validation function for a Applicative type during its
/// validation stage.
pub fn validate(accounts: &Accounts, ap: &Applicative) -> ValidateCarry {
    match ap {
        Applicative::Balance(sig, args) => validate_balance(accounts, sig, args),
        Applicative::Withdraw(solver_sig, user_sig, ap) => {
            validate_withdraw(accounts, solver_sig, user_sig, ap)
        }
        Applicative::Order(user_sig, args, ap) => validate_order(accounts, user_sig, args, ap),
        Applicative::Cancel(solver_sig, user_sig, ap) => {
            validate_cancel(accounts, solver_sig, user_sig, ap)
        }
        Applicative::Commit(solver_sig, args, ap1, ap2) => {
            validate_commit(accounts, solver_sig, args, ap1, ap2)
        }
        Applicative::CommitLeftFilledToBalance(ap)
        | Applicative::CommitRightFilledToBalance(ap)
        | Applicative::CommitLeftExcessToOrder(ap)
        | Applicative::CommitRightExcessToOrder(ap) => validate_wrapped_commit(accounts, ap),
        Applicative::Join(user_sig, left, right) => validate_join(accounts, user_sig, left, right),
    }
}

/// User friendly trait for construction of Applicative with types
/// included.
pub trait UserApplicative {
    fn balance(signer: SigningKey, asset: Address, chain: u32, amount: U256) -> Applicative;
    fn withdraw(signer: SigningKey, commit: Applicative) -> Applicative;
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
