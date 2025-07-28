use stylus_sdk::{alloy_primitives::*, prelude::*, storage::*};

use alloc::{vec, vec::Vec};

pub type KeyEdAddr = FixedBytes<32>;

#[storage]
pub struct StoragePassport {
    // Owners of these addresses, based on the ed25519 signature.
    pub owners: StorageMap<KeyEdAddr, StorageAddress>,

    pub unspent_balances: StorageMap<Address, StorageU256>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for StoragePassport {
    fn default() -> Self {
        use stylus_sdk::testing::vm::TestVM;
        StoragePassport::from(&TestVM::new())
    }
}
