use alloc::vec::Vec;

pub use crate::storage::*;

use crate::{encoding::*, error::*};

use stylus_sdk::alloy_primitives::*;

impl StoragePassport {
    pub fn spend(&mut self, spendable: Vec<(BAddress, BU256)>) -> R {
        ok_count(spendable.iter().fold(U256::ZERO, |acc, (_, x)| acc + x.x))
    }
}
