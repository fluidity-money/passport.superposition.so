use crate::{error::NOOP, Storage, R};

impl Storage {
    pub fn onboard(
        &mut self,
        key: [u8; 32],
        owner: [u8; 20],
        value: [u8; 32],
        deadline: [u8; 32],
        v: u8,
        r: [u8; 32],
        s: [u8; 32],
    ) -> R {
        NOOP
    }
}
