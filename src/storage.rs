use stylus_sdk::{alloy_primitives::*, prelude::*, storage::*};

use crate::{
    accounts::AccountsExpanded,
    error::{Error, ErrorDiscriminant},
};

use ed25519_dalek::VerifyingKey;

use alloc::{vec, vec::Vec};

pub type KeyEdAddr = FixedBytes<32>;

pub struct StorageBucket {
    /// The underlying asset of this bucket.
    pub asset: Address,
    /// The amount of the spendable asset in this bucket.
    pub amt: U128,
}

#[storage]
pub struct StoragePassport {
    // Owners of these addresses, using the ed25519 signatures.
    pub ed25519_owners: StorageMap<KeyEdAddr, StorageAddress>,

    /// Unspent amounts that can only be consumed by a withdrawal operation
    /// or by using a Balance application. This number is decreased
    /// if this amount is spent down, and increased if someone uses the
    /// on-chain deposit path for this contract.
    pub unspent_balances: StorageMap<Address, StorageMap<Address, StorageU128>>,

    /// Debited nonce amounts of an asset that the user has.
    pub spendable_commits: StorageMap<Address, StorageMap<Address, StorageMap<U128, StorageU128>>>,

    /// Whether, in our retracing of the operation that's already taken
    /// place, we've seen this amount deployed on-chain. If so, we
    /// use what's here, or we store it again ourselves.
    pub buckets: StorageMap<FixedBytes<32>, StorageU256>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for StoragePassport {
    fn default() -> Self {
        use stylus_sdk::testing::vm::TestVM;
        StoragePassport::from(&TestVM::new())
    }
}

impl StoragePassport {
    pub fn find_ed25519_addr(
        &self,
        accounts: &AccountsExpanded,
        id: [u8; 4],
    ) -> Result<Address, Error> {
        let addr = self.ed25519_owners.get(accounts.find_key_bytes(id)?);
        if addr.is_zero() {
            Err(Error {
                typ: ErrorDiscriminant::AccountNotFound,
                cd: vec![],
            })
        } else {
            Ok(addr)
        }
    }

    pub fn find_ed25519_key(&self, key: &VerifyingKey) -> Result<Address, Error> {
        let addr = self.ed25519_owners.get(FixedBytes::new(*key.as_bytes()));
        if addr.is_zero() {
            Err(Error {
                typ: ErrorDiscriminant::AccountNotFound,
                cd: vec![],
            })
        } else {
            Ok(addr)
        }
    }
}
