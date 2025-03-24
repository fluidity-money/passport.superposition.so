use stylus_sdk::{alloy_primitives::*, storage::*};

#[cfg_attr(target_arch = "wasm32", stylus_sdk::prelude::storage)]
pub struct StoragePassport {
    // Owners of these addresses, based on the ed25519 signature.
    pub owners: StorageMap<FixedBytes<32>, StorageAddress>,

    // Associated Ethereum addresses with signatures. There can only be one
    // public key at a time associated.
    pub associated: StorageMap<Address, FixedBytes<32>>,

    // Nonces per ed25519 signature.
    pub nonces: StorageMap<FixedBytes<32>, StorageMap<FixedBytes<32>, StorageU256>>,
}
