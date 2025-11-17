use crate::{
    applicative::*,
    error::*,
    state_machine::{self, StateMachine},
    storage,
};

use alloc::vec::Vec;

use borsh::BorshSerialize;

use bobcat_sdk::maths::U;

use ed25519_dalek::{DigestSigner, DigestVerifier, Signature, SigningKey, VerifyingKey};

use sha2::{digest::Digest, Sha512};

use alloc::boxed::Box;

#[cfg(not(target_arch = "wasm32"))]
use proptest::prelude::*;

fn err_sig(from: ApplicativeLabel) -> Error {
    Error::from(ErrorDiscriminant::BadSignatureCreation).app(from)
}

fn err_verify(from: ApplicativeLabel) -> Error {
    Error::from(ErrorDiscriminant::BadStrictVerify).app(from)
}

fn err_verify_two(from: ApplicativeLabel, x: u8) -> Error {
    Error::from(ErrorDiscriminant::BadStrictVerifyTwo)
        .app(from)
        .side(x)
}

pub type Hash = [u8; 64];

pub type ValidateCarry = Result<Sha512, Error>;

fn check_sig(
    from: ApplicativeLabel,
    verifying_key: &VerifyingKey,
    sig: &EdSig,
    msg: &[u8],
    prev_digest: &[u8],
) -> ValidateCarry {
    let d = Sha512::default()
        .chain_update(msg)
        .chain_update(prev_digest);
    verifying_key
        .verify_digest(
            d.clone(),
            &Signature::from_slice(sig.into()).map_err(|_| err_sig(from))?,
        )
        .map_err(|_| {
            // When it comes to returning the error here, we can do so since the
            // caller will revert so we can be excessive with the penalties of
            // encoding a message.
            err_verify(from)
        })?;
    Ok(d)
}

fn check_sig_two(
    from: ApplicativeLabel,
    verifying_key1: &VerifyingKey,
    sig1: &EdSig,
    verifying_key2: &VerifyingKey,
    sig2: &EdSig,
    msg: &[u8],
    prev_digest: &[u8],
) -> ValidateCarry {
    let d = Sha512::default()
        .chain_update(msg)
        .chain_update(prev_digest);
    verifying_key2
        .verify_prehashed_strict(
            d.clone(),
            None,
            &Signature::from_slice(sig2.into()).map_err(|_| err_sig(from))?,
        )
        .map_err(|_| err_verify_two(from, 2))?;
    verifying_key1
        .verify_prehashed_strict(
            d.clone(),
            None,
            &Signature::from_slice(sig1.into()).map_err(|_| err_sig(from))?,
        )
        .map_err(|_| err_verify_two(from, 1))?;
    Ok(d)
}

pub fn make_sig(key: &SigningKey, sig: &[u8], prev_digest: &[u8]) -> Result<EdSig, Error> {
    let s: [u8; 64] = key
        .sign_digest(
            Sha512::default()
                .chain_update(sig)
                .chain_update(prev_digest),
        )
        .into();
    Ok(s.into())
}

struct Scratch<const CAP: usize> {
    x: [u8; CAP],
    c: usize,
}

impl<const CAP: usize> Default for Scratch<CAP> {
    fn default() -> Self {
        Scratch {
            x: [0u8; CAP],
            c: 0,
        }
    }
}

impl<const CAP: usize> borsh::io::Write for Scratch<CAP> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, borsh::io::Error> {
        // We don't bother with runtime protection here since we'll be
        // the only users. It's a compile time error if a type somehow
        // gets through that exceeds the size restriction.
        if buf.is_empty() {
            return Ok(0);
        }
        self.x[self.c..self.c + buf.len()].copy_from_slice(buf);
        self.c += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), borsh::io::Error> {
        Ok(())
    }
}

impl<const CAP: usize> AsRef<[u8]> for Scratch<CAP> {
    fn as_ref(&self) -> &[u8] {
        &self.x[..self.c]
    }
}

impl<const CAP: usize> core::ops::Deref for Scratch<CAP> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.x[..self.c]
    }
}

fn serialise_inplace<T: BorshSerialize, const CAP: usize>(x: &T) -> Scratch<CAP> {
    let mut b = Scratch::default();
    x.serialize(&mut b).unwrap();
    b
}

fn digest_inplace<T: BorshSerialize, const CAP: usize>(x: &T) -> [u8; 64] {
    Sha512::default()
        .chain_update(serialise_inplace::<_, CAP>(x))
        .finalize()
        .into()
}

fn label(x: &Applicative) -> ApplicativeLabel {
    ApplicativeLabel::from(x)
}

fn err_bad_ap_transition_digest(from: ApplicativeLabel, to: &Applicative) -> Error {
    Error::from(ErrorDiscriminant::BadApplicativeTransitionDigest)
        .app(from)
        .app_to(label(to))
}

fn err_bad_ap_transition_validate(from: ApplicativeLabel, to: &Applicative) -> Error {
    Error::from(ErrorDiscriminant::BadApplicativeTransitionValidate)
        .app(from)
        .app_to(label(to))
}

fn chain_digests(x: &[u8], y: &[u8]) -> [u8; 64] {
    Sha512::default()
        .chain_update(x)
        .chain_update(y)
        .finalize()
        .into()
}

fn get_bal_hash(st: &state_machine::Balance) -> Hash {
    use state_machine::Balance;
    match st {
        Balance::Inline(_, h)
        | Balance::CommitLeftFilledToBal(_, h)
        | Balance::CommitRightFilledToBal(_, h)
        | Balance::Onchain(h)
        | Balance::Cancel(_, h)
        | Balance::Join(_, _, h) => *h,
    }
}

fn get_order_hash(st: &state_machine::Order) -> Hash {
    use state_machine::Order;
    match st {
        Order::Inline(_, _, h)
        | Order::Onchain(h)
        | Order::CommitLeftExcessToOrder(_, h)
        | Order::CommitRightExcessToOrder(_, h) => *h,
    }
}

fn get_commit_hash(st: &state_machine::Commit) -> Hash {
    use state_machine::Commit;
    match st {
        Commit::Inline(_, _, _, h) | Commit::Onchain(h) => *h,
    }
}

/// Validate the Balance against the signature given using an array on the stack.
fn validate_balance(
    accounts: &[u64],
    (owner_i, owner_sig): &UserSig,
    args: &ArgsBalance,
) -> Result<state_machine::Balance, Error> {
    #[cfg(feature = "tracing")]
    dbg!("validating balance");
    let owner_id = U::from(accounts[*owner_i as usize]);
    let o = storage::find_ed25519_key(&owner_id)?;
    let d = check_sig(
        ApplicativeLabel::Balance,
        &o,
        owner_sig,
        &serialise_inplace::<_, { size_of::<ArgsBalance>() }>(args),
        &[],
    )?;
    let h = d.finalize().into();
    storage::ensure_hash_unseen(&h).ok_or(Error::from(ErrorDiscriminant::HashAlreadyOnchain))?;
    if args.amount.0 == 0 {
        return Err(Error::from(ErrorDiscriminant::ZeroBalanceAmount));
    }
    #[cfg(feature = "tracing")]
    dbg!("done with balance");
    Ok(state_machine::Balance::Inline(
        state_machine::BalanceArgs {
            ms_ts: args.ms_timestamp.0,
            owner: storage::find_ed25519_addr(&owner_id)?,
            asset: args.asset.0,
            amt: args.amount.0,
        },
        h,
    ))
}

fn validate_commit_left_filled_to_bal(
    solver_key: &VerifyingKey,
    accounts: &Vec<u64>,
    ap: &Applicative,
) -> Result<state_machine::Balance, Error> {
    #[cfg(feature = "tracing")]
    dbg!("validating commit left filled to balance");
    let commit = validate_wrapped_commit(
        solver_key,
        ApplicativeLabel::CommitLeftFilledToBalance,
        accounts,
        ap,
    )?;
    let c_hash = get_commit_hash(&commit);
    let h = chain_digests(&[Nonce::CommitLeftFilledToBalance.into()], &c_hash);
    storage::ensure_hash_unseen(&h).ok_or(Error::from(ErrorDiscriminant::HashAlreadyOnchain))?;
    #[cfg(feature = "tracing")]
    dbg!("done with commit left filled to bal");
    Ok(state_machine::Balance::CommitLeftFilledToBal(
        Box::new(commit),
        h,
    ))
}

fn validate_commit_right_filled_to_bal(
    solver_key: &VerifyingKey,
    accounts: &Vec<u64>,
    ap: &Applicative,
) -> Result<state_machine::Balance, Error> {
    #[cfg(feature = "tracing")]
    dbg!("validating commit right filled to balance");
    let commit = validate_wrapped_commit(
        solver_key,
        ApplicativeLabel::CommitRightFilledToBalance,
        accounts,
        ap,
    )?;
    let c_hash = get_commit_hash(&commit);
    let h = chain_digests(&[Nonce::CommitRightFilledToBalance.into()], &c_hash);
    storage::ensure_hash_unseen(&h).ok_or(Error::from(ErrorDiscriminant::HashAlreadyOnchain))?;
    #[cfg(feature = "tracing")]
    dbg!("done with commit right filled to bal");
    Ok(state_machine::Balance::CommitRightFilledToBal(
        Box::new(commit),
        h,
    ))
}

fn validate_wrapped_balance(
    solver_key: &VerifyingKey,
    from: ApplicativeLabel,
    accounts: &Vec<u64>,
    ap: &Applicative,
) -> Result<state_machine::Balance, Error> {
    match ap {
        Applicative::Balance(sig, args) => validate_balance(accounts, sig, args),
        Applicative::CommitLeftFilledToBalance(ap) => {
            validate_commit_left_filled_to_bal(solver_key, accounts, ap)
        }
        Applicative::CommitRightFilledToBalance(ap) => {
            validate_commit_right_filled_to_bal(solver_key, accounts, ap)
        }
        Applicative::Cancel(solver_sig, user_sig, ap) => {
            validate_cancel(solver_key, accounts, solver_sig, user_sig, ap)
        }
        _ => Err(err_bad_ap_transition_validate(from, ap)),
    }
}

fn validate_order(
    solver_key: &VerifyingKey,
    accounts: &Vec<u64>,
    (owner_i, owner_sig): &UserSig,
    args: &ArgsOrder,
    ap: &Applicative,
) -> Result<state_machine::Order, Error> {
    #[cfg(feature = "tracing")]
    dbg!("validating order");
    let bal = validate_wrapped_balance(solver_key, label(ap), accounts, ap)?;
    let bal_hash = get_bal_hash(&bal);
    let owner_id = U::from(accounts[*owner_i as usize]);
    let o = storage::find_ed25519_key(&owner_id)?;
    let d = check_sig(
        ApplicativeLabel::Order,
        &o,
        owner_sig,
        &digest_inplace::<_, { size_of::<ArgsOrder>() }>(args),
        &bal_hash,
    )?;
    #[cfg(feature = "tracing")]
    dbg!("done with order");
    Ok(state_machine::Order::Inline(
        state_machine::OrderArgs {
            desired_asset: args.desired_asset.0,
            from_amt: args.from_amt.0,
            desired_amt: args.desired_amt.0,
            max_pol_fee: 0,              // TODO
            ord_partial_fill_okay: true, // TODO
        },
        Box::new(bal),
        d.finalize().into(),
    ))
}

fn validate_commit_left_excess_to_order(
    solver_key: &VerifyingKey,
    accounts: &Vec<u64>,
    ap: &Applicative,
) -> Result<state_machine::Order, Error> {
    let commit = validate_wrapped_commit(
        solver_key,
        ApplicativeLabel::CommitLeftExcessToOrder,
        accounts,
        ap,
    )?;
    let c_hash = get_commit_hash(&commit);
    let hash = chain_digests(&[Nonce::CommitLeftExcessToOrder.into()], &c_hash);
    storage::ensure_hash_unseen(&hash).ok_or(Error::from(ErrorDiscriminant::HashAlreadyOnchain))?;
    #[cfg(feature = "tracing")]
    dbg!("done with commit left excess to order");
    Ok(state_machine::Order::CommitLeftExcessToOrder(
        Box::new(commit),
        hash,
    ))
}

fn validate_commit_right_excess_to_order(
    solver_key: &VerifyingKey,
    accounts: &Vec<u64>,
    ap: &Applicative,
) -> Result<state_machine::Order, Error> {
    let commit = validate_wrapped_commit(
        solver_key,
        ApplicativeLabel::CommitRightExcessToOrder,
        accounts,
        ap,
    )?;
    let c_hash = get_commit_hash(&commit);
    let hash = chain_digests(&[Nonce::CommitRightExcessToOrder.into()], &c_hash);
    storage::ensure_hash_unseen(&hash).ok_or(Error::from(ErrorDiscriminant::HashAlreadyOnchain))?;
    #[cfg(feature = "tracing")]
    dbg!("done with commit right excess to order");
    Ok(state_machine::Order::CommitRightExcessToOrder(
        Box::new(commit),
        hash,
    ))
}

fn validate_wrapped_order(
    solver_key: &VerifyingKey,
    from: ApplicativeLabel,
    accounts: &Vec<u64>,
    ap: &Applicative,
) -> Result<state_machine::Order, Error> {
    match ap {
        Applicative::Order(sig, args, ap) => validate_order(solver_key, accounts, sig, args, ap),
        Applicative::CommitLeftExcessToOrder(ap) => {
            validate_commit_left_excess_to_order(solver_key, accounts, ap)
        }
        Applicative::CommitRightExcessToOrder(ap) => {
            validate_commit_right_excess_to_order(solver_key, accounts, ap)
        }
        _ => Err(err_bad_ap_transition_validate(from, ap)),
    }
}

/// Validate a commit using the solver's signature. This function only
/// validates the signature and the state transition, allowing the state
/// machine to do the checking of the amounts and constraints.
fn validate_commit(
    solver_key: &VerifyingKey,
    accounts: &Vec<u64>,
    solver_sig: &EdSig,
    args: &ArgsCommit,
    left: &Applicative,
    right: &Applicative,
) -> Result<state_machine::Commit, Error> {
    #[cfg(feature = "tracing")]
    dbg!("validating commit");
    let l = ApplicativeLabel::Commit;
    let left_order = validate_wrapped_order(solver_key, l, accounts, left)?;
    let right_order = validate_wrapped_order(solver_key, l, accounts, right)?;
    let left_hash = get_order_hash(&left_order);
    let right_hash = get_order_hash(&right_order);
    let d = check_sig(
        ApplicativeLabel::Commit,
        solver_key,
        solver_sig,
        &digest_inplace::<_, { size_of::<ArgsCommit>() }>(args),
        &chain_digests(&left_hash, &right_hash),
    )?;
    #[cfg(feature = "tracing")]
    dbg!("done with commit");
    Ok(state_machine::Commit::Inline(
        state_machine::CommitArgs {
            ms_ts: args.ms_timestamp.0,
        },
        Box::new(left_order),
        Box::new(right_order),
        d.finalize().into(),
    ))
}

/// Validate the interior commit. Does not contain any
/// values itself, but it does contain information on how the liquidity
/// contained within the commit should be reused by virtue of its typing
/// system. So the translation function knows how to manipulate this.
/// Does not do any validation except validate the contained value.
fn validate_wrapped_commit(
    solver_key: &VerifyingKey,
    from: ApplicativeLabel,
    accounts: &Vec<u64>,
    ap: &Applicative,
) -> Result<state_machine::Commit, Error> {
    match ap {
        Applicative::Commit(sig, args, left, right) => {
            validate_commit(solver_key, accounts, sig, args, left, right)
        }
        _ => Err(err_bad_ap_transition_validate(from, ap)),
    }
}

/// Validate the state transition of the Withdrawal applicator and the
/// signature. The Withdraw applicator can go from
/// CommitLeftExcessToBalance, CommitRightExcessToBalance, and Balance to
/// an amount that should be redeemed to the user by the contract.
fn validate_withdraw(
    solver_key: &VerifyingKey,
    accounts: &Vec<u64>,
    solver_sig: &EdSig,
    (owner_i, owner_sig): &UserSig,
    ap: &Applicative,
) -> Result<state_machine::Withdraw, Error> {
    #[cfg(feature = "tracing")]
    dbg!("validating withdraw");
    // Since the argument to the right isn't known in the type here, we
    // validate the signature, and we feed the computed digest into a
    // concatenation here.
    let bal = validate_wrapped_balance(solver_key, label(ap), accounts, ap)?;
    let bal_hash = get_bal_hash(&bal);
    let owner_id = U::from(accounts[*owner_i as usize]);
    let o = storage::find_ed25519_key(&owner_id)?;
    let d = check_sig_two(
        ApplicativeLabel::Withdraw,
        solver_key,
        solver_sig,
        &o,
        owner_sig,
        &[Nonce::Withdraw.into()],
        &bal_hash,
    )?;
    #[cfg(feature = "tracing")]
    dbg!("done with withdraw");
    Ok(state_machine::Withdraw::Inline(
        Box::new(bal),
        d.finalize().into(),
    ))
}

fn validate_cancel(
    solver_key: &VerifyingKey,
    accounts: &Vec<u64>,
    solver_sig: &EdSig,
    (owner_i, owner_sig): &UserSig,
    ap: &Applicative,
) -> Result<state_machine::Balance, Error> {
    #[cfg(feature = "tracing")]
    dbg!("validating cancel");
    let order = validate_wrapped_order(solver_key, ApplicativeLabel::Cancel, accounts, ap)?;
    let order_hash = get_order_hash(&order);
    let owner_id = U::from(accounts[*owner_i as usize]);
    let o = storage::find_ed25519_key(&owner_id)?;
    let d = check_sig_two(
        ApplicativeLabel::Cancel,
        solver_key,
        solver_sig,
        &o,
        owner_sig,
        &[Nonce::Cancel.into()],
        &order_hash,
    )?;
    #[cfg(feature = "tracing")]
    dbg!("done with cancel");
    Ok(state_machine::Balance::Cancel(
        Box::new(order),
        d.finalize().into(),
    ))
}

fn validate_join(
    solver_key: &VerifyingKey,
    accounts: &Vec<u64>,
    (owner_i, owner_sig): &UserSig,
    left: &Applicative,
    right: &Applicative,
) -> Result<state_machine::Balance, Error> {
    #[cfg(feature = "tracing")]
    dbg!("validating join");
    let l = ApplicativeLabel::Join;
    let left_bal = validate_wrapped_balance(solver_key, l, accounts, left)?;
    let right_bal = validate_wrapped_balance(solver_key, l, accounts, right)?;
    let left_hash = get_bal_hash(&left_bal);
    let right_hash = get_bal_hash(&right_bal);
    let owner_id = U::from(accounts[*owner_i as usize]);
    let o = storage::find_ed25519_key(&owner_id)?;
    #[cfg(feature = "tracing")]
    dbg!("done with join");
    Ok(state_machine::Balance::Join(
        Box::new(left_bal),
        Box::new(right_bal),
        check_sig(
            ApplicativeLabel::Join,
            &o,
            owner_sig,
            &[Nonce::Join.into()],
            &chain_digests(&left_hash, &right_hash),
        )?
        .finalize()
        .into(),
    ))
}

/// Entrypoint validation function for a Applicative type during its
/// validation stage.
pub fn validate(
    solver_key: &VerifyingKey,
    accounts: &Vec<u64>,
    ap: &Applicative,
) -> Result<StateMachine, Error> {
    match ap {
        Applicative::Balance(sig, args) => Ok(StateMachine::Balance(validate_balance(
            accounts, sig, args,
        )?)),
        Applicative::Withdraw(solver_sig, user_sig, _, ap) => Ok(StateMachine::Withdraw(
            validate_withdraw(solver_key, accounts, solver_sig, user_sig, ap)?,
        )),
        Applicative::Order(user_sig, args, ap) => Ok(StateMachine::Order(validate_order(
            solver_key, accounts, user_sig, args, ap,
        )?)),
        Applicative::Cancel(solver_sig, user_sig, ap) => Ok(StateMachine::Balance(
            validate_cancel(solver_key, accounts, solver_sig, user_sig, ap)?,
        )),
        Applicative::Commit(solver_sig, args, ap1, ap2) => Ok(StateMachine::Commit(
            validate_commit(solver_key, accounts, solver_sig, args, ap1, ap2)?,
        )),
        Applicative::CommitLeftFilledToBalance(ap) => Ok(StateMachine::Balance(
            validate_commit_left_filled_to_bal(solver_key, accounts, ap)?,
        )),
        Applicative::CommitRightFilledToBalance(ap) => Ok(StateMachine::Balance(
            validate_commit_right_filled_to_bal(solver_key, accounts, ap)?,
        )),
        Applicative::CommitLeftExcessToOrder(ap) => Ok(StateMachine::Order(
            validate_commit_left_excess_to_order(solver_key, accounts, ap)?,
        )),
        Applicative::CommitRightExcessToOrder(ap) => Ok(StateMachine::Order(
            validate_commit_right_excess_to_order(solver_key, accounts, ap)?,
        )),
        Applicative::Join(user_sig, left, right) => Ok(StateMachine::Balance(validate_join(
            solver_key, accounts, user_sig, left, right,
        )?)),
    }
}

pub fn sign_balance(k: &SigningKey, args: &ArgsBalance) -> EdSig {
    make_sig(
        k,
        &serialise_inplace::<_, { size_of::<ArgsBalance>() }>(args),
        &[],
    )
    .unwrap()
}

pub fn digest_wrapped_balance(from: ApplicativeLabel, ap: &Applicative) -> Result<[u8; 64], Error> {
    match ap {
        Applicative::Balance(_, args) => {
            Ok(digest_inplace::<_, { size_of::<ArgsBalance>() }>(args))
        }
        Applicative::CommitLeftFilledToBalance(ap) => {
            let commit_hash =
                digest_wrapped_commit(ApplicativeLabel::CommitLeftFilledToBalance, ap)?;
            Ok(chain_digests(
                &[Nonce::CommitLeftFilledToBalance.into()],
                &commit_hash,
            ))
        }
        Applicative::CommitRightFilledToBalance(ap) => {
            let commit_hash =
                digest_wrapped_commit(ApplicativeLabel::CommitRightFilledToBalance, ap)?;
            Ok(chain_digests(
                &[Nonce::CommitRightFilledToBalance.into()],
                &commit_hash,
            ))
        }
        Applicative::Cancel(_, _, ap) => {
            let order_hash = digest_wrapped_order(ApplicativeLabel::Cancel, ap)?;
            Ok(chain_digests(&[Nonce::Cancel.into()], &order_hash))
        }
        ap => Err(err_bad_ap_transition_digest(from, ap)),
    }
}

pub fn digest_order(args: &ArgsOrder, ap: &Applicative) -> Result<[u8; 64], Error> {
    Ok(chain_digests(
        &digest_inplace::<_, { size_of::<ArgsOrder>() }>(args),
        &digest_wrapped_balance(ApplicativeLabel::Order, ap)?,
    ))
}

fn digest_wrapped_order(from: ApplicativeLabel, ap: &Applicative) -> Result<[u8; 64], Error> {
    match ap {
        Applicative::Order(_, args, ap) => digest_order(args, ap),
        Applicative::CommitLeftExcessToOrder(ap) => {
            let commit_hash = digest_wrapped_commit(ApplicativeLabel::CommitLeftExcessToOrder, ap)?;
            Ok(chain_digests(
                &[Nonce::CommitLeftExcessToOrder.into()],
                &commit_hash,
            ))
        }
        Applicative::CommitRightExcessToOrder(ap) => {
            let commit_hash =
                digest_wrapped_commit(ApplicativeLabel::CommitRightExcessToOrder, ap)?;
            Ok(chain_digests(
                &[Nonce::CommitRightExcessToOrder.into()],
                &commit_hash,
            ))
        }
        ap => Err(err_bad_ap_transition_digest(from, ap)),
    }
}

fn digest_wrapped_commit(from: ApplicativeLabel, ap: &Applicative) -> Result<[u8; 64], Error> {
    if let Applicative::Commit(_, args, left, right) = ap {
        Ok(chain_digests(
            &digest_inplace::<_, { size_of::<ArgsCommit>() }>(args),
            &chain_digests(
                &digest_wrapped_order(from, left)?,
                &digest_wrapped_order(from, right)?,
            ),
        ))
    } else {
        Err(err_bad_ap_transition_digest(from, ap))
    }
}

pub fn sign_withdraw(key: &SigningKey, ap: &Applicative) -> Result<EdSig, Error> {
    make_sig(
        key,
        &[Nonce::Withdraw.into()],
        &digest_wrapped_balance(ApplicativeLabel::Withdraw, ap)?,
    )
}

pub fn sign_order(key: &SigningKey, args: &ArgsOrder, ap: &Applicative) -> Result<EdSig, Error> {
    make_sig(
        key,
        &digest_inplace::<_, { size_of::<ArgsOrder>() }>(args),
        &digest_wrapped_balance(ApplicativeLabel::Order, ap)?,
    )
}

pub fn sign_cancel(k: &SigningKey, ap: &Applicative) -> Result<EdSig, Error> {
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
) -> Result<EdSig, Error> {
    let l = ApplicativeLabel::Commit;
    make_sig(
        k,
        &digest_inplace::<_, { size_of::<ArgsCommit>() }>(args),
        &chain_digests(
            &digest_wrapped_order(l, left)?,
            &digest_wrapped_order(l, right)?,
        ),
    )
}

pub fn sign_join(k: &SigningKey, left: &Applicative, right: &Applicative) -> Result<EdSig, Error> {
    let l = ApplicativeLabel::Join;
    make_sig(
        k,
        &[Nonce::Join.into()],
        &chain_digests(
            &digest_wrapped_balance(l, left)?,
            &digest_wrapped_balance(l, right)?,
        ),
    )
}

#[cfg(not(target_arch = "wasm32"))]
proptest! {
    #[test]
    fn test_sign_validate(
        p in any::<[u8; 32]>(),
        msg in any::<[u8; 32]>(),
        prev_digest in any::<Option<[u8; 64]>>()
    ) {
        let p = SigningKey::from_bytes(&p);
        let prev_digest = match prev_digest {
            Some(v) => v.to_vec(),
            None => Vec::new()
        };
        let s = make_sig(&p, &msg, &prev_digest).unwrap();
        check_sig(
            ApplicativeLabel::Balance,
            &p.verifying_key(),
            &s,
            &msg,
            &prev_digest
        )
        .unwrap();
    }
}

#[test]
fn test_signing_assumptions() {
    let args = ArgsBalance {
        asset: Asset([
            139, 154, 122, 66, 30, 11, 80, 120, 198, 114, 248, 170, 10, 100, 108, 141, 208, 224,
            170, 129,
        ]),
        chain: 123123123,
        amount: U128(226069396470166194839733876294202945097),
        ms_timestamp: U128(123),
    };
    let signer_priv = SigningKey::from_bytes(&[1u8; 32]);
    let sig = sign_balance(&signer_priv, &args);
    check_sig(
        ApplicativeLabel::Balance,
        &signer_priv.verifying_key(),
        &sig,
        &serialise_inplace::<_, { size_of::<ArgsBalance>() }>(&args),
        &[],
    )
    .unwrap();
}
