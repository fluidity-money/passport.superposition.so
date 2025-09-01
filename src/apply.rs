use stylus_sdk::alloy_primitives::Address;

use crate::{
    error::{Error, ErrorDiscriminant},
    state_machine::{
        Balance, BalanceArgs, Commit, CommitArgs, Order, OrderArgs, StateMachine, Withdraw,
    },
    storage::StoragePassport,
};

pub type R<T> = Result<T, Error>;

fn err_same_assets() -> Error {
    Error {
        typ: ErrorDiscriminant::SameAssets,
        cd: vec![],
    }
}

fn err_bad_asset_asks() -> Error {
    Error {
        typ: ErrorDiscriminant::BadAssetAsks,
        cd: vec![],
    }
}

pub fn commit_left_owner(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, o, _, _) => order_owner(o),
        Commit::Onchain(_) => todo!(),
    }
}

pub fn commit_right_owner(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, _, o, _) => order_owner(o),
        Commit::Onchain(_) => todo!(),
    }
}

pub fn order_owner(o: &Order) -> Address {
    match o {
        Order::Inline(_, b, _) => balance_owner(b),
        Order::Onchain(_) => todo!(),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_owner(c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_owner(c),
    }
}

pub fn balance_owner(b: &Balance) -> Address {
    match b {
        Balance::Inline(BalanceArgs { owner, .. }, _) => owner.clone(),
        Balance::Onchain(_) => todo!(),
        Balance::CommitLeftFilledToBal(c, _) => commit_left_owner(c),
        Balance::CommitRightFilledToBal(c, _) => commit_right_owner(c),
        Balance::Cancel(o, _) => order_owner(o),
        Balance::Join(b, _, _) => balance_owner(b),
    }
}

pub fn bal_amount(b: &Balance) -> R<u128> {
    match b {
        Balance::Inline(BalanceArgs { amt, .. }, _) => Ok(*amt),
        Balance::Onchain(_) => todo!(),
        Balance::CommitLeftFilledToBal(c, _) => commit_left_amount_filled(c),
        Balance::CommitRightFilledToBal(c, _) => commit_right_amount_filled(c),
        Balance::Cancel(o, _) => order_amount(o),
        Balance::Join(b, _, _) => balance_amount(b),
    }
}

pub fn commit_left_amount_filled(c: &Commit) -> R<u128> {
    match c {
        Commit::Inline(_, l, r, _) => Ok(u128::min(order_desired_amount(r)?, order_amount(l)?)),
        Commit::Onchain(_) => todo!(),
    }
}

pub fn commit_right_amount_filled(c: &Commit) -> R<u128> {
    match c {
        Commit::Inline(_, l, r, _) => Ok(u128::min(order_desired_amount(l)?, order_amount(r)?)),
        Commit::Onchain(_) => todo!(),
    }
}

pub fn order_desired_amount(c: &Order) -> R<u128> {
    match c {
        Order::Inline(OrderArgs { desired_amt, .. }, _, _) => Ok(*desired_amt),
        Order::Onchain(_) => todo!(),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_amount_unfilled(c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_amount_unfilled(c),
    }
}

fn checked_sub(x: u128, y: u128) -> R<u128> {
    x.checked_sub(y).ok_or(Error {
        typ: ErrorDiscriminant::CheckedSub(x, y),
        cd: vec![],
    })
}

pub fn commit_left_amount_unfilled(o: &Commit) -> R<u128> {
    match o {
        Commit::Inline(_, l, r, _) => checked_sub(order_amount(l)?, order_desired_amount(r)?),
        Commit::Onchain(_) => todo!(),
    }
}

pub fn commit_right_amount_unfilled(o: &Commit) -> R<u128> {
    match o {
        Commit::Inline(_, l, r, _) => checked_sub(order_amount(r)?, order_desired_amount(l)?),
        Commit::Onchain(_) => todo!(),
    }
}

pub fn order_amount(o: &Order) -> R<u128> {
    match o {
        // TODO: let the user supply the balance to spend from the ticket.
        Order::Inline(_, b, _) => balance_amount(b),
        Order::Onchain(_) => todo!(),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_amount_unfilled(c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_amount_unfilled(c),
    }
}

pub fn balance_amount(b: &Balance) -> R<u128> {
    match b {
        Balance::Inline(BalanceArgs { amt, .. }, _) => Ok(*amt),
        Balance::Onchain(_) => todo!(),
        Balance::CommitLeftFilledToBal(c, _) => commit_left_amount_filled(c),
        Balance::CommitRightFilledToBal(c, _) => commit_right_amount_filled(c),
        Balance::Join(l, r, _) => Ok(balance_amount(l)? + balance_amount(r)?),
        Balance::Cancel(o, _) => order_amount(o),
    }
}

pub fn commit_timestamp(c: &Commit) -> u128 {
    match c {
        Commit::Inline(CommitArgs { ms_ts }, _, _, _) => *ms_ts,
        Commit::Onchain(_) => todo!(),
    }
}

pub fn balance_timestamp(b: &Balance) -> u128 {
    match b {
        Balance::Inline(BalanceArgs { ms_ts, .. }, _) => *ms_ts,
        Balance::Onchain(_) => todo!(),
        Balance::CommitLeftFilledToBal(c, _) | Balance::CommitRightFilledToBal(c, _) => {
            commit_timestamp(c)
        }
        Balance::Join(l, _, _) => balance_timestamp(l),
        Balance::Cancel(o, _) => order_timestamp(o),
    }
}

pub fn order_timestamp(o: &Order) -> u128 {
    match o {
        Order::Inline(_, b, _) => balance_timestamp(b),
        Order::Onchain(_) => todo!(),
        Order::CommitLeftExcessToOrder(c, _) | Order::CommitRightExcessToOrder(c, _) => {
            commit_timestamp(c)
        }
    }
}

pub fn balance_asset(b: &Balance) -> Address {
    match b {
        Balance::Inline(BalanceArgs { asset, .. }, _) => *asset,
        Balance::Onchain(_) => todo!(),
        Balance::CommitLeftFilledToBal(c, _) => commit_left_asset(c),
        Balance::CommitRightFilledToBal(c, _) => commit_right_asset(c),
        Balance::Join(l, _, _) => balance_asset(l),
        Balance::Cancel(o, _) => order_asset(o),
    }
}

pub fn commit_left_asset(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, l, _, _) => order_asset(l),
        Commit::Onchain(_) => todo!(),
    }
}

pub fn commit_right_asset(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, _, r, _) => order_asset(r),
        Commit::Onchain(_) => todo!(),
    }
}

pub fn order_asset(o: &Order) -> Address {
    match o {
        Order::Inline(_, b, _) => balance_asset(b),
        Order::Onchain(_) => todo!(),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_asset(c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_asset(c),
    }
}

pub fn commit_left_desired_asset(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, o, _, _) => order_desired_asset(o),
        Commit::Onchain(_) => todo!(),
    }
}

pub fn commit_right_desired_asset(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, _, o, _) => order_desired_asset(o),
        Commit::Onchain(_) => todo!(),
    }
}

pub fn order_desired_asset(o: &Order) -> Address {
    match o {
        Order::Inline(OrderArgs { desired_asset, .. }, _, _) => *desired_asset,
        Order::Onchain(_) => todo!(),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_desired_asset(c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_desired_asset(c),
    }
}

impl StoragePassport {
    pub fn apply_balance_inline(&mut self, b: &Balance) -> R<()> {
        let owner = balance_owner(b);
        let ts = balance_timestamp(b);
        let amt = balance_amount(b)?;
        let asset = balance_asset(b);
        self.increase_interim(owner, asset, ts, amt)?;
        // To increase the user's interim amount, we do so if the user made a
        // deposit and created the Balance::Inline object (after a Withdrawal).
        let Balance::Inline(_, _) = b else {
            unreachable!();
        };
        self.decrease_withdrawal(owner, asset, amt)?;
        Ok(())
    }

    pub fn apply_balance_cancel(&mut self, b: &Balance) -> R<()> {
        let amt = balance_amount(b)?;
        let owner = balance_owner(b);
        let asset = balance_asset(b);
        let ts = balance_timestamp(b);
        let Balance::Cancel(o, _) = b else {
            unreachable!();
        };
        self.apply_order(o)?;
        if amt == 0 {
            return Ok(());
        }
        self.increase_interim(owner, asset, ts, amt)?;
        self.decrease_order(owner, asset, ts, amt)
    }

    pub fn apply_balance(&mut self, b: &Balance) -> R<()> {
        match b {
            Balance::Inline(_, _) => self.apply_balance_inline(b),
            Balance::Onchain(_) => todo!(),
            Balance::CommitLeftFilledToBal(c, _) | Balance::CommitRightFilledToBal(c, _) => {
                self.apply_commit(c)
            }
            Balance::Cancel(_, _) => self.apply_balance_cancel(b),
            Balance::Join(_, _, _) => todo!(),
        }
    }

    pub fn apply_inline_commit(&mut self, c: &Commit) -> R<()> {
        let Commit::Inline(CommitArgs { ms_ts }, l, r, _) = c else {
            unreachable!();
        };
        let l_asset = order_asset(l);
        let r_asset = order_asset(r);
        let l_desired_asset = order_desired_asset(l);
        let r_desired_asset = order_desired_asset(r);
        let l_filled = commit_left_amount_filled(c)?;
        let r_filled = commit_right_amount_filled(c)?;
        let l_owner = order_owner(l);
        let r_owner = order_owner(r);
        if l_desired_asset == r_desired_asset {
            return Err(err_same_assets());
        }
        if l_desired_asset != r_asset || r_desired_asset != l_asset {
            return Err(err_bad_asset_asks());
        }
        self.apply_order(l)?;
        self.apply_order(r)?;
        self.decrease_order(l_owner, l_asset, order_timestamp(l), l_filled)?;
        self.decrease_order(r_owner, r_asset, order_timestamp(r), r_filled)?;
        self.increase_interim(l_owner, r_asset, *ms_ts, l_filled)?;
        self.increase_interim(r_owner, l_asset, *ms_ts, r_filled)
    }

    pub fn apply_inline_order(&mut self, o: &Order) -> R<()> {
        let from_asset = order_asset(o);
        let owner = order_owner(o);
        let amt = order_amount(o)?;
        let desired_asset = order_desired_asset(o);
        let ts = order_timestamp(o);
        let Order::Inline(_, b, _) = o else {
            unreachable!();
        };
        self.apply_balance(b)?;
        if from_asset == desired_asset {
            return Err(err_same_assets());
        }
        self.increase_order(owner, from_asset, ts, amt)?;
        self.decrease_interim(owner, from_asset, ts, amt)
    }

    pub fn apply_commit(&mut self, c: &Commit) -> R<()> {
        match c {
            Commit::Inline(_, _, _, _) => self.apply_inline_commit(c),
            Commit::Onchain(_) => todo!(),
        }
    }

    pub fn apply_order(&mut self, o: &Order) -> R<()> {
        match o {
            Order::Inline(_, _, _) => self.apply_inline_order(o),
            Order::Onchain(_) => todo!(),
            Order::CommitLeftExcessToOrder(c, _) | Order::CommitRightExcessToOrder(c, _) => {
                self.apply_commit(c)
            }
        }
    }

    pub fn apply_inline_withdraw(&mut self, w: &Withdraw) -> R<()> {
        let Withdraw::Inline(b, _) = w else {
            unreachable!()
        };
        let amt = balance_amount(b)?;
        let owner = balance_owner(b);
        let asset = balance_asset(b);
        let ts = balance_timestamp(b);
        self.apply_balance(b)?;
        self.decrease_interim(owner, asset, ts, amt)?;
        self.increase_withdrawal(owner, asset, amt)
    }

    pub fn apply_withdraw(&mut self, w: &Withdraw) -> R<()> {
        match w {
            Withdraw::Inline(_, _) => self.apply_inline_withdraw(w),
            Withdraw::Onchain(_) => todo!(),
        }
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
