use crate::{
    R,
    error::Res,
    storage::{ed25519_owners, hash_owner_l, hash_owner_r, withdrawable},
};
use bobcat_sdk::maths::U;

pub fn view_withdrawable(owner: &U, asset: &U) -> R {
    Ok(Res::DoneU256(withdrawable::get(owner, asset)))
}

pub fn view_owner(id: &U) -> R {
    Ok(Res::DoneAddress(ed25519_owners::get(&id).into()))
}

pub fn view_hash_owner_l(hash: &U) -> R {
    Ok(Res::DoneAddress(hash_owner_l::get(&hash).into()))
}

pub fn view_hash_owner_r(hash: &U) -> R {
    Ok(Res::DoneAddress(hash_owner_r::get(&hash).into()))
}
