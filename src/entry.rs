use crate::{applicative::Applicative, error::*, storage::*};

use alloc::vec::Vec;

impl Storage {
    pub fn dummy(&self) -> R {
        DONE_UNIT
    }

    pub fn solve(&mut self, accounts: Vec<u64>, applicative: Applicative) -> R {
        self.apply(self.validate(&accounts, applicative)?)?;
        DONE_UNIT
    }
}
