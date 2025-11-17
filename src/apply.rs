use crate::error::{Error, ErrorDiscriminant};

use bobcat_sdk::maths::U;

type Address = [u8; 20];

use crate::{
    call_eip20_extras,
    error::ApplyContext,
    state_machine::{Balance, BalanceArgs, Commit, Order, OrderArgs, StateMachine, Withdraw},
    storage,
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
        Commit::Onchain(h) => storage::hash_owner_l::get_hash(h).into(),
    }
}

pub fn commit_right_owner(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, _, o, _) => order_owner(o),
        Commit::Onchain(h) => storage::hash_owner_r::get_hash(h).into(),
    }
}

pub fn order_owner(o: &Order) -> Address {
    match o {
        Order::Inline(_, b, _) => balance_owner(b),
        Order::Onchain(h) => storage::hash_owner_l::get_hash(h).into(),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_owner(c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_owner(c),
    }
}

pub fn balance_owner(b: &Balance) -> Address {
    match b {
        Balance::Inline(BalanceArgs { owner, .. }, _) => *owner,
        Balance::Onchain(h) => storage::hash_owner_l::get_hash(h).into(),
        Balance::CommitLeftFilledToBal(c, _) => commit_left_owner(c),
        Balance::CommitRightFilledToBal(c, _) => commit_right_owner(c),
        Balance::Cancel(o, _) => order_owner(o),
        Balance::Join(b, _, _) => balance_owner(b),
    }
}

pub fn balance_amount(owner: Address, asset: Address, b: &Balance) -> R<u128> {
    match b {
        Balance::Inline(BalanceArgs { amt, .. }, _) => Ok(*amt),
        Balance::Onchain(h) => {
            Ok(storage::interim_amt::get_hash(&owner.into(), &asset.into(), h).into())
        }
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
        Commit::Onchain(h) => {
            Ok(storage::interim_amt::get_hash(&owner.into(), &l_asset.into(), h).into())
        }
    }
}

pub fn commit_right_amount_filled(owner: Address, r_asset: Address, c: &Commit) -> R<u128> {
    match c {
        Commit::Inline(_, l, r, _) => Ok(u128::min(
            order_desired_amount(owner, r_asset, l)?,
            order_from(owner, r_asset, r)?,
        )),
        Commit::Onchain(h) => {
            Ok(storage::interim_amt::get_hash(&owner.into(), &r_asset.into(), h).into())
        }
    }
}

pub fn order_desired_amount(owner: Address, asset: Address, c: &Order) -> R<u128> {
    match c {
        Order::Inline(OrderArgs { desired_amt, .. }, _, _) => Ok(*desired_amt),
        Order::Onchain(h) => Ok(storage::hash_desired_amt::get_hash(&h).into()),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_amount_unfilled(owner, asset, c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_amount_unfilled(owner, asset, c),
    }
}

pub fn commit_left_amount_unfilled(owner: Address, asset: Address, o: &Commit) -> R<u128> {
    match o {
        Commit::Inline(_, l, r, _) => {
            Ok(order_desired_amount(owner, asset, r)?.saturating_sub(order_from(owner, asset, l)?))
        }
        Commit::Onchain(h) => {
            let owner = storage::hash_owner_l::get_hash(h);
            let asset = storage::hash_asset_l::get_hash(h);
            Ok(storage::order_amt::get_hash(&owner, &asset, h).into())
        }
    }
}

pub fn commit_right_amount_unfilled(owner: Address, asset: Address, o: &Commit) -> R<u128> {
    match o {
        Commit::Inline(_, l, r, _) => {
            Ok(order_desired_amount(owner, asset, l)?.saturating_sub(order_from(owner, asset, r)?))
        }
        Commit::Onchain(h) => {
            let owner = storage::hash_owner_r::get_hash(h);
            let asset = storage::hash_asset_r::get_hash(h);
            Ok(storage::order_amt::get_hash(&owner, &asset, h).into())
        }
    }
}

pub fn order_from(owner: Address, asset: Address, o: &Order) -> R<u128> {
    match o {
        Order::Inline(OrderArgs { from_amt, .. }, _, _) => Ok(*from_amt),
        Order::Onchain(h) => {
            Ok(storage::order_amt::get_hash(&owner.into(), &asset.into(), h).into())
        }
        Order::CommitLeftExcessToOrder(c, _) => commit_left_amount_unfilled(owner, asset, c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_amount_unfilled(owner, asset, c),
    }
}

pub fn order_underlying_amt(owner: Address, asset: Address, o: &Order) -> R<u128> {
    match o {
        Order::Inline(_, b, _) => balance_amount(owner, asset, b),
        Order::Onchain(h) => {
            Ok(storage::order_amt::get_hash(&owner.into(), &asset.into(), h).into())
        }
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
        Balance::Onchain(h) => storage::hash_asset_l::get_hash(h).into(),
        Balance::CommitLeftFilledToBal(c, _) => commit_right_asset(c),
        Balance::CommitRightFilledToBal(c, _) => commit_left_asset(c),
        Balance::Join(l, _, _) => balance_asset(l),
        Balance::Cancel(o, _) => order_asset(o),
    }
}

pub fn balance_hash<'a>(b: &'a Balance) -> &'a [u8; 64] {
    match b {
        Balance::Inline(_, h)
        | Balance::Onchain(h)
        | Balance::CommitLeftFilledToBal(_, h)
        | Balance::CommitRightFilledToBal(_, h)
        | Balance::Join(_, _, h)
        | Balance::Cancel(_, h) => h,
    }
}

pub fn commit_left_asset(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, l, _, _) => order_asset(l),
        Commit::Onchain(h) => storage::hash_asset_l::get_hash(h).into(),
    }
}

pub fn commit_right_asset(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, _, r, _) => order_asset(r),
        Commit::Onchain(h) => storage::hash_asset_r::get_hash(h).into(),
    }
}

pub fn order_asset(o: &Order) -> Address {
    match o {
        Order::Inline(_, b, _) => balance_asset(b),
        Order::Onchain(h) => storage::hash_asset_l::get_hash(h).into(),
        Order::CommitLeftExcessToOrder(c, _) => commit_left_asset(c),
        Order::CommitRightExcessToOrder(c, _) => commit_right_asset(c),
    }
}

pub fn commit_left_desired_asset(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, o, _, _) => order_desired_asset(o),
        Commit::Onchain(h) => storage::hash_asset_r::get_hash(h).into(),
    }
}

pub fn commit_right_desired_asset(c: &Commit) -> Address {
    match c {
        Commit::Inline(_, _, o, _) => order_desired_asset(o),
        Commit::Onchain(h) => storage::hash_asset_l::get_hash(h).into(),
    }
}

pub fn order_desired_asset(o: &Order) -> Address {
    match o {
        Order::Inline(OrderArgs { desired_asset, .. }, _, _) => *desired_asset,
        Order::Onchain(h) => storage::hash_asset_r::get_hash(h).into(),
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

pub fn balance_inline(b: &Balance) -> R<()> {
    let owner = balance_owner(b);
    let asset = balance_asset(b);
    let amt = balance_amount(owner, asset, b)?;
    let Balance::Inline(_, h) = b else {
        unreachable!();
    };
    let h: [u8; 32] = h[..32].try_into().unwrap();
    storage::interim_amt::add(&owner.into(), &asset.into(), &U(h), &U::from(amt));
    storage::hash_asset_l::set(&U(h), &asset.into());
    storage::withdrawable::sub(&owner.into(), &asset.into(), &U::from(amt));
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
    let order_hash = order_hash(o);
    if amt == 0 {
        return Ok(());
    }
    storage::hash_asset_l::set(&U(h[..32].try_into().unwrap()), &asset.into());
    let amt = U::from(amt);
    storage::interim_amt::add(
        &owner.into(),
        &asset.into(),
        &U(h[..32].try_into().unwrap()),
        &amt,
    );
    storage::order_amt::sub(
        &owner.into(),
        &asset.into(),
        &U(order_hash[..32].try_into().unwrap()),
        &amt,
    );
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
    let hash = U(commit_hash(c)[..32].try_into().unwrap());
    let l_asset = order_asset(l);
    let l_hash: U = U(order_hash(l)[..32].try_into().unwrap());
    let r_asset = order_asset(r);
    let r_hash: U = U(order_hash(r)[..32].try_into().unwrap());
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
    let l_filled = U::from(l_filled);
    let r_filled = U::from(r_filled);
    storage::order_amt::sub(&l_owner.into(), &l_asset.into(), &l_hash, &l_filled.into());
    storage::order_amt::sub(&r_owner.into(), &r_asset.into(), &r_hash, &r_filled.into());
    storage::interim_amt::add(&l_owner.into(), &r_asset.into(), &hash, &l_filled);
    storage::interim_amt::add(&r_owner.into(), &l_asset.into(), &hash, &r_filled);
    storage::hash_asset_l::set(&hash, &l_asset.into());
    storage::hash_asset_r::set(&hash, &r_asset.into());
    storage::hash_owner_r::set(&hash, &r_owner.into());
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
    let h: U = U(order_hash(o)[..32].try_into().unwrap());
    dbg!("ORDER HASH", h);
    match o {
        Order::Inline(_, b, _) => {
            let b_hash: U = match **b {
                Balance::Inline(_, h)
                | Balance::Onchain(h)
                | Balance::CommitLeftFilledToBal(_, h)
                | Balance::CommitRightFilledToBal(_, h)
                | Balance::Cancel(_, h)
                | Balance::Join(_, _, h) => U(h[..32].try_into().unwrap()),
            };
            balance(b)?;
            storage::interim_amt::sub(&owner.into(), &from_asset.into(), &b_hash, &U::from(amt));
            storage::order_amt::add(&owner.into(), &from_asset.into(), &h, &U::from(amt));
        }
        Order::Onchain(_) => (),
        Order::CommitLeftExcessToOrder(c, _) => {
            // The commit step already applies an order for us!
            commit(c)?;
            let owner = commit_left_owner(c);
            let asset = commit_left_asset(c);
            let amt = commit_left_amount_unfilled(owner, asset, c)?;
            // But we do need to lift the interim balances to an order amount for this side:
            storage::order_amt::add(&owner.into(), &asset.into(), &h, &U::from(amt));
        }
        Order::CommitRightExcessToOrder(c, _) => {
            commit(c)?;
            let owner = commit_right_owner(c);
            let asset = commit_right_asset(c);
            let amt = commit_right_amount_unfilled(owner, asset, c)?;
            storage::order_amt::add(&owner.into(), &asset.into(), &h, &U::from(amt));
        }
    };
    if from_asset == desired_asset {
        return Err(err_same_assets());
    }
    if bal_amt < amt {
        return Err(err_bad_balance_from_order());
    }
    storage::hash_asset_l::set(&h, &from_asset.into());
    storage::hash_asset_r::set(&h, &desired_asset.into());
    Ok(())
}

pub fn withdraw(w: &Withdraw) -> R<()> {
    let owner = withdraw_owner(w);
    let asset = withdraw_asset(w);
    let amt = withdraw_balance(owner, asset, w)?;
    let Withdraw::Inline(b, _) = w;
    let hash = U(balance_hash(b)[..32].try_into().unwrap());
    balance(b)?;
    storage::interim_amt::sub(&owner.into(), &asset.into(), &hash, &U::from(amt));
    storage::withdrawable::add(&owner.into(), &asset.into(), &U::from(amt));
    call_eip20_extras::transfer(asset, owner, &U::from(amt))
        .ok_or(Error::from(ErrorDiscriminant::Erc20Invoke))
}

pub fn apply(s: StateMachine) -> R<()> {
    match s {
        StateMachine::Balance(b) => balance(&b),
        StateMachine::Commit(c) => commit(&c),
        StateMachine::Order(o) => order(&o),
        StateMachine::Withdraw(w) => withdraw(&w),
    }
}
