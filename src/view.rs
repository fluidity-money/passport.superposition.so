use crate::{R, error::Res, storage::withdrawable};
use bobcat_sdk::maths::U;

pub fn view_withdrawable(owner: &U, asset: &U) -> R {
    Ok(Res::DoneU256(withdrawable::get(owner, asset)))
}
