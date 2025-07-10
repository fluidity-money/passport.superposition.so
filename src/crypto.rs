use crate::{accounts::Accounts, applicative::*, error::*};

use borsh::BorshSerialize;

use arrayvec::ArrayVec;

use ed25519_dalek::{Signature, SigningKey, VerifyingKey};

use sha2::{digest::Digest, Sha512};

use alloc::{format, string::String, vec::Vec};

fn err_str(d: ErrorDiscriminant, msg: String) -> Error {
    Error {
        typ: d,
        cd: msg.as_bytes().to_vec(),
    }
}

fn err_sig(msg: String) -> Error {
    err_str(ErrorDiscriminant::BadStrictVerify, msg)
}

fn err_prehashed(msg: String) -> Error {
    err_str(ErrorDiscriminant::UnableToSignPrehashed, msg)
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

pub fn make_sig(key: &SigningKey, sig: &[u8], prev_digest: &[u8]) -> Result<[u8; 64], Error> {
    Ok(key
        .sign_prehashed(
            Sha512::default()
                .chain_update(sig)
                .chain_update(prev_digest),
            None,
        )
        .map_err(|msg| err_prehashed(format!("{msg}")))?
        .to_bytes())
}

pub fn serialise_inplace<'a, T: BorshSerialize, const CAP: usize>(x: &T) -> ArrayVec<u8, CAP> {
    let mut b = ArrayVec::<u8, CAP>::new();
    x.serialize(&mut b).unwrap();
    b
}

pub fn digest_inplace<'a, T: BorshSerialize, const CAP: usize>(x: &T) -> [u8; 64] {
    Sha512::default()
        .chain_update(&serialise_inplace::<_, CAP>(x))
        .finalize()
        .into()
}

/// Validate the Balance against the signature given using an array on the stack.
pub fn validate_balance(
    accounts: &Accounts,
    (owner_id, owner_sig): &UserSig,
    ap: &ArgsBalance,
) -> ValidateCarry {
    check_sig(
        &accounts.find_key(*owner_id)?,
        owner_sig,
        &serialise_inplace::<_, { size_of::<ArgsBalance>() }>(ap),
        &[],
    )
}

fn label(x: &Applicative) -> ApplicativeLabel {
    ApplicativeLabel::from(x)
}

fn err_bad_ap_transition(from: ApplicativeLabel, to: &Applicative) -> Error {
    Error {
        typ: ErrorDiscriminant::BadApplicativeTransition(from, label(to)),
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

pub fn validate_wrapped_balance(
    from: ApplicativeLabel,
    accounts: &Accounts,
    ap: &Applicative,
) -> ValidateCarry {
    match ap {
        Applicative::Balance(sig, args) => validate_balance(accounts, sig, args),
        Applicative::CommitLeftFilledToBalance(ap) => {
            validate_wrapped_commit(ApplicativeLabel::CommitLeftFilledToBalance, accounts, ap)
        }
        Applicative::CommitRightFilledToBalance(ap) => {
            validate_wrapped_commit(ApplicativeLabel::CommitRightFilledToBalance, accounts, ap)
        }
        _ => Err(err_bad_ap_transition(from, ap)),
    }
}

pub fn validate_order(
    accounts: &Accounts,
    (owner_id, owner_sig): &UserSig,
    args: &ArgsOrder,
    ap: &Applicative,
) -> ValidateCarry {
    check_sig(
        &accounts.find_key(*owner_id)?,
        owner_sig,
        &digest_inplace::<_, { size_of::<ArgsOrder>() }>(args),
        &match ap {
            Applicative::CommitLeftFilledToBalance(ap) => {
                validate_wrapped_commit(ApplicativeLabel::CommitLeftFilledToBalance, accounts, ap)
            }
            Applicative::CommitRightFilledToBalance(ap) => {
                validate_wrapped_commit(ApplicativeLabel::CommitRightFilledToBalance, accounts, ap)
            }
            ap => validate_wrapped_balance(label(ap), accounts, ap),
        }?,
    )
}

pub fn validate_wrapped_order(
    from: ApplicativeLabel,
    accounts: &Accounts,
    ap: &Applicative,
) -> ValidateCarry {
    match ap {
        Applicative::Order(sig, args, ap) => validate_order(accounts, sig, args, ap),
        Applicative::CommitLeftExcessToOrder(ap) => {
            validate_wrapped_commit(ApplicativeLabel::CommitLeftExcessToOrder, accounts, ap)
        }
        Applicative::CommitRightExcessToOrder(ap) => {
            validate_wrapped_commit(ApplicativeLabel::CommitRightExcessToOrder, accounts, ap)
        }
        _ => Err(err_bad_ap_transition(from, ap)),
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
    let l = ApplicativeLabel::Commit;
    check_sig(
        &accounts.solver,
        solver_sig,
        &digest_inplace::<_, { size_of::<ArgsCommit>() }>(args),
        &chain_digests(
            validate_wrapped_order(l, accounts, left)?,
            validate_wrapped_order(l, accounts, right)?,
        ),
    )
}

/// Validate the interior commit. Does not contain any
/// values itself, but it does contain information on how the liquidity
/// contained within the commit should be reused by virtue of its typing
/// system. So the translation function knows how to manipulate this.
/// Does not do any validation except validate the contained value.
pub fn validate_wrapped_commit(
    from: ApplicativeLabel,
    accounts: &Accounts,
    ap: &Applicative,
) -> ValidateCarry {
    match ap {
        Applicative::Commit(sig, args, left, right) => {
            validate_commit(accounts, sig, args, left, right)
        }
        _ => Err(err_bad_ap_transition(from, ap)),
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
    let l = ApplicativeLabel::Withdraw;
    check_sig_two(
        &accounts.solver,
        solver_sig,
        &accounts.find_key(*owner_id)?,
        owner_sig,
        &[Nonce::Withdraw.into()],
        &match ap {
            Applicative::Balance(sig, args) => validate_balance(accounts, sig, args),
            Applicative::CommitLeftFilledToBalance(ap) => {
                validate_wrapped_commit(ApplicativeLabel::CommitLeftFilledToBalance, accounts, ap)
            }
            Applicative::CommitRightFilledToBalance(ap) => {
                validate_wrapped_commit(ApplicativeLabel::CommitRightFilledToBalance, accounts, ap)
            }
            ap => Err(err_bad_ap_transition(l, ap)),
        }?,
    )
}

pub fn validate_cancel(
    accounts: &Accounts,
    solver_sig: &[u8; 64],
    (owner_id, owner_sig): &UserSig,
    ap: &Applicative,
) -> ValidateCarry {
    let l = label(ap);
    check_sig_two(
        &accounts.solver,
        solver_sig,
        &accounts.find_key(*owner_id)?,
        owner_sig,
        &[Nonce::Cancel.into()],
        &match ap {
            Applicative::Order(user_sig, args, ap) => validate_order(accounts, user_sig, args, ap),
            // We only handle the excess amount cancellation since that's
            // the type aside from Commit that's implicitly turned into a
            // Order if it's not filled.
            Applicative::CommitLeftExcessToOrder(ap) => {
                validate_wrapped_commit(ApplicativeLabel::CommitLeftExcessToOrder, accounts, ap)
            }
            Applicative::CommitRightExcessToOrder(ap) => {
                validate_wrapped_commit(ApplicativeLabel::CommitRightExcessToOrder, accounts, ap)
            }
            _ => Err(err_bad_ap_transition(l, ap)),
        }?,
    )
}

pub fn validate_join(
    accounts: &Accounts,
    (owner_id, owner_sig): &UserSig,
    left: &Applicative,
    right: &Applicative,
) -> ValidateCarry {
    let l = ApplicativeLabel::Join;
    check_sig(
        &accounts.find_key(*owner_id)?,
        owner_sig,
        &[Nonce::Join.into()],
        &chain_digests(
            validate_wrapped_balance(l, accounts, left)?,
            validate_wrapped_balance(l, accounts, right)?,
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
        Applicative::CommitLeftFilledToBalance(ap) => {
            validate_wrapped_commit(ApplicativeLabel::CommitLeftFilledToBalance, accounts, ap)
        }
        Applicative::CommitRightFilledToBalance(ap) => {
            validate_wrapped_commit(ApplicativeLabel::CommitRightFilledToBalance, accounts, ap)
        }
        Applicative::CommitLeftExcessToOrder(ap) => {
            validate_wrapped_commit(ApplicativeLabel::CommitLeftExcessToOrder, accounts, ap)
        }
        Applicative::CommitRightExcessToOrder(ap) => {
            validate_wrapped_commit(ApplicativeLabel::CommitRightExcessToOrder, accounts, ap)
        }
        Applicative::Join(user_sig, left, right) => validate_join(accounts, user_sig, left, right),
    }
}

pub fn sign_balance(k: &SigningKey, args: &ArgsBalance) -> [u8; 64] {
    make_sig(
        k,
        &serialise_inplace::<ArgsBalance, { size_of::<ArgsBalance>() }>(args),
        &[],
    )
    .unwrap()
}

fn digest_wrapped_balance(from: ApplicativeLabel, ap: &Applicative) -> Result<[u8; 64], Error> {
    match ap {
        Applicative::Balance(_, args) => {
            Ok(digest_inplace::<_, { size_of::<ArgsBalance>() }>(args))
        }
        Applicative::CommitLeftFilledToBalance(ap) => {
            digest_wrapped_commit(ApplicativeLabel::CommitLeftFilledToBalance, ap)
        }
        Applicative::CommitRightFilledToBalance(ap) => {
            digest_wrapped_commit(ApplicativeLabel::CommitRightFilledToBalance, ap)
        }
        ap => Err(err_bad_ap_transition(from, ap)),
    }
}

pub fn digest_order(args: &ArgsOrder, ap: &Applicative) -> Result<[u8; 64], Error> {
    Ok(chain_digests(
        digest_inplace::<_, { size_of::<ArgsOrder>() }>(args),
        digest_wrapped_balance(ApplicativeLabel::Order, ap)?,
    ))
}

fn digest_wrapped_order(from: ApplicativeLabel, ap: &Applicative) -> Result<[u8; 64], Error> {
    match ap {
        Applicative::Order(_, args, ap) => digest_order(args, ap),
        Applicative::CommitLeftExcessToOrder(ap) => {
            digest_wrapped_commit(ApplicativeLabel::CommitLeftExcessToOrder, ap)
        }
        Applicative::CommitRightExcessToOrder(ap) => {
            digest_wrapped_commit(ApplicativeLabel::CommitRightExcessToOrder, ap)
        }
        ap => Err(err_bad_ap_transition(from, ap)),
    }
}

fn digest_wrapped_commit(from: ApplicativeLabel, ap: &Applicative) -> Result<[u8; 64], Error> {
    if let Applicative::Commit(_, args, left, right) = ap {
        Ok(chain_digests(
            digest_inplace::<_, { size_of::<ArgsCommit>() }>(args),
            chain_digests(
                digest_wrapped_order(from, left)?,
                digest_wrapped_order(from, right)?,
            ),
        ))
    } else {
        Err(err_bad_ap_transition(from, ap))
    }
}

pub fn sign_withdraw(key: &SigningKey, ap: &Applicative) -> Result<[u8; 64], Error> {
    make_sig(
        key,
        &[Nonce::Withdraw.into()],
        &match ap {
            Applicative::Balance(_, args) => {
                Ok(digest_inplace::<_, { size_of::<ArgsBalance>() }>(args))
            }
            Applicative::CommitLeftFilledToBalance(ap) => {
                digest_wrapped_commit(ApplicativeLabel::CommitLeftFilledToBalance, ap)
            }
            Applicative::CommitRightFilledToBalance(ap) => {
                digest_wrapped_commit(ApplicativeLabel::CommitRightFilledToBalance, ap)
            }
            ap => Err(err_bad_ap_transition(ApplicativeLabel::Withdraw, ap)),
        }?,
    )
}

pub fn sign_order(key: &SigningKey, args: &ArgsOrder, ap: &Applicative) -> Result<[u8; 64], Error> {
    make_sig(
        key,
        &digest_inplace::<_, { size_of::<ArgsOrder>() }>(args),
        &digest_wrapped_balance(ApplicativeLabel::Order, ap)?,
    )
}

pub fn sign_cancel(k: &SigningKey, ap: &Applicative) -> Result<[u8; 64], Error> {
    make_sig(
        k,
        &[Nonce::Cancel.into()],
        &digest_wrapped_order(ApplicativeLabel::Cancel, ap)?,
    )
}

pub fn sign_commit(
    k: &SigningKey,
    args: &ArgsCommit,
    left: &Applicative,
    right: &Applicative,
) -> Result<[u8; 64], Error> {
    let l = ApplicativeLabel::Commit;
    make_sig(
        k,
        &digest_inplace::<_, { size_of::<ArgsCommit>() }>(args),
        &chain_digests(
            digest_wrapped_order(l, left)?,
            digest_wrapped_order(l, right)?,
        ),
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
            let a = Accounts::default().register(k.verifying_key());
            validate_balance(&a, &(0, sign_balance(&k, &args_bal)), &args_bal).unwrap();
            let bal = (0, sign_balance(&k, &args_bal));
            validate(&a, &Applicative::Balance(bal, args_bal.clone())).unwrap();
            // Test that someone can't break things:
            sign_key[31] = sign_key[31].wrapping_add(1);
            let k2 = SigningKey::from_bytes(&sign_key);
            assert!(
                validate_balance(&a, &(0, k2.sign(&b).to_bytes()), &args_bal)
                    .unwrap_err()
                    .is_typ(ErrorDiscriminant::BadStrictVerify)
            );
        }

        #[test]
        fn test_validate_balance_digest(
            sign_key in any::<[u8; 32]>(),
            args_bal in any::<ArgsBalance>()
        ) {
            let signer_key = SigningKey::from_bytes(&sign_key);
            let a = Accounts::default().register(signer_key.verifying_key());
            assert_eq!(
                digest_inplace::<_, { size_of::<ArgsBalance>() }>(&args_bal),
                digest_inplace::<_, { size_of::<ArgsBalance>() }>(&args_bal)
            );
            assert_eq!(
                digest_inplace::<_, { size_of::<ArgsBalance>() }>(&args_bal),
                validate_balance(
                    &a,
                    &(0, sign_balance(&signer_key, &args_bal)),
                    &args_bal
                )
                .unwrap()
            );
        }

        #[test]
        fn test_validate_withdraw(
            sign_key in any::<[u8; 32]>(),
            solver_key in any::<[u8; 32]>(),
            args_bal in any::<ArgsBalance>()
        ) {
            let solver_key = SigningKey::from_bytes(&solver_key);
            let signer_key = SigningKey::from_bytes(&sign_key);
            let a = Accounts::default().register(signer_key.verifying_key())
                .with_solver(solver_key.verifying_key().	to_bytes());
            let bal = Applicative::Balance((0, sign_balance(&signer_key, &args_bal)), args_bal);
            let solver_sig = sign_withdraw(&solver_key, &bal).unwrap();
            let signer_sig = (0, sign_withdraw(&signer_key, &bal).unwrap());
            validate(
                &a,
                &Applicative::Withdraw(solver_sig, signer_sig, Box::new(bal.clone())),
            )
            .unwrap();
            // Test it also breaks...
            let mut solver_sig = sign_withdraw(&solver_key, &bal).unwrap();
            solver_sig[31] = solver_sig[31].wrapping_add(1);
            assert!(
                validate(
                    &a,
                    &Applicative::Withdraw(solver_sig, signer_sig, Box::new(bal)),
                )
                .unwrap_err().is_typ(ErrorDiscriminant::BadStrictVerify)
            )
        }
    }
}
