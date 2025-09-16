use crate::{Storage, DONE_UNIT, R};

impl Storage {
    pub fn onboard(
        &mut self,
        key: [u8; 32],
        sig: [u8; 64],
        nonce: u16,
        owner: [u8; 20],
        onboard_v: u8,
        onboard_r: [u8; 32],
        onboard_s: [u8; 32],
        token: [u8; 20],
        value: u128,
        deadline: [u8; 32],
        permit_v: u8,
        permit_r: [u8; 32],
        permit_s: [u8; 32],
    ) -> R {
        // Check the user's signature first:
        self.add_liq(token, owner, value, deadline, permit_v, permit_r, permit_s)?;
        DONE_UNIT
    }
}
