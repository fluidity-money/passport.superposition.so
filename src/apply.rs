use stylus_sdk::alloy_primitives::{Address, U256};

use crate::{
    call_eip20_extras,
    error::{Error, ErrorDiscriminant},
    state_machine::{Balance, BalanceArgs, Commit, Order, OrderArgs, StateMachine, Withdraw},
    storage::StorageApplication,
};

pub type R<T> = Result<T, Error>;

fn err_same_assets() -> Error {
    Error {
        typ: ErrorDiscriminant::SameAssets,
    }
}

fn err_bad_asset_asks() -> Error {
    Error {
        typ: ErrorDiscriminant::BadAssetAsks,
    }
}

fn err_bad_balance_from_order() -> Error {
    Error {
        typ: ErrorDiscriminant::BalanceTransitionToOrderBad,
    }
}

fn checked_sub(x: u128, y: u128) -> R<u128> {
    x.checked_sub(y).ok_or(Error {
        typ: ErrorDiscriminant::CheckedSub,
    })
}

impl StorageApplication {
    pub fn commit_left_owner(&self, c: &Commit) -> Address {
        match c {
            Commit::Inline(_, o, _, _) => self.order_owner(o),
            Commit::Onchain(h) => self.get_hash_owner_l(h),
        }
    }

    pub fn commit_right_owner(&self, c: &Commit) -> Address {
        match c {
            Commit::Inline(_, _, o, _) => self.order_owner(o),
            Commit::Onchain(h) => self.get_hash_owner_r(h),
        }
    }

    pub fn order_owner(&self, o: &Order) -> Address {
        match o {
            Order::Inline(_, b, _) => self.balance_owner(b),
            Order::Onchain(h) => self.get_hash_owner_l(h),
            Order::CommitLeftExcessToOrder(c, _) => self.commit_left_owner(c),
            Order::CommitRightExcessToOrder(c, _) => self.commit_right_owner(c),
        }
    }

    pub fn balance_owner(&self, b: &Balance) -> Address {
        match b {
            Balance::Inline(BalanceArgs { owner, .. }, _) => owner.clone(),
            Balance::Onchain(h) => self.get_hash_owner_l(h),
            Balance::CommitLeftFilledToBal(c, _) => self.commit_left_owner(c),
            Balance::CommitRightFilledToBal(c, _) => self.commit_right_owner(c),
            Balance::Cancel(o, _) => self.order_owner(o),
            Balance::Join(b, _, _) => self.balance_owner(b),
        }
    }

    pub fn balance_amount(&self, owner: Address, asset: Address, b: &Balance) -> R<u128> {
        match b {
            Balance::Inline(BalanceArgs { amt, .. }, _) => Ok(*amt),
            Balance::Onchain(h) => Ok(self.get_interim(owner, asset, h)),
            Balance::CommitLeftFilledToBal(c, _) => self.commit_left_amount_filled(owner, asset, c),
            Balance::CommitRightFilledToBal(c, _) => {
                self.commit_right_amount_filled(owner, asset, c)
            }
            Balance::Cancel(o, _) => self.order_from(owner, asset, o),
            Balance::Join(l, r, _) => self
                .balance_amount(owner, asset, l)?
                .checked_add(self.balance_amount(owner, asset, r)?)
                .ok_or(Error {
                    typ: ErrorDiscriminant::CheckedAdd,
                }),
        }
    }

    pub fn commit_left_amount_filled(
        &self,
        owner: Address,
        l_asset: Address,
        c: &Commit,
    ) -> R<u128> {
        match c {
            Commit::Inline(_, l, r, _) => Ok(u128::min(
                self.order_desired_amount(owner, l_asset, r)?,
                self.order_from(owner, l_asset, l)?,
            )),
            Commit::Onchain(h) => Ok(self.get_interim(owner, l_asset, h)),
        }
    }

    pub fn commit_right_amount_filled(
        &self,
        owner: Address,
        r_asset: Address,
        c: &Commit,
    ) -> R<u128> {
        match c {
            Commit::Inline(_, l, r, _) => Ok(u128::min(
                self.order_desired_amount(owner, r_asset, l)?,
                self.order_from(owner, r_asset, r)?,
            )),
            Commit::Onchain(h) => Ok(self.get_interim(owner, r_asset, h)),
        }
    }

    pub fn order_desired_amount(&self, owner: Address, asset: Address, c: &Order) -> R<u128> {
        match c {
            Order::Inline(OrderArgs { desired_amt, .. }, _, _) => Ok(*desired_amt),
            Order::Onchain(h) => Ok(self.get_hash_order_desired_amount(h)),
            Order::CommitLeftExcessToOrder(c, _) => {
                self.commit_left_amount_unfilled(owner, asset, c)
            }
            Order::CommitRightExcessToOrder(c, _) => {
                self.commit_right_amount_unfilled(owner, asset, c)
            }
        }
    }

    pub fn commit_left_amount_unfilled(
        &self,
        owner: Address,
        asset: Address,
        o: &Commit,
    ) -> R<u128> {
        match o {
            Commit::Inline(_, l, r, _) => checked_sub(
                self.order_from(owner, asset, l)?,
                self.order_desired_amount(owner, asset, r)?,
            ),
            Commit::Onchain(h) => {
                let owner = self.get_hash_owner_l(h);
                let asset = self.get_hash_asset_l(h);
                Ok(self.get_order(owner, asset, h))
            }
        }
    }

    pub fn commit_right_amount_unfilled(
        &self,
        owner: Address,
        asset: Address,
        o: &Commit,
    ) -> R<u128> {
        match o {
            Commit::Inline(_, l, r, _) => checked_sub(
                self.order_from(owner, asset, r)?,
                self.order_desired_amount(owner, asset, l)?,
            ),
            Commit::Onchain(h) => {
                let owner = self.get_hash_owner_r(h);
                let asset = self.get_hash_asset_r(h);
                Ok(self.get_order(owner, asset, h))
            }
        }
    }

    pub fn order_from(&self, owner: Address, asset: Address, o: &Order) -> R<u128> {
        match o {
            Order::Inline(OrderArgs { from_amt, .. }, _, _) => Ok(*from_amt),
            Order::Onchain(h) => Ok(self.get_order(owner, asset, h)),
            Order::CommitLeftExcessToOrder(c, _) => {
                self.commit_left_amount_unfilled(owner, asset, c)
            }
            Order::CommitRightExcessToOrder(c, _) => {
                self.commit_right_amount_unfilled(owner, asset, c)
            }
        }
    }

    pub fn order_underlying_amt(&self, owner: Address, asset: Address, o: &Order) -> R<u128> {
        match o {
            Order::Inline(_, b, _) => self.balance_amount(owner, asset, b),
            Order::Onchain(h) => Ok(self.get_order(owner, asset, h)),
            Order::CommitLeftExcessToOrder(c, _) => {
                self.commit_left_amount_unfilled(owner, asset, c)
            }
            Order::CommitRightExcessToOrder(c, _) => {
                self.commit_right_amount_unfilled(owner, asset, c)
            }
        }
    }

    pub fn order_hash<'a>(&self, o: &'a Order) -> &'a [u8; 64] {
        match o {
            Order::Inline(_, _, h)
            | Order::Onchain(h)
            | Order::CommitLeftExcessToOrder(_, h)
            | Order::CommitRightExcessToOrder(_, h) => h,
        }
    }

    pub fn balance_asset(&self, b: &Balance) -> Address {
        match b {
            Balance::Inline(BalanceArgs { asset, .. }, _) => *asset,
            Balance::Onchain(h) => self.get_hash_asset_l(h),
            Balance::CommitLeftFilledToBal(c, _) => self.commit_left_asset(c),
            Balance::CommitRightFilledToBal(c, _) => self.commit_right_asset(c),
            Balance::Join(l, _, _) => self.balance_asset(l),
            Balance::Cancel(o, _) => self.order_asset(o),
        }
    }

    pub fn commit_left_asset(&self, c: &Commit) -> Address {
        match c {
            Commit::Inline(_, l, _, _) => self.order_asset(l),
            Commit::Onchain(h) => self.get_hash_asset_l(h),
        }
    }

    pub fn commit_right_asset(&self, c: &Commit) -> Address {
        match c {
            Commit::Inline(_, _, r, _) => self.order_asset(r),
            Commit::Onchain(h) => self.get_hash_asset_r(h),
        }
    }

    pub fn order_asset(&self, o: &Order) -> Address {
        match o {
            Order::Inline(_, b, _) => self.balance_asset(b),
            Order::Onchain(h) => self.get_hash_asset_l(h),
            Order::CommitLeftExcessToOrder(c, _) => self.commit_left_asset(c),
            Order::CommitRightExcessToOrder(c, _) => self.commit_right_asset(c),
        }
    }

    pub fn commit_left_desired_asset(&self, c: &Commit) -> Address {
        match c {
            Commit::Inline(_, o, _, _) => self.order_desired_asset(o),
            Commit::Onchain(h) => self.get_hash_asset_r(h),
        }
    }

    pub fn commit_right_desired_asset(&self, c: &Commit) -> Address {
        match c {
            Commit::Inline(_, _, o, _) => self.order_desired_asset(o),
            Commit::Onchain(h) => self.get_hash_asset_l(h),
        }
    }

    pub fn order_desired_asset(&self, o: &Order) -> Address {
        match o {
            Order::Inline(OrderArgs { desired_asset, .. }, _, _) => *desired_asset,
            Order::Onchain(h) => self.get_hash_asset_r(h),
            Order::CommitLeftExcessToOrder(c, _) => self.commit_left_desired_asset(c),
            Order::CommitRightExcessToOrder(c, _) => self.commit_right_desired_asset(c),
        }
    }

    pub fn commit_hash<'a>(&self, c: &'a Commit) -> &'a [u8; 64] {
        match c {
            Commit::Inline(_, _, _, h) | Commit::Onchain(h) => h,
        }
    }

    pub fn withdraw_balance(
        &self,
        owner: Address,
        asset: Address,
        Withdraw::Inline(b, _): &Withdraw,
    ) -> R<u128> {
        self.balance_amount(owner, asset, b)
    }

    pub fn withdraw_owner(&self, Withdraw::Inline(b, _): &Withdraw) -> Address {
        self.balance_owner(b)
    }

    pub fn withdraw_asset(&self, Withdraw::Inline(b, _): &Withdraw) -> Address {
        self.balance_asset(b)
    }

    pub fn withdraw_hash<'a>(&self, Withdraw::Inline(_, h): &'a Withdraw) -> &'a [u8; 64] {
        h
    }

    pub fn apply_balance_inline(&mut self, b: &Balance) -> R<()> {
        let owner = self.balance_owner(b);
        let asset = self.balance_asset(b);
        let amt = self.balance_amount(owner, asset, b)?;
        let Balance::Inline(_, h) = b else {
            unreachable!();
        };
        self.increase_interim(owner, asset, h, amt)?;
        self.set_hash_details_l(h, owner, asset);
        self.decrease_withdrawal(owner, asset, amt)?;
        Ok(())
    }

    pub fn apply_balance_join(&mut self, l: &Balance, r: &Balance) -> R<()> {
        if self.balance_owner(l) != self.balance_owner(r) {
            return Err(Error {
                typ: ErrorDiscriminant::InconsistentOwners,
            });
        }
        self.apply_balance(l)?;
        self.apply_balance(r)?;
        Ok(())
    }

    pub fn apply_balance_cancel(&mut self, b: &Balance) -> R<()> {
        let owner = self.balance_owner(b);
        let asset = self.balance_asset(b);
        let amt = self.balance_amount(owner, asset, b)?;
        let Balance::Cancel(o, h) = b else {
            unreachable!();
        };
        self.apply_order(o)?;
        if amt == 0 {
            return Ok(());
        }
        self.set_hash_details_l(h, owner, asset);
        self.increase_interim(owner, asset, h, amt)?;
        self.decrease_order(owner, asset, h, amt)
    }

    pub fn apply_balance(&mut self, b: &Balance) -> R<()> {
        match b {
            Balance::Inline(_, _) => self.apply_balance_inline(b),
            Balance::Onchain(_) => Ok(()),
            Balance::CommitLeftFilledToBal(c, _) | Balance::CommitRightFilledToBal(c, _) => {
                self.apply_commit(c)
            }
            Balance::Cancel(_, _) => self.apply_balance_cancel(b),
            Balance::Join(l, r, _) => self.apply_balance_join(l, r),
        }
    }

    pub fn apply_commit(&mut self, c: &Commit) -> R<()> {
        let Commit::Inline(_, l, r,_) = c else {
            return Ok(());
        };
        let hash = self.commit_hash(c);
        let l_asset = self.order_asset(l);
        let l_hash = self.order_hash(l);
        let r_asset = self.order_asset(r);
        let r_hash = self.order_hash(r);
        let l_desired_asset = self.order_desired_asset(l);
        let r_desired_asset = self.order_desired_asset(r);
        let l_owner = self.order_owner(l);
        let r_owner = self.order_owner(r);
        let l_filled = self.commit_left_amount_filled(l_owner, l_asset, c)?;
        let r_filled = self.commit_right_amount_filled(r_owner, r_asset, c)?;
        if l_desired_asset == r_desired_asset {
            return Err(err_same_assets());
        }
        if l_desired_asset != r_asset || r_desired_asset != l_asset {
            return Err(err_bad_asset_asks());
        }
        self.apply_order(l)?;
        self.apply_order(r)?;
        self.decrease_order(l_owner, l_asset, l_hash, l_filled)?;
        self.decrease_order(r_owner, r_asset, r_hash, r_filled)?;
        self.increase_interim(l_owner, r_asset, hash, l_filled)?;
        self.increase_interim(r_owner, l_asset, hash, r_filled)?;
        self.set_hash_details_l(hash, l_owner, l_asset);
        self.set_hash_details_r(hash, r_owner, r_asset);
        Ok(())
    }

    pub fn apply_order(&mut self, o: &Order) -> R<()> {
        if let Order::Onchain(_) = o {
            // If it's the case that the order is already onchain, we don't want to
            // apply it again.
            return Ok(());
        }
        let from_asset = self.order_asset(o);
        let owner = self.order_owner(o);
        // This function should check the argument for the amount,
        // instead of the underlying balance.
        let amt = self.order_from(owner, from_asset, o)?;
        let bal_amt = self.order_underlying_amt(owner, from_asset, o)?;
        let desired_asset = self.order_desired_asset(o);
        let h = self.order_hash(o);
        match o {
            Order::Inline(_, b, _) => self.apply_balance(b)?,
            Order::Onchain(_) => (),
            Order::CommitLeftExcessToOrder(c, _) | Order::CommitRightExcessToOrder(c, _) => {
                self.apply_commit(c)?
            }
        };
        if from_asset == desired_asset {
            return Err(err_same_assets());
        }
        if bal_amt < amt {
            return Err(err_bad_balance_from_order());
        }
        self.increase_order(owner, from_asset, h, amt)?;
        self.decrease_interim(owner, from_asset, h, amt)?;
        self.set_hash_details_l(h, owner, from_asset);
        self.set_hash_details_desired_asset(h, desired_asset);
        Ok(())
    }

    pub fn apply_withdraw(&mut self, w: &Withdraw) -> R<()> {
        let owner = self.withdraw_owner(w);
        let asset = self.withdraw_asset(w);
        let amt = self.withdraw_balance(owner, asset, w)?;
        let hash = self.withdraw_hash(w);
        let Withdraw::Inline(b, _) = w;
        self.apply_balance(b)?;
        self.decrease_interim(owner, asset, hash, amt)?;
        self.increase_withdrawal(owner, asset, amt)?;
        call_eip20_extras::transfer(self, asset, owner, u128_to_u256(amt))
    }

    pub fn apply(&mut self, s: StateMachine) -> R<()> {
        match s {
            StateMachine::Balance(b) => self.apply_balance(&b),
            StateMachine::Commit(c) => self.apply_commit(&c),
            StateMachine::Order(o) => self.apply_order(&o),
            StateMachine::Withdraw(w) => self.apply_withdraw(&w),
        }
    }
}

fn u128_to_u256(x: u128) -> U256 {
    let mut b = [0u8; 32];
    b[16..].copy_from_slice(&x.to_be_bytes());
    U256::from_be_bytes(b)
}
