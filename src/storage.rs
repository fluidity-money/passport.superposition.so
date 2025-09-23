use stylus_sdk::{alloy_primitives::*, prelude::*, storage::*};

use crate::error::{Error, ErrorDiscriminant, MathContext};

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

// Testing storage that's used for EIP20 and more.
#[storage]
#[cfg(not(target_arch = "wasm32"))]
pub struct StorageTest {
    pub balances: StorageMap<Address, StorageMap<Address, StorageU256>>,
    // Contract => Owner (user) => Spender (passport) => Amount
    pub allowances: StorageMap<Address, StorageMap<Address, StorageMap<Address, StorageU256>>>,
}

#[storage]
#[cfg(target_arch = "wasm32")]
pub struct StorageTest;

#[storage]
pub struct StorageApplicationV1 {
    // It's very important that this contains nothing during a on-chain
    // deployment.
    pub test_eip20: StorageTest,

    // Count of the number of seen addresses, that we use our shortened
    // accounts list form to look up. We use this instead of a map so we can
    // use a u64 instead of the native wasm word (u32).
    pub ed25519_count: StorageU64,

    // Tool to find the VerifyingKey using an id, to reduce calldata size.
    pub ed25519_keys: StorageMap<u64, StorageFixedBytes<32>>,

    // Owners of the offset of these addresses, using the ed25519 signatures.
    pub ed25519_owners: StorageMap<u64, StorageAddress>,

    /// Outstanding orders that could be used in another part of the operation.
    pub orders: StorageTickets,

    /// Amounts that could be withdrawn from the system. Owner => asset => amount.
    pub withdrawable: StorageMap<Address, StorageMap<Address, StorageU128>>,

    /// Interim balances that make up Balances.
    pub interim: StorageTickets,

    /// The owner of the left side of the hash given. It should not be zero.
    pub details_hash_owner_l: StorageMap<FixedBytes<32>, StorageAddress>,

    /// The owner of the right side of the hash given.
    pub details_hash_owner_r: StorageMap<FixedBytes<32>, StorageAddress>,

    /// The first asset in this hash.
    pub details_hash_asset_l: StorageMap<FixedBytes<32>, StorageAddress>,

    /// The second asset of the hash. This is used by orders to store the desired asset.
    pub details_hash_asset_r: StorageMap<FixedBytes<32>, StorageAddress>,

    /// The desired asset by the order at this hash on its own.
    pub details_hash_order_desired_amt: StorageMap<FixedBytes<32>, StorageU128>,
}

#[storage]
pub struct StorageAdminV1 {
    pub owner: StorageAddress,
}

/// Toplevel storage for the entire application. TODO: figure out how to
/// set offsets for each storage accessor here, then comment out the bits
/// we don't use in each facet.
#[storage]
pub struct Storage {
    pub app: StorageApplicationV1,
    pub admin: StorageAdminV1,
}

unsafe impl TopLevelStorage for Storage {}
unsafe impl TopLevelStorage for StorageApplicationV1 {}

#[cfg(not(target_arch = "wasm32"))]
use std::{cell::RefCell, collections::HashMap};

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    // This field is used as a local thread helper to make it possible to
    // scan the hashes in the interim balances and others when this is
    // used in an offline (non-contract) context. It's scanned to see
    // if we saw any hashes. It shouldn't be trusted completely.
    pub static SEEN_HASHES: RefCell<HashMap<[u8; 64], bool>> =
        RefCell::new(HashMap::new());
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for Storage {
    fn default() -> Self {
        use stylus_sdk::testing::vm::TestVM;
        Storage::from(&TestVM::new())
    }
}

fn err_checked_add(c: MathContext, x: U128, y: u128) -> Error {
    Error::from(ErrorDiscriminant::CheckedAdd(
        c,
        u128::from_le_bytes(x.to_le_bytes()),
        y,
    ))
}

fn err_checked_sub(c: MathContext, x: U128, y: u128) -> Error {
    Error::from(ErrorDiscriminant::CheckedSub(
        c,
        u128::from_le_bytes(x.to_le_bytes()),
        y,
    ))
}

fn track_hash(x: &[u8; 64]) {
    #[cfg(not(target_arch = "wasm32"))]
    SEEN_HASHES.with(|h| h.borrow_mut().insert(*x, true));
}

impl StorageApplicationV1 {
    pub fn set_hash_details_l(&mut self, h: &[u8; 64], owner: Address, asset: Address) {
        let h = FixedBytes::from_slice(&h[..32]);
        self.details_hash_owner_l.setter(h).set(owner);
        self.details_hash_asset_l.setter(h).set(asset);
    }

    pub fn set_hash_details_r(&mut self, h: &[u8; 64], owner: Address, asset: Address) {
        let h = FixedBytes::from_slice(&h[..32]);
        self.details_hash_owner_r.setter(h).set(owner);
        self.details_hash_asset_r.setter(h).set(asset);
    }

    pub fn set_hash_details_desired_asset(&mut self, h: &[u8; 64], asset: Address) {
        // Sets the right side asset.
        let h = FixedBytes::from_slice(&h[..32]);
        self.details_hash_asset_r.setter(h).set(asset);
    }

    pub fn get_hash_asset_l(&self, h: &[u8; 64]) -> Address {
        self.details_hash_asset_l
            .get(FixedBytes::from_slice(&h[..32]))
    }

    pub fn get_hash_asset_r(&self, h: &[u8; 64]) -> Address {
        self.details_hash_asset_r
            .get(FixedBytes::from_slice(&h[..32]))
    }

    pub fn get_hash_owner_l(&self, h: &[u8; 64]) -> Address {
        self.details_hash_owner_l
            .get(FixedBytes::from_slice(&h[..32]))
    }

    pub fn get_hash_owner_r(&self, h: &[u8; 64]) -> Address {
        self.details_hash_owner_r
            .get(FixedBytes::from_slice(&h[..32]))
    }

    pub fn get_hash_order_desired_amount(&self, h: &[u8; 64]) -> u128 {
        u128::from_be_bytes(
            self.details_hash_order_desired_amt
                .get(FixedBytes::from_slice(&h[..32]))
                .to_be_bytes(),
        )
    }

    pub fn find_ed25519_key(&self, i: u64) -> Result<VerifyingKey, Error> {
        let v = self.ed25519_keys.get(i);
        if v.is_zero() {
            Err(Error::from(ErrorDiscriminant::AccountIdNotFound))
        } else {
            VerifyingKey::from_bytes(&v.0)
                .map_err(|_| Error::from(ErrorDiscriminant::BadVerifyingKey))
        }
    }

    pub fn find_ed25519_addr(&self, i: u64) -> Result<Address, Error> {
        let addr = self.ed25519_owners.get(i);
        if addr.is_zero() {
            Err(Error::from(ErrorDiscriminant::AccountIdNotFound))
        } else {
            Ok(addr)
        }
    }

    pub fn get_interim(&self, owner: Address, asset: Address, h: &[u8; 64]) -> u128 {
        track_hash(h);
        u128::from_be_bytes(
            self.interim
                .getter(owner)
                .getter(asset)
                .get(FixedBytes::from_slice(&h[..32]))
                .to_be_bytes(),
        )
    }

    pub fn increase_interim(
        &mut self,
        owner: Address,
        asset: Address,
        h: &[u8; 64],
        y: u128,
    ) -> Result<(), Error> {
        track_hash(h);
        let h = FixedBytes::from_slice(&h[..32]);
        let x = self.interim.getter(owner).getter(asset).get(h);
        self.interim.setter(owner).setter(asset).setter(h).set(
            x.checked_add(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_add(MathContext::IncreaseInterim, x, y))?,
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
        track_hash(h);
        let h = FixedBytes::from_slice(&h[..32]);
        let x = self.interim.getter(owner).getter(asset).get(h);
        self.interim.setter(owner).setter(asset).setter(h).set(
            x.checked_sub(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_sub(MathContext::DecreaseInterim, x, y))?,
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
                .ok_or(err_checked_add(MathContext::IncreaseWithdrawal, x, y))?,
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
                .ok_or(err_checked_sub(MathContext::DecreaseWithdrawal, x, y))?,
        );
        Ok(())
    }

    pub fn get_order(&self, owner: Address, asset: Address, h: &[u8; 64]) -> u128 {
        let h = FixedBytes::from_slice(&h[..32]);
        u128::from_be_bytes(self.orders.getter(owner).getter(asset).get(h).to_be_bytes())
    }

    pub fn increase_order(
        &mut self,
        owner: Address,
        asset: Address,
        h: &[u8; 64],
        y: u128,
    ) -> Result<(), Error> {
        let h = FixedBytes::from_slice(&h[..32]);
        let x = self.orders.getter(owner).getter(asset).get(h);
        self.orders.setter(owner).setter(asset).setter(h).set(
            x.checked_add(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_add(MathContext::IncreaseOrder, x, y))?,
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
        let h = FixedBytes::from_slice(&h[..32]);
        let x = self.orders.getter(owner).getter(asset).get(h);
        self.orders.setter(owner).setter(asset).setter(h).set(
            x.checked_sub(U128::from_le_bytes(y.to_le_bytes()))
                .ok_or(err_checked_sub(MathContext::DecreaseOrder, x, y))?,
        );
        Ok(())
    }
}
