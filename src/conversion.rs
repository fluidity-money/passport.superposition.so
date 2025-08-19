use crate::{
    accounts::AccountsExpanded,
    applicative::{self, Applicative},
    error::*,
    state_machine::{Bucket, Colour},
    storage::StoragePassport,
};

// Most of the operations in this file are done with a type of pattern
// matching that goes pretty deep inside the structure. It does this to
// efficiently unpack the sent applicative form. This step checks
// balances inside the structure, but does not check if the user actually
// has enough balance to fulfill a trade. It needs self to look up the
// owners of signatures for the transformation however.

fn err_bad_conv() -> Error {
    Error {
        typ: ErrorDiscriminant::BadConversionFrom,
        cd: vec![],
    }
}

fn err_bad_asset_comp() -> Error {
    Error {
        typ: ErrorDiscriminant::BadAssetComparison,
        cd: vec![],
    }
}

fn err_not_enough_for_derivative() -> Error {
    Error {
        typ: ErrorDiscriminant::NotEnoughForDeriv,
        cd: vec![],
    }
}

fn err_owner_diff() -> Error {
    Error {
        typ: ErrorDiscriminant::InconsistentOwners,
        cd: vec![],
    }
}

impl StoragePassport {
    pub fn order_to_bucket(
        &self,
        accounts: &AccountsExpanded,
        ap: Applicative,
    ) -> Result<Bucket, Error> {
        let Applicative::Order(_, applicative::ArgsOrder { from_amt, .. }, from) = ap else {
            return Err(err_bad_conv());
        };
        let from = self.balance_to_bucket(accounts, *from)?;
        let Bucket {
            owner,
            ref colour,
            spent,
            ms_ts,
            chain,
            asset,
            ..
        } = from;
        if from_amt > spent {
            return Err(err_not_enough_for_derivative());
        }
        // Whatever the user doesn't want to spend in this order we allow them to
        // consume separately.
        let saved = spent - from_amt;
        Ok(Bucket {
            colour: colour.clone(),
            from: Some(Box::new(from)),
            spent: from_amt,
            saved,
            ms_ts,
            chain,
            asset,
            owner,
        })
    }

    // Validate the internal constraints of the Commit.
    pub fn validate_commit(&self, _order: &Applicative) -> Result<(), Error> {
        todo!()
    }

    fn order_desired(&self, ap: &Applicative) -> Result<u128, Error> {
        // Recursively descend into the Applicative to find the amount that the
        // user's order, or the derived order, is trying to fill.
        match *ap {
            Applicative::Order(_, applicative::ArgsOrder { desired_amt, .. }, _) => Ok(desired_amt),
            _ => todo!(),
        }
    }

    pub fn commit_left_filled_to_bucket(
        &self,
        accounts: &AccountsExpanded,
        ap: Applicative,
    ) -> Result<Bucket, Error> {
        let Applicative::Commit(_, applicative::ArgsCommit { ms_timestamp }, ref left, ref right) =
            ap
        else {
            return Err(err_bad_conv());
        };
        self.validate_commit(&ap)?;
        // To get the amount that was filled for the left side, we need to get
        // the max of the amount of the right side that's spendable, or the
        // desired amount. To get this information, we need to reconcile the
        // delta between each commit recursively to build up a temporary balance
        // of each position.
        let left_desired = self.order_desired(&left)?;
        let left = self.order_to_bucket(accounts, *left.clone())?;
        let Bucket {
            spent: left_amt,
            owner,
            chain,
            ..
        } = left;
        let right = self.order_to_bucket(accounts, *right.clone())?;
        let Bucket {
            spent: right_amt,
            asset,
            ..
        } = right;
        let left_filled = {
            let x = left_amt.wrapping_sub(right_amt);
            if x > left_desired {
                x
            } else {
                left_desired
            }
        };
        Ok(Bucket {
            colour: Colour::OtherUser,
            from: Some(Box::new(left)),
            // The spent amount here is the amount that was consumed in the commit.
            spent: left_filled,
            // We set the saved amount here to 0, since the storage should be set for the
            // user at the order stage with the amount that's not filled, so we can retrieve
            // it later.
            saved: 0,
            ms_ts: ms_timestamp,
            chain,
            asset,
            owner,
        })
    }

    pub fn balance_to_bucket(
        &self,
        accounts: &AccountsExpanded,
        ap: Applicative,
    ) -> Result<Bucket, Error> {
        match ap {
            Applicative::Balance(
                (bal_id, _),
                applicative::ArgsBalance {
                    asset,
                    chain,
                    amount,
                    ms_timestamp,
                },
            ) => Ok(Bucket {
                colour: Colour::Ephereal,
                from: None,
                spent: amount,
                saved: 0,
                ms_ts: ms_timestamp,
                chain,
                asset: asset.x,
                owner: self.find_ed25519_addr(accounts, bal_id)?,
            }),
            Applicative::CommitLeftFilledToBalance(_)
            | Applicative::CommitRightFilledToBalance(_) => {
                let b = match ap {
                    Applicative::CommitLeftFilledToBalance(o) => {
                        self.commit_left_filled_to_bucket(accounts, *o)
                    }
                    Applicative::CommitRightFilledToBalance(_) => {
                        todo!()
                    }
                    _ => unreachable!(),
                }?;
                let Bucket {
                    spent,
                    ms_ts,
                    asset,
                    chain,
                    owner,
                    ..
                } = b;
                Ok(Bucket {
                    colour: Colour::OtherUser,
                    from: Some(Box::new(b)),
                    spent,
                    saved: 0,
                    ms_ts,
                    chain,
                    asset,
                    owner,
                })
            }
            _ => Err(err_bad_conv()),
        }
    }

    pub fn withdraw_to_bucket(
        &self,
        accounts: &AccountsExpanded,
        ap: Applicative,
    ) -> Result<Bucket, Error> {
        let Applicative::Withdraw(_, _, from) = ap else {
            return Err(err_bad_conv());
        };
        let colour = match *from {
            Applicative::Balance(_, _) => Colour::Ephereal,
            Applicative::CommitLeftFilledToBalance(_)
            | Applicative::CommitRightFilledToBalance(_) => Colour::OtherUser,
            _ => unreachable!(),
        };
        let b = self.balance_to_bucket(accounts, *from)?;
        let Bucket {
            spent,
            ms_ts,
            chain,
            asset,
            owner,
            ..
        } = b;
        Ok(Bucket {
            colour,
            from: Some(Box::new(b)),
            spent,
            saved: 0,
            ms_ts,
            chain,
            asset,
            owner,
        })
    }
}
