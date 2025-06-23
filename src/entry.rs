
pub use crate::{applicative::Applicative, storage::*};

use crate::*;

impl StoragePassport {
    pub fn dummy(&self) -> R {
        NOTHING
    }

    pub fn deposit_liquidity(&mut self, _l: DepositLiquidity) -> R {
        NOTHING
    }

    pub fn solve(&mut self, applicative: Applicative) -> R {
        NOTHING
    }
}
