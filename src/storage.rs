use stylus_sdk::{alloy_primitives::*, prelude::*, storage::*};

use alloc::{vec, vec::Vec};

#[storage]
pub struct StoragePassport {
    // Owners of these addresses, based on the ed25519 signature.
    pub owners: StorageMap<FixedBytes<32>, StorageAddress>,

    // Nonces per ed25519 signature. If the nonce is 0, we assume it doesn't
    // exist, and we bump the nonce.
    pub nonces: StorageMap<FixedBytes<32>, StorageU256>,

    // Nonces needed to do bonding of ed25519 accounts.
    pub bonding_nonces: StorageMap<Address, StorageU256>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for StoragePassport {
    fn default() -> Self {
        use stylus_sdk::testing::vm::TestVM;
        StoragePassport::from(&TestVM::new())
    }
}
