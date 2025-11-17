use crate::{
    call_eip20_extras,
    storage::withdrawable,
    DONE_UNIT, R,
};

use bobcat_sdk::{
    entry::{contract_address, msg_sender},
    maths::U,
};

pub fn add_liq(
    token: [u8; 20],
    recipient: [u8; 20],
    value: u128,
    deadline: U,
    v: u8,
    r: U,
    s: U,
) -> R {
    let owner = msg_sender();
    let spender = contract_address();
    let value = U::from(value);
    call_eip20_extras::permit(token, owner, spender, value, deadline, v, r, s)?;
    call_eip20_extras::transfer_from(token, owner, spender, &value)?;
    withdrawable::add(&recipient.into(), &token.into(), &value);
    DONE_UNIT
}
