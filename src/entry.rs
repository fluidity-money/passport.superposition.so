use crate::{encoding::*, applicative::Applicative, storage::*, error::*, ops::*};

impl StoragePassport {
    pub fn dummy(&self) -> R {
        NOTHING
    }

    pub fn query_unused_liq(&self, _addr: BAddress) -> R {
        NOTHING
    }

    pub fn deposit_unused_liq(&mut self, _l: DepositUnusedLiquidity) -> R {
        NOTHING
    }

    pub fn solve(&mut self, _applicative: Applicative) -> R {
        NOTHING
    }
}
