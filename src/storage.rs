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

/// Storage for amounts available for spending at a timestamp. Is owner
/// => asset => timestamp => amount.
pub type StorageTickets = StorageMap<Address, StorageMap<Address, StorageMap<U128, StorageU128>>>;

#[storage]
pub struct StoragePassport {
    // Owners of these addresses, using the ed25519 signatures.
    pub ed25519_owners: StorageMap<KeyEdAddr, StorageAddress>,

    pub orders: StorageTickets,

    pub withdrawable: StorageMap<Address, StorageMap<Address, StorageU128>>,

    pub interim: StorageTickets,
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
        typ: ErrorDiscriminant::CheckedAdd(u128::from_le_bytes(x.to_le_bytes()), y),
        cd: vec![],
    }
}

fn err_checked_sub(x: U128, y: u128) -> Error {
    Error {
        typ: ErrorDiscriminant::CheckedSub(u128::from_le_bytes(x.to_le_bytes()), y),
        cd: vec![],
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
                typ: ErrorDiscriminant::AccountIdNotFound(id),
                cd: vec![],
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
                cd: vec![],
            })
        } else {
            Ok(addr)
        }
    }

    pub fn increase_interim(
        &mut self,
        owner: Address,
        asset: Address,
        ms_ts: u128,
        y: u128,
    ) -> Result<(), Error> {
        let ms_ts = U128::from_le_bytes(ms_ts.to_le_bytes());
        let x = self.interim.getter(owner).getter(asset).get(ms_ts);
        self.interim.setter(owner).setter(asset).setter(ms_ts).set(
            x.checked_add(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_add(x, y))?,
        );
        Ok(())
    }

    pub fn decrease_interim(
        &mut self,
        owner: Address,
        asset: Address,
        ms_ts: u128,
        y: u128,
    ) -> Result<(), Error> {
        let ms_ts = U128::from_le_bytes(ms_ts.to_le_bytes());
        let x = self.interim.getter(owner).getter(asset).get(ms_ts);
        self.interim.setter(owner).setter(asset).setter(ms_ts).set(
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
        ms_ts: u128,
        y: u128,
    ) -> Result<(), Error> {
        let ms_ts = U128::from_le_bytes(ms_ts.to_le_bytes());
        let x = self.orders.getter(owner).getter(asset).get(ms_ts);
        self.orders.setter(owner).setter(asset).setter(ms_ts).set(
            x.checked_add(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_add(x, y))?,
        );
        Ok(())
    }

    pub fn decrease_order(
        &mut self,
        owner: Address,
        asset: Address,
        ms_ts: u128,
        y: u128,
    ) -> Result<(), Error> {
        let ms_ts = U128::from_le_bytes(ms_ts.to_le_bytes());
        let x = self.orders.getter(owner).getter(asset).get(ms_ts);
        self.orders.setter(owner).setter(asset).setter(ms_ts).set(
            x.checked_sub(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_sub(x, y))?,
        );
        Ok(())
    }
}
