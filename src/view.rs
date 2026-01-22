use crate::{
    R,
    error::Res,
    storage::{ed25519_owners, withdrawable},
};
use bobcat_sdk::maths::U;

pub fn view_withdrawable(owner: &U, asset: &U) -> R {
    Ok(Res::DoneU256(withdrawable::get(owner, asset)))
}

pub fn view_owner(id: &U) -> R {
    Ok(Res::DoneAddress(ed25519_owners::get(&id).into()))
}
