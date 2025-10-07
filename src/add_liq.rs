#[cfg(feature = "storage-gen-apply")]
use crate::{
    call_eip20_extras,
    error::{ApplyContext, Error, ErrorDiscriminant},
    storage::StorageApplicationV1,
    DONE_UNIT, R,
};

#[cfg(feature = "storage-gen-apply")]
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U128, U256},
    prelude::HostAccess,
};

#[cfg(feature = "storage-gen-apply")]
impl StorageApplicationV1 {
    pub fn add_liq(
        &mut self,
        token: [u8; 20],
        recipient: [u8; 20],
        value: u128,
        deadline: [u8; 32],
        v: u8,
        r: [u8; 32],
        s: [u8; 32],
    ) -> R {
        let token = Address::from(token);
        let recipient = Address::from(recipient);
        let owner = self.vm().msg_sender();
        let value_ = U128::from_le_bytes(value.to_le_bytes());
        let value = {
            let mut b = [0u8; 32];
            b[16..].copy_from_slice(&value.to_be_bytes());
            U256::from_be_bytes(b)
        };
        let deadline = U256::from_be_bytes(deadline);
        let r = FixedBytes(r);
        let s = FixedBytes(s);
        let spender = self.vm().contract_address();
        call_eip20_extras::permit(self, token, owner, spender, value, deadline, v, r, s)?;
        call_eip20_extras::transfer_from(self, token, owner, spender, value)?;
        self.apply.withdrawable
            .setter(recipient)
            .setter(token)
            .update_check_add(value_)
            .ok_or(
                Error::from(ErrorDiscriminant::CheckedAdd)
                    .ctx(ApplyContext::AddLiq)
                    .x(u128::from_le_bytes(
                        self.apply.withdrawable.getter(recipient).get(token).to_le_bytes(),
                    ))
                    .y(u128::from_le_bytes(value_.to_le_bytes())),
            )?;
        DONE_UNIT
    }
}
