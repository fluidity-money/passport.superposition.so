use alloc::vec::Vec;

pub use crate::storage::*;

use crate::*;

use stylus_sdk::alloy_primitives::*;

impl StoragePassport {
    pub fn dummy(&self) -> R {
        NOTHING
    }

    /// Simulate a spend operation, reverting at the end if everything went through okay.
    pub fn sim_spend(
        &mut self,
        allowlist: Vec<BAddress>,
        spendable: Vec<(BAddress, BU256)>,
        cds: Vec<Vec<u8>>,
    ) -> R {
        DONE
    }

    /// Actually make a spend, first verifying the signature after spendable goals and
    /// allowlist.
    pub fn spend(
        &mut self,
        allowlist: Vec<BAddress>,
        spendable: Vec<(BAddress, BU256)>,
        sig: SecpSig,
        cds: Vec<Vec<u8>>,
    ) -> R {
        DONE
    }
}
