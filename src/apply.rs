use crate::error::{Error, ErrorDiscriminant};

use bobcat_sdk::maths::U;

type Address = [u8; 20];

use crate::{
    call_eip20_extras,
    error::ApplyContext,
    state_machine::{Balance, BalanceArgs, Commit, Order, OrderArgs, StateMachine, Withdraw},
    storage::*,
};

pub type R<T> = Result<T, Error>;

pub fn err_same_assets() -> Error {
    Error::from(ErrorDiscriminant::SameAssets)
}

pub fn err_bad_asset_asks() -> Error {
    Error::from(ErrorDiscriminant::BadAssetAsks)
}

pub fn err_bad_balance_from_order() -> Error {
    Error::from(ErrorDiscriminant::BalanceTransitionToOrderBad)
}

pub fn commit_left_owner(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, o, _, _) => order_owner(o),
        Commit::Onchain(h) => get_hash_owner_l(h),
    }
}

pub fn commit_right_owner(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, _, o, _) => order_owner(o),
        Commit::Onchain(h) => get_hash_owner_r(h),
    }
}

pub fn order_owner(o: &Order) -> Address {
    match o {
        Order::Inline(_, b, _) => balance_owner(b),
        Order::Onchain(h) => get_hash_owner_l(h),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_owner(c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_owner(c),
    }
}

pub fn balance_owner(b: &Balance) -> Address {
    match b {
        Balance::Inline(BalanceArgs { owner, .. }, _) => *owner,
        Balance::Onchain(h) => get_hash_owner_l(h),
        Balance::CommitLeftFilledToBal(c, _) => commit_left_owner(c),
        Balance::CommitRightFilledToBal(c, _) => commit_right_owner(c),
        Balance::Cancel(o, _) => order_owner(o),
        Balance::Join(b, _, _) => balance_owner(b),
    }
}

pub fn balance_amount(owner: Address, asset: Address, b: &Balance) -> R<u128> {
    match b {
        Balance::Inline(BalanceArgs { amt, .. }, _) => Ok(*amt),
        Balance::Onchain(h) => Ok(get_interim_amount_hash(&owner, &asset, h)),
        Balance::CommitLeftFilledToBal(c, _) => commit_left_amount_filled(owner, asset, c),
        Balance::CommitRightFilledToBal(c, _) => commit_right_amount_filled(owner, asset, c),
        Balance::Cancel(o, _) => order_from(owner, asset, o),
        Balance::Join(l, r, _) => {
            let l_bal_amt = balance_amount(owner, asset, l)?;
            let r_bal_amt = balance_amount(owner, asset, r)?;
            l_bal_amt.checked_add(r_bal_amt).ok_or(
                Error::from(ErrorDiscriminant::CheckedAdd)
                    .ctx(ApplyContext::ApplyCommitBalanceAmount)
                    .x(U::from(l_bal_amt))
                    .y(U::from(r_bal_amt)),
            )
        }
    }
}

pub fn commit_left_amount_filled(owner: Address, l_asset: Address, c: &Commit) -> R<u128> {
    match c {
        Commit::Inline(_, l, r, _) => Ok(u128::min(
            order_desired_amount(owner, l_asset, r)?,
            order_from(owner, l_asset, l)?,
        )),
        Commit::Onchain(h) => Ok(get_interim_amount_hash(&owner, &l_asset, &h)),
    }
}

pub fn commit_right_amount_filled(owner: Address, r_asset: Address, c: &Commit) -> R<u128> {
    match c {
        Commit::Inline(_, l, r, _) => Ok(u128::min(
            order_desired_amount(owner, r_asset, l)?,
            order_from(owner, r_asset, r)?,
        )),
        Commit::Onchain(h) => Ok(get_interim_amount_hash(&owner, &r_asset, &h)),
    }
}

pub fn order_desired_amount(owner: Address, asset: Address, c: &Order) -> R<u128> {
    match c {
        Order::Inline(OrderArgs { desired_amt, .. }, _, _) => Ok(*desired_amt),
        Order::Onchain(h) => Ok(get_details_hash_order_desired_amt_hash(&h)),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_amount_unfilled(owner, asset, c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_amount_unfilled(owner, asset, c),
    }
}

pub fn commit_left_amount_unfilled(owner: Address, asset: Address, o: &Commit) -> R<u128> {
    match o {
        Commit::Inline(_, l, r, _) => {
            // FIXME: SATURATING SUB NEEDED
            Ok(order_desired_amount(owner, asset, r)?
                .checked_sub(order_from(owner, asset, l)?)
                .unwrap())
        }
        Commit::Onchain(h) => {
            let owner = get_hash_owner_l(h);
            let asset = get_details_hash_asset_l_hash(h);
            Ok(get_order_amt_hash(&owner, &asset, h))
        }
    }
}

pub fn commit_right_amount_unfilled(owner: Address, asset: Address, o: &Commit) -> R<u128> {
    match o {
        // FIXME: SATURATING SUB
        Commit::Inline(_, l, r, _) => Ok(order_desired_amount(owner, asset, l)?
            .checked_sub(order_from(owner, asset, r)?)
            .unwrap()),
        Commit::Onchain(h) => {
            let owner = get_hash_owner_r(h);
            let asset = get_details_hash_asset_r_hash(h);
            Ok(get_order_amt_hash(&owner, &asset, h))
        }
    }
}

pub fn order_from(owner: Address, asset: Address, o: &Order) -> R<u128> {
    match o {
        Order::Inline(OrderArgs { from_amt, .. }, _, _) => Ok(*from_amt),
        Order::Onchain(h) => Ok(get_order_amt_hash(&owner, &asset, h)),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_amount_unfilled(owner, asset, c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_amount_unfilled(owner, asset, c),
    }
}

pub fn order_underlying_amt(owner: Address, asset: Address, o: &Order) -> R<u128> {
    match o {
        Order::Inline(_, b, _) => balance_amount(owner, asset, b),
        Order::Onchain(h) => Ok(get_order_amt_hash(&owner, &asset, h)),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_amount_unfilled(owner, asset, c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_amount_unfilled(owner, asset, c),
    }
}

pub fn order_hash<'a>(o: &'a Order) -> &'a [u8; 64] {
    match o {
        Order::Inline(_, _, h)
        | Order::Onchain(h)
        | Order::CommitLeftExcessToOrder(_, h)
        | Order::CommitRightExcessToOrder(_, h) => h,
    }
}

pub fn balance_asset(b: &Balance) -> Address {
    match b {
        Balance::Inline(BalanceArgs { asset, .. }, _) => *asset,
        Balance::Onchain(h) => get_details_hash_asset_l_hash(h),
        Balance::CommitLeftFilledToBal(c, _) => commit_right_asset(c),
        Balance::CommitRightFilledToBal(c, _) => commit_left_asset(c),
        Balance::Join(l, _, _) => balance_asset(l),
        Balance::Cancel(o, _) => order_asset(o),
    }
}

pub fn commit_left_asset(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, l, _, _) => order_asset(l),
        Commit::Onchain(h) => get_details_hash_asset_l_hash(h),
    }
}

pub fn commit_right_asset(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, _, r, _) => order_asset(r),
        Commit::Onchain(h) => get_details_hash_asset_r_hash(h),
    }
}

pub fn order_asset(o: &Order) -> Address {
    match o {
        Order::Inline(_, b, _) => balance_asset(b),
        Order::Onchain(h) => get_details_hash_asset_l_hash(h),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_asset(c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_asset(c),
    }
}

pub fn commit_left_desired_asset(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, o, _, _) => order_desired_asset(o),
        Commit::Onchain(h) => get_details_hash_asset_r_hash(h),
    }
}

pub fn commit_right_desired_asset(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, _, o, _) => order_desired_asset(o),
        Commit::Onchain(h) => get_details_hash_asset_l_hash(h),
    }
}

pub fn order_desired_asset(o: &Order) -> Address {
    match o {
        Order::Inline(OrderArgs { desired_asset, .. }, _, _) => *desired_asset,
        Order::Onchain(h) => get_details_hash_asset_r_hash(h),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_desired_asset(c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_desired_asset(c),
    }
}

pub fn commit_hash<'a>(c: &'a Commit) -> &'a [u8; 64] {
    match c {
        Commit::Inline(_, _, _, h) | Commit::Onchain(h) => h,
    }
}

pub fn withdraw_balance(
    owner: Address,
    asset: Address,
    Withdraw::Inline(b, _): &Withdraw,
) -> R<u128> {
    balance_amount(owner, asset, b)
}

pub fn withdraw_owner(Withdraw::Inline(b, _): &Withdraw) -> Address {
    balance_owner(b)
}

pub fn withdraw_asset(Withdraw::Inline(b, _): &Withdraw) -> Address {
    balance_asset(b)
}

pub fn withdraw_hash<'a>(Withdraw::Inline(_, h): &'a Withdraw) -> &'a [u8; 64] {
    h
}

pub fn balance_inline(b: &Balance) -> R<()> {
    let owner = balance_owner(b);
    let asset = balance_asset(b);
    let amt = balance_amount(owner, asset, b)?;
    let Balance::Inline(_, h) = b else {
        unreachable!();
    };
    let ctx = ApplyContext::ApplyBalanceInline;
    increase_interim_amount(ctx, &owner, &asset, h, &U::from(amt))?;
    set_details_hash_asset_l_hash(h, &asset);
    decrease_withdrawable(ctx, &owner, &asset, &U::from(amt))?;
    Ok(())
}

pub fn balance_join(l: &Balance, r: &Balance) -> R<()> {
    if balance_owner(l) != balance_owner(r) {
        return Err(Error::from(ErrorDiscriminant::InconsistentOwners));
    }
    balance(l)?;
    balance(r)?;
    Ok(())
}

pub fn balance_cancel(b: &Balance) -> R<()> {
    let owner = balance_owner(b);
    let asset = balance_asset(b);
    let amt = balance_amount(owner, asset, b)?;
    let Balance::Cancel(o, h) = b else {
        unreachable!();
    };
    order(o)?;
    if amt == 0 {
        return Ok(());
    }
    set_details_hash_asset_l_hash(h, &asset);
    let ctx = ApplyContext::ApplyBalanceCancel;
    let amt = U::from(amt);
    increase_interim_amount(ctx, &owner, &asset, &h, &amt)?;
    decrease_order_amount(ctx, &owner, &asset, h, &amt)?;
    Ok(())
}

pub fn balance(b: &Balance) -> R<()> {
    match b {
        Balance::Inline(_, _) => balance_inline(b),
        Balance::Onchain(_) => Ok(()),
        Balance::CommitLeftFilledToBal(c, _) | Balance::CommitRightFilledToBal(c, _) => commit(c),
        Balance::Cancel(_, _) => balance_cancel(b),
        Balance::Join(l, r, _) => balance_join(l, r),
    }
}

pub fn commit(c: &Commit) -> R<()> {
    let Commit::Inline(_, l, r, _) = c else {
        return Ok(());
    };
    let hash = commit_hash(c);
    let l_asset = order_asset(l);
    let l_hash = order_hash(l);
    let r_asset = order_asset(r);
    let r_hash = order_hash(r);
    let l_desired_asset = order_desired_asset(l);
    let r_desired_asset = order_desired_asset(r);
    let l_owner = order_owner(l);
    let r_owner = order_owner(r);
    let l_filled = commit_left_amount_filled(l_owner, l_asset, c)?;
    let r_filled = commit_right_amount_filled(r_owner, r_asset, c)?;
    if l_desired_asset == r_desired_asset {
        return Err(err_same_assets());
    }
    if l_desired_asset != r_asset || r_desired_asset != l_asset {
        return Err(err_bad_asset_asks());
    }
    order(l)?;
    order(r)?;
    let ctx = ApplyContext::ApplyCommit;
    let l_filled = U::from(l_filled);
    let r_filled = U::from(r_filled);
    decrease_order_amount(ctx, &l_owner, &l_asset, l_hash, &l_filled)?;
    decrease_order_amount(ctx, &r_owner, &r_asset, r_hash, &r_filled)?;
    increase_interim_amount(ctx, &l_owner, &r_asset, hash, &l_filled)?;
    increase_interim_amount(ctx, &r_owner, &l_asset, hash, &r_filled)?;
    set_details_hash_asset_l_hash(hash, &l_asset);
    set_details_hash_asset_r_hash(hash, &r_asset);
    set_hash_owner_r(hash, &r_owner);
    Ok(())
}

pub fn order(o: &Order) -> R<()> {
    if let Order::Onchain(_) = o {
        // If it's the case that the order is already onchain, we don't want to
        // it again.
        return Ok(());
    }
    let from_asset = order_asset(o);
    let owner = order_owner(o);
    // This function should check the argument for the amount,
    // instead of the underlying balance.
    let amt = order_from(owner, from_asset, o)?;
    let bal_amt = order_underlying_amt(owner, from_asset, o)?;
    let desired_asset = order_desired_asset(o);
    let h = order_hash(o);
    let ctx = ApplyContext::ApplyOrder;
    match o {
        Order::Inline(_, b, _) => {
            let b_hash = match **b {
                Balance::Inline(_, h)
                | Balance::Onchain(h)
                | Balance::CommitLeftFilledToBal(_, h)
                | Balance::CommitRightFilledToBal(_, h)
                | Balance::Cancel(_, h)
                | Balance::Join(_, _, h) => h,
            };
            balance(b)?;
            decrease_interim_amount(ctx, &owner, &from_asset, &b_hash, &U::from(amt))?;
            increase_order_amount(ctx, &owner, &from_asset, h, &U::from(amt))?;
        }
        Order::Onchain(_) => (),
        Order::CommitLeftExcessToOrder(c, _) | Order::CommitRightExcessToOrder(c, _) => {
            // The commit step already applies an order for us!
            commit(c)?
        }
    };
    if from_asset == desired_asset {
        return Err(err_same_assets());
    }
    if bal_amt < amt {
        return Err(err_bad_balance_from_order());
    }
    /* set_details_hash_asset_l(h, from_asset); FIXME */
    /* set_hash_details_desired_asset(h, desired_asset); */
    Ok(())
}

pub fn withdraw(w: &Withdraw) -> R<()> {
    let owner = withdraw_owner(w);
    let asset = withdraw_asset(w);
    let amt = withdraw_balance(owner, asset, w)?;
    let hash = withdraw_hash(w);
    let Withdraw::Inline(b, _) = w;
    balance(b)?;
    let ctx = ApplyContext::ApplyWithdraw;
    decrease_interim_amount(ctx, &owner, &asset, hash, &U::from(amt))?;
    increase_withdrawable(ctx, &owner, &asset, &U::from(amt))?;
    call_eip20_extras::transfer(asset, owner, U::from(amt))
}

pub fn apply(s: StateMachine) -> R<()> {
    match s {
        StateMachine::Balance(b) => balance(&b),
        StateMachine::Commit(c) => commit(&c),
        StateMachine::Order(o) => order(&o),
        StateMachine::Withdraw(w) => withdraw(&w),
    }
}
