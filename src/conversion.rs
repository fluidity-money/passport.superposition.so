use crate::{
    applicative::*,
    error::*,
    immutables::SOLVER_KEY_TESTNET,
    state_machine::{self, StateMachine},
    storage::StorageApplicationV1,
};

use alloc::vec::Vec;

use arrayvec::ArrayVec;

use stylus_sdk::alloy_primitives::{Address, FixedBytes};

use borsh::BorshSerialize;

use ed25519_dalek::{Signature, SigningKey, VerifyingKey};

use sha2::{digest::Digest, Sha512};

use alloc::boxed::Box;

fn err_sig() -> Error {
    Error {
        typ: ErrorDiscriminant::BadStrictVerify,
    }
}

fn err_prehashed() -> Error {
    Error {
        typ: ErrorDiscriminant::UnableToSignPrehashed,
    }
}

pub type Hash = [u8; 64];

pub type ValidateCarry = Result<Hash, Error>;

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
            &Signature::from_slice(sig).map_err(|_| err_sig())?,
        )
        .map_err(|_| {
            // When it comes to returning the error here, we can do so since the
            // caller will revert so we can be excessive with the penalties of
            // encoding a message.
            err_sig()
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
            &Signature::from_slice(sig1).map_err(|_| err_sig())?,
        )
        .map_err(|_| err_sig())?;
    verifying_key2
        .verify_prehashed_strict(
            d.clone(),
            None,
            &Signature::from_slice(sig2).map_err(|_| err_sig())?,
        )
        .map_err(|_| err_sig())?;
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
        .map_err(|_| err_prehashed())?
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

fn label(x: &Applicative) -> ApplicativeLabel {
    ApplicativeLabel::from(x)
}

fn err_bad_ap_transition(_from: ApplicativeLabel, _to: &Applicative) -> Error {
    Error {
        typ: ErrorDiscriminant::BadApplicativeTransition,
    }
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

fn err_hash_already_onchain(h: &[u8; 64]) -> Error {
    Error {
        typ: ErrorDiscriminant::HashAlreadyOnchain(h.clone()),
    }
}

impl StorageApplicationV1 {
    fn ensure_hash_unseen(&self, hash: &[u8; 64]) -> Result<(), Error> {
        // We need to truncate the first part of the hash to access it in the storage tree.
        if !self
            .details_hash_owner_l
            .get(FixedBytes::<32>::from_slice(&hash[..32]))
            .is_zero()
        {
            return Err(err_hash_already_onchain(hash));
        }
        Ok(())
    }

    /// Validate the Balance against the signature given using an array on the stack.
    fn validate_balance(
        &self,
        accounts: &Vec<u64>,
        (owner_i, owner_sig): &UserSig,
        ap: &ArgsBalance,
    ) -> Result<state_machine::Balance, Error> {
        let owner_id = accounts[*owner_i as usize];
        let o = self.find_ed25519_key(owner_id)?;
        let hash = check_sig(
            &o,
            owner_sig,
            &serialise_inplace::<_, { size_of::<ArgsBalance>() }>(ap),
            &[],
        )?;
        self.ensure_hash_unseen(&hash)?;
        Ok(state_machine::Balance::Inline(
            state_machine::BalanceArgs {
                ms_ts: ap.ms_timestamp,
                owner: self.find_ed25519_addr(owner_id)?,
                asset: Address::new(ap.asset),
                amt: ap.amount,
            },
            hash,
        ))
    }

    fn validate_commit_left_filled_to_bal(
        &self,
        accounts: &Vec<u64>,
        ap: &Applicative,
    ) -> Result<state_machine::Balance, Error> {
        let commit = self.validate_wrapped_commit(
            ApplicativeLabel::CommitLeftFilledToBalance,
            accounts,
            ap,
        )?;
        let c_hash = get_commit_hash(&commit);
        let hash = chain_digests(&[Nonce::CommitLeftFilledToBalance.into()], &c_hash);
        self.ensure_hash_unseen(&hash)?;
        Ok(state_machine::Balance::CommitLeftFilledToBal(
            Box::new(commit),
            hash,
        ))
    }

    fn validate_commit_right_filled_to_bal(
        &self,
        accounts: &Vec<u64>,
        ap: &Applicative,
    ) -> Result<state_machine::Balance, Error> {
        let commit = self.validate_wrapped_commit(
            ApplicativeLabel::CommitRightFilledToBalance,
            accounts,
            ap,
        )?;
        let c_hash = get_commit_hash(&commit);
        let hash = chain_digests(&[Nonce::CommitRightFilledToBalance.into()], &c_hash);
        self.ensure_hash_unseen(&hash)?;
        Ok(state_machine::Balance::CommitRightFilledToBal(
            Box::new(commit),
            hash,
        ))
    }

    fn validate_wrapped_balance(
        &self,
        from: ApplicativeLabel,
        accounts: &Vec<u64>,
        ap: &Applicative,
    ) -> Result<state_machine::Balance, Error> {
        match ap {
            Applicative::Balance(sig, args) => self.validate_balance(accounts, sig, args),
            Applicative::CommitLeftFilledToBalance(ap) => {
                self.validate_commit_left_filled_to_bal(accounts, ap)
            }
            Applicative::CommitRightFilledToBalance(ap) => {
                self.validate_commit_right_filled_to_bal(accounts, ap)
            }
            _ => Err(err_bad_ap_transition(from, ap)),
        }
    }

    fn validate_order(
        &self,
        accounts: &Vec<u64>,
        (owner_i, owner_sig): &UserSig,
        args: &ArgsOrder,
        ap: &Applicative,
    ) -> Result<state_machine::Order, Error> {
        let bal = self.validate_wrapped_balance(label(ap), accounts, ap)?;
        let bal_hash = get_bal_hash(&bal);
        let owner_id = accounts[*owner_i as usize];
        let o = self.find_ed25519_key(owner_id)?;
        let hash = check_sig(
            &o,
            owner_sig,
            &digest_inplace::<_, { size_of::<ArgsOrder>() }>(args),
            &bal_hash,
        )?;
        self.ensure_hash_unseen(&hash)?;
        Ok(state_machine::Order::Inline(
            state_machine::OrderArgs {
                desired_asset: Address::new(args.desired_asset),
                from_amt: args.from_amt,
                desired_amt: args.desired_amt,
                max_pol_fee: 0,              // TODO
                ord_partial_fill_okay: true, // TODO
            },
            Box::new(bal),
            hash,
        ))
    }

    fn validate_commit_left_excess_to_order(
        &self,
        accounts: &Vec<u64>,
        ap: &Applicative,
    ) -> Result<state_machine::Order, Error> {
        let commit =
            self.validate_wrapped_commit(ApplicativeLabel::CommitLeftExcessToOrder, accounts, ap)?;
        let c_hash = get_commit_hash(&commit);
        let hash = chain_digests(&[Nonce::CommitLeftExcessToOrder.into()], &c_hash);
        self.ensure_hash_unseen(&hash)?;
        Ok(state_machine::Order::CommitLeftExcessToOrder(
            Box::new(commit),
            hash,
        ))
    }

    fn validate_commit_right_excess_to_order(
        &self,
        accounts: &Vec<u64>,
        ap: &Applicative,
    ) -> Result<state_machine::Order, Error> {
        let commit =
            self.validate_wrapped_commit(ApplicativeLabel::CommitRightExcessToOrder, accounts, ap)?;
        let c_hash = get_commit_hash(&commit);
        let hash = chain_digests(&[Nonce::CommitRightExcessToOrder.into()], &c_hash);
        self.ensure_hash_unseen(&hash)?;
        Ok(state_machine::Order::CommitRightExcessToOrder(
            Box::new(commit),
            hash,
        ))
    }

    fn validate_wrapped_order(
        &self,
        from: ApplicativeLabel,
        accounts: &Vec<u64>,
        ap: &Applicative,
    ) -> Result<state_machine::Order, Error> {
        match ap {
            Applicative::Order(sig, args, ap) => self.validate_order(accounts, sig, args, ap),
            Applicative::CommitLeftExcessToOrder(ap) => {
                self.validate_commit_left_excess_to_order(accounts, ap)
            }
            Applicative::CommitRightExcessToOrder(ap) => {
                self.validate_commit_right_excess_to_order(accounts, ap)
            }
            _ => Err(err_bad_ap_transition(from, ap)),
        }
    }

    /// Validate a commit using the solver's signature. This function only
    /// validates the signature and the state transition, allowing the state
    /// machine to do the checking of the amounts and constraints.
    fn validate_commit(
        &self,
        accounts: &Vec<u64>,
        solver_sig: &[u8; 64],
        args: &ArgsCommit,
        left: &Applicative,
        right: &Applicative,
    ) -> Result<state_machine::Commit, Error> {
        let l = ApplicativeLabel::Commit;
        let left_order = self.validate_wrapped_order(l, accounts, left)?;
        let right_order = self.validate_wrapped_order(l, accounts, right)?;
        let left_hash = get_order_hash(&left_order);
        let right_hash = get_order_hash(&right_order);
        let hash = check_sig(
            &SOLVER_KEY_TESTNET,
            solver_sig,
            &digest_inplace::<_, { size_of::<ArgsCommit>() }>(args),
            &chain_digests(&left_hash, &right_hash),
        )?;
        self.ensure_hash_unseen(&hash)?;
        Ok(state_machine::Commit::Inline(
            state_machine::CommitArgs {
                ms_ts: args.ms_timestamp,
            },
            Box::new(left_order),
            Box::new(right_order),
            hash,
        ))
    }

    /// Validate the interior commit. Does not contain any
    /// values itself, but it does contain information on how the liquidity
    /// contained within the commit should be reused by virtue of its typing
    /// system. So the translation function knows how to manipulate this.
    /// Does not do any validation except validate the contained value.
    fn validate_wrapped_commit(
        &self,
        from: ApplicativeLabel,
        accounts: &Vec<u64>,
        ap: &Applicative,
    ) -> Result<state_machine::Commit, Error> {
        match ap {
            Applicative::Commit(sig, args, left, right) => {
                self.validate_commit(accounts, sig, args, left, right)
            }
            _ => Err(err_bad_ap_transition(from, ap)),
        }
    }

    /// Validate the state transition of the Withdrawal applicator and the
    /// signature. The Withdraw applicator can go from
    /// CommitLeftExcessToBalance, CommitRightExcessToBalance, and Balance to
    /// an amount that should be redeemed to the user by the contract.
    fn validate_withdraw(
        &self,
        accounts: &Vec<u64>,
        solver_sig: &[u8; 64],
        (owner_i, owner_sig): &UserSig,
        ap: &Applicative,
    ) -> Result<state_machine::Withdraw, Error> {
        // Since the argument to the right isn't known in the type here, we
        // validate the signature, and we feed the computed digest into a
        // concatenation here. Very stack expensive.
        let bal = self.validate_wrapped_balance(label(ap), accounts, ap)?;
        let bal_hash = get_bal_hash(&bal);
        let owner_id = accounts[*owner_i as usize];
        let o = self.find_ed25519_key(owner_id)?;
        let hash = check_sig_two(
            &SOLVER_KEY_TESTNET,
            solver_sig,
            &o,
            owner_sig,
            &[Nonce::Withdraw.into()],
            &bal_hash,
        )?;
        self.ensure_hash_unseen(&hash)?;
        Ok(state_machine::Withdraw::Inline(Box::new(bal), hash))
    }

    fn validate_cancel(
        &self,
        accounts: &Vec<u64>,
        solver_sig: &[u8; 64],
        (owner_i, owner_sig): &UserSig,
        ap: &Applicative,
    ) -> Result<state_machine::Balance, Error> {
        let order = self.validate_wrapped_order(ApplicativeLabel::Cancel, accounts, ap)?;
        let order_hash = get_order_hash(&order);
        let owner_id = accounts[*owner_i as usize];
        let o = self.find_ed25519_key(owner_id)?;
        let hash = check_sig_two(
            &SOLVER_KEY_TESTNET,
            solver_sig,
            &o,
            owner_sig,
            &[Nonce::Cancel.into()],
            &order_hash,
        )?;
        self.ensure_hash_unseen(&hash)?;
        Ok(state_machine::Balance::Cancel(Box::new(order), hash))
    }

    fn validate_join(
        &self,
        accounts: &Vec<u64>,
        (owner_i, owner_sig): &UserSig,
        left: &Applicative,
        right: &Applicative,
    ) -> Result<state_machine::Balance, Error> {
        let l = ApplicativeLabel::Join;
        let left_bal = self.validate_wrapped_balance(l, accounts, left)?;
        let right_bal = self.validate_wrapped_balance(l, accounts, right)?;
        let left_hash = get_bal_hash(&left_bal);
        let right_hash = get_bal_hash(&right_bal);
        let owner_id = accounts[*owner_i as usize];
        let o = self.find_ed25519_key(owner_id)?;
        Ok(state_machine::Balance::Join(
            Box::new(left_bal),
            Box::new(right_bal),
            check_sig(
                &o,
                owner_sig,
                &[Nonce::Join.into()],
                &chain_digests(&left_hash, &right_hash),
            )?,
        ))
    }

    /// Entrypoint validation function for a Applicative type during its
    /// validation stage.
    pub fn validate(&self, accounts: &Vec<u64>, ap: Applicative) -> Result<StateMachine, Error> {
        match ap {
            Applicative::Balance(sig, args) => Ok(StateMachine::Balance(
                self.validate_balance(accounts, &sig, &args)?,
            )),
            Applicative::Withdraw(solver_sig, user_sig, _, ap) => Ok(StateMachine::Withdraw(
                self.validate_withdraw(accounts, &solver_sig, &user_sig, &ap)?,
            )),
            Applicative::Order(user_sig, args, ap) => Ok(StateMachine::Order(
                self.validate_order(accounts, &user_sig, &args, &ap)?,
            )),
            Applicative::Cancel(solver_sig, user_sig, ap) => Ok(StateMachine::Balance(
                self.validate_cancel(accounts, &solver_sig, &user_sig, &ap)?,
            )),
            Applicative::Commit(solver_sig, args, ap1, ap2) => Ok(StateMachine::Commit(
                self.validate_commit(accounts, &solver_sig, &args, &ap1, &ap2)?,
            )),
            Applicative::CommitLeftFilledToBalance(ap) => Ok(StateMachine::Balance(
                self.validate_commit_left_filled_to_bal(accounts, &ap)?,
            )),
            Applicative::CommitRightFilledToBalance(ap) => Ok(StateMachine::Balance(
                self.validate_commit_right_filled_to_bal(accounts, &ap)?,
            )),
            Applicative::CommitLeftExcessToOrder(ap) => Ok(StateMachine::Order(
                self.validate_commit_left_excess_to_order(accounts, &ap)?,
            )),
            Applicative::CommitRightExcessToOrder(ap) => Ok(StateMachine::Order(
                self.validate_commit_right_excess_to_order(accounts, &ap)?,
            )),
            Applicative::Join(user_sig, left, right) => Ok(StateMachine::Balance(
                self.validate_join(accounts, &user_sig, &left, &right)?,
            )),
        }
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

pub fn digest_wrapped_balance(from: ApplicativeLabel, ap: &Applicative) -> Result<[u8; 64], Error> {
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
        &digest_inplace::<_, { size_of::<ArgsOrder>() }>(args),
        &digest_wrapped_balance(ApplicativeLabel::Order, &ap)?,
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
            &digest_inplace::<_, { size_of::<ArgsCommit>() }>(args),
            &chain_digests(
                &digest_wrapped_order(from, left)?,
                &digest_wrapped_order(from, right)?,
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
            &digest_wrapped_order(l, left)?,
            &digest_wrapped_order(l, right)?,
        ),
    )
}

pub fn sign_join(
    k: &SigningKey,
    left: &Applicative,
    right: &Applicative,
) -> Result<[u8; 64], Error> {
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
