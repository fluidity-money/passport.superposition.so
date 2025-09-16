use crate::{
    call_eip20_extras,
    error::{Error, ErrorDiscriminant},
    Storage, R,
    DONE_UNIT,
};

use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U128, U256},
    prelude::HostAccess,
};

impl Storage {
    pub fn add_liq(
        &mut self,
        token: [u8; 20],
        owner: [u8; 20],
        value: u128,
        deadline: [u8; 32],
        v: u8,
        r: [u8; 32],
        s: [u8; 32],
    ) -> R {
        let token = Address::from(token);
        let owner = self.vm().msg_sender();
        let recipient = Address::from(owner);
        let value_ = U128::from_le_bytes(value.to_le_bytes());
        let value = {
            let mut b = [0u8; 32];
            b[16..].copy_from_slice(&value.to_le_bytes());
            U256::from_be_bytes(b)
        };
        let deadline = U256::from_be_bytes(deadline);
        let r = FixedBytes(r);
        let s = FixedBytes(s);
        let spender = self.vm().contract_address();
        call_eip20_extras::permit(self, token, owner, spender, value, deadline, v, r, s)?;
        call_eip20_extras::transfer_from(self, token, spender, owner, value)?;
        self.withdrawable
            .setter(recipient)
            .setter(token)
            .update_check_add(value_)
            .ok_or(Error {
                typ: ErrorDiscriminant::CheckedAdd,
            })?;
        DONE_UNIT
    }
}
