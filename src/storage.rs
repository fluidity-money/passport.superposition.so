use stylus_sdk::{alloy_primitives::*, prelude::*, storage::*};

use crate::{
    accounts::AccountsList,
    error::{Error, ErrorDiscriminant},
};

use alloc::{vec, vec::Vec};

use ed25519_dalek::VerifyingKey;

pub type KeyEdAddr = FixedBytes<32>;

pub struct StorageBucket {
    /// The underlying asset of this bucket.
    pub asset: Address,
    /// The amount of the spendable asset in this bucket.
    pub amt: U128,
}

/// Storage for amounts available for spending at a timestamp. Is owner
/// => asset => timestamp => amount.
pub type StorageTickets =
    StorageMap<Address, StorageMap<Address, StorageMap<FixedBytes<32>, StorageU128>>>;

#[storage]
pub struct StoragePassport {
    // Owners of these addresses, using the ed25519 signatures.
    pub ed25519_owners: StorageMap<KeyEdAddr, StorageAddress>,

    /// Outstanding orders that could be used in another part of the operation.
    pub orders: StorageTickets,

    /// Amounts that could be withdrawn from the system.
    pub withdrawable: StorageMap<Address, StorageMap<Address, StorageU128>>,

    /// Interim balances that make up Balances.
    pub interim: StorageTickets,

    /// The owner of the left side of the hash given. It should not be zero.
    pub details_hash_owner_l: StorageMap<FixedBytes<32>, StorageU256>,

    /// The owner of the right side of the hash given.
    pub details_hash_owner_r: StorageMap<FixedBytes<32>, StorageU256>,

    /// The first asset in this hash.
    pub details_asset_l: StorageMap<FixedBytes<32>, StorageAddress>,

    /// The second asset of the hash.
    pub details_asset_r: StorageMap<FixedBytes<32>, StorageAddress>,
}

unsafe impl stylus_sdk::stylus_core::storage::TopLevelStorage for StoragePassport {}

#[cfg(not(target_arch = "wasm32"))]
impl Default for StoragePassport {
    fn default() -> Self {
        use stylus_sdk::testing::vm::TestVM;
        StoragePassport::from(&TestVM::new())
    }
}

fn err_checked_add(x: U128, y: u128) -> Error {
    Error {
        typ: ErrorDiscriminant::CheckedAdd,
    }
}

fn err_checked_sub(_x: U128, _y: u128) -> Error {
    Error {
        typ: ErrorDiscriminant::CheckedSub,
    }
}

impl StoragePassport {
    pub fn set_hash_details_l(&mut self, h: &[u8; 64], owner: Address, asset: Address) {
        todo!()
    }

    pub fn set_hash_details_r(&mut self, h: &[u8; 64], owner: Address, asset: Address) {
        todo!()
    }

    pub fn find_ed25519_addr(
        &self,
        accounts: &AccountsList,
        id: [u8; 4],
    ) -> Result<Address, Error> {
        let addr = self.ed25519_owners.get(accounts.find_key_bytes(id)?);
        if addr.is_zero() {
            Err(Error {
                typ: ErrorDiscriminant::AccountIdNotFound,
            })
        } else {
            Ok(addr)
        }
    }

    pub fn find_ed25519_key(&self, key: &VerifyingKey) -> Result<Address, Error> {
        let k = *key.as_bytes();
        let addr = self.ed25519_owners.get(FixedBytes::new(k));
        if addr.is_zero() {
            Err(Error {
                typ: ErrorDiscriminant::AccountKeyNotFound,
            })
        } else {
            Ok(addr)
        }
    }

    pub fn increase_interim(
        &mut self,
        owner: Address,
        asset: Address,
        h: &[u8; 64],
        y: u128,
    ) -> Result<(), Error> {
        let h = FixedBytes::from_slice(h);
        let x = self.interim.getter(owner).getter(asset).get(h);
        self.interim.setter(owner).setter(asset).setter(h).set(
            x.checked_add(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_add(x, y))?,
        );
        Ok(())
    }

    pub fn decrease_interim(
        &mut self,
        owner: Address,
        asset: Address,
        h: &[u8; 64],
        y: u128,
    ) -> Result<(), Error> {
        let h = FixedBytes::from_slice(&h[..32]);
        let x = self.interim.getter(owner).getter(asset).get(h);
        self.interim.setter(owner).setter(asset).setter(h).set(
            x.checked_sub(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_sub(x, y))?,
        );
        Ok(())
    }

    pub fn increase_withdrawal(
        &mut self,
        owner: Address,
        asset: Address,
        y: u128,
    ) -> Result<(), Error> {
        let x = self.withdrawable.getter(owner).getter(asset).get();
        self.withdrawable.setter(owner).setter(asset).set(
            x.checked_add(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_add(x, y))?,
        );
        Ok(())
    }

    pub fn decrease_withdrawal(
        &mut self,
        owner: Address,
        asset: Address,
        y: u128,
    ) -> Result<(), Error> {
        let x = self.withdrawable.getter(owner).getter(asset).get();
        self.withdrawable.setter(owner).setter(asset).set(
            x.checked_sub(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_sub(x, y))?,
        );
        Ok(())
    }

    pub fn increase_order(
        &mut self,
        owner: Address,
        asset: Address,
        h: &[u8; 64],
        y: u128,
    ) -> Result<(), Error> {
    let h = FixedBytes::from_slice(h);
        let x = self.orders.getter(owner).getter(asset).get(h);
        self.orders.setter(owner).setter(asset).setter(h).set(
            x.checked_add(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_add(x, y))?,
        );
        Ok(())
    }

    pub fn decrease_order(
        &mut self,
        owner: Address,
        asset: Address,
        h: &[u8; 64],
        y: u128,
    ) -> Result<(), Error> {
        let h = FixedBytes::from_slice(h);
        let x = self.orders.getter(owner).getter(asset).get(h);
        self.orders.setter(owner).setter(asset).setter(h).set(
            x.checked_sub(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_sub(x, y))?,
        );
        Ok(())
    }
}
