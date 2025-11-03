use ed25519_dalek::VerifyingKey;

use bobcat_sdk::{maths::*, storage::*};

use crate::error::{ApplyContext, Error, ErrorDiscriminant};

pub type Address = [u8; 20];

pub mod ed25519_count {
    use super::*;

    pub const fn hash() -> U {
        const_keccak256(b"ed25519_count")
    }

    // Count of the number of seen addresses, that we use our shortened
    // accounts list form to look up. We use this instead of a map so we can
    // use a u64 instead of the native wasm word (u32).
    pub fn get() -> U {
        storage_load(&hash())
    }

    pub fn incr() -> Result<U, Error> {
        storage_checked_add_res(&hash(), &U::ONE)
            .map_err(|(x, y)| Error::from(ErrorDiscriminant::CheckedAdd).x(x).y(y))
    }
}

pub mod ed25519_keys {
     use super::*;

    pub fn hash(id: &U) -> U {
        slot_map(&const_keccak256(b"ed25519_keys"), id)
    }

    // Find the VerifyingKey using an id.
    pub fn get(id: &U) -> U {
        storage_load(&hash(id))
    }

    pub fn set(id: &U, key: &U) {
        storage_store(&hash(id), key)
    }
}

pub mod ed25519_owners {
    use super::*;

    pub fn hash(id: &U) -> U {
        slot_map(&const_keccak256(b"ed25519_owners"), id)
    }

    // Find the address owner of a key using its id.
    pub fn get(id: &U) -> Address {
        storage_load(&hash(id)).into()
    }

    pub fn set(id: &U, key: &U) {
        storage_store(&hash(id), key)
    }
}

fn hash_details_hash_owner_l(h: &U) -> U {
    slot_map(&const_keccak256(b"details_hash_owner_l"), &h)
}

fn hash_details_hash_owner_r(h: &U) -> U {
    slot_map(&const_keccak256(b"details_hash_owner_r"), &h)
}

fn hash_interim_amount(owner: &Address, asset: &Address, hash: &U) -> U {
    slot_map(
        &slot_map(
            &slot_map(&const_keccak256(b"interim_amount"), &owner.into()),
            &asset.into(),
        ),
        hash,
    )
}

// Get the interim amount for the owner, the asset, and the hash given.
pub fn get_interim_amount(owner: &Address, asset: &Address, hash: &U) -> u128 {
    storage_load(&hash_interim_amount(owner, asset, hash)).into()
}

pub fn get_interim_amount_hash(owner: &Address, asset: &Address, hash: &[u8; 64]) -> u128 {
    let h: [u8; 32] = hash[..32].try_into().unwrap();
    get_interim_amount(owner, asset, &U::from(h))
}

pub fn increase_interim_amount(
    ctx: ApplyContext,
    owner: &Address,
    asset: &Address,
    hash: &[u8; 64],
    amt: &U,
) -> Result<U, Error> {
    let hash: [u8; 32] = hash[..32].try_into().unwrap();
    storage_checked_add_res(&hash_interim_amount(owner, asset, &U::from(hash)), amt)
        .map_err(|(x, y)| err_checked_add(ctx, x, y))
}

pub fn decrease_interim_amount(
    ctx: ApplyContext,
    owner: &Address,
    asset: &Address,
    hash: &[u8; 64],
    amt: &U,
) -> Result<U, Error> {
    let hash: [u8; 32] = hash[..32].try_into().unwrap();
    storage_checked_sub_res(&hash_interim_amount(owner, asset, &U::from(hash)), amt)
        .map_err(|(x, y)| err_checked_sub(ctx, x, y))
}

pub fn hash_order_amount(owner: &Address, asset: &Address, hash: &U) -> U {
    slot_map(
        &slot_map(
            &slot_map(&const_keccak256(b"order_amount"), &owner.into()),
            &asset.into(),
        ),
        hash,
    )
    .into()
}

// Get the order amount for the owner, the asset, and the hash given.
pub fn get_order_amt(owner: &Address, asset: &Address, hash: &U) -> u128 {
    storage_load(&hash_order_amount(owner, asset, hash)).into()
}

pub fn increase_order_amount(
    ctx: ApplyContext,
    owner: &Address,
    asset: &Address,
    hash: &[u8; 64],
    amt: &U,
) -> Result<U, Error> {
    let hash: [u8; 32] = hash[..32].try_into().unwrap();
    storage_checked_add_res(&hash_order_amount(owner, asset, &U::from(hash)), amt)
        .map_err(|(x, y)| err_checked_add(ctx, x, y))
}

pub fn decrease_order_amount(
    ctx: ApplyContext,
    owner: &Address,
    asset: &Address,
    hash: &[u8; 64],
    amt: &U,
) -> Result<U, Error> {
    let hash: [u8; 32] = hash[..32].try_into().unwrap();
    storage_checked_sub_res(&hash_order_amount(owner, asset, &U::from(hash)), amt)
        .map_err(|(x, y)| err_checked_sub(ctx, x, y))
}

pub fn get_order_amt_hash(owner: &Address, asset: &Address, hash: &[u8; 64]) -> u128 {
    let h: [u8; 32] = hash[..32].try_into().unwrap();
    get_order_amt(owner, asset, &U::from(h))
}

fn hash_withdrawable(owner: &Address, asset: &Address) -> U {
    slot_map(
        &slot_map(&const_keccak256(b"withdrawable"), &owner.into()),
        &asset.into(),
    )
}

// Get the withdrawable amount for an owner and asset.
pub fn get_withdrawable(owner: &Address, asset: &Address) -> U {
    storage_load(&hash_withdrawable(owner, asset))
}

pub fn increase_withdrawable(
    ctx: ApplyContext,
    owner: &Address,
    asset: &Address,
    amt: &U,
) -> Result<U, Error> {
    storage_checked_add_res(&hash_withdrawable(owner, asset), amt)
        .map_err(|(x, y)| err_checked_add(ctx, x, y))
}

pub fn decrease_withdrawable(
    ctx: ApplyContext,
    owner: &Address,
    asset: &Address,
    amt: &U,
) -> Result<U, Error> {
    storage_checked_sub_res(&hash_withdrawable(owner, asset), amt)
        .map_err(|(x, y)| err_checked_sub(ctx, x, y))
}

// Get the owner of the right side of the hash given.
pub fn get_details_hash_owner_r(h: &U) -> Address {
    storage_load(&slot_map(&const_keccak256(b"details_hash_owner_r"), h)).into()
}

pub fn hash_get_details_hash_asset_l(h: &U) -> U {
    slot_map(&const_keccak256(b"details_hash_asset_l"), h)
}

// Get the first asset in this hash.
pub fn get_details_hash_asset_l(h: &U) -> Address {
    storage_load(&hash_get_details_hash_asset_l(h)).into()
}

pub fn get_details_hash_asset_l_hash(h: &[u8; 64]) -> Address {
    let h: [u8; 32] = h[..32].try_into().unwrap();
    get_details_hash_asset_l(&U::from(h))
}

pub fn set_details_hash_asset_l(h: &U, v: &Address) {
    storage_store(&hash_get_details_hash_asset_l(h), &v.into())
}

pub fn set_details_hash_asset_l_hash(h: &[u8; 64], v: &Address) {
    let h: [u8; 32] = h[..32].try_into().unwrap();
    set_details_hash_asset_l(&U::from(h), v.into())
}

pub fn hash_details_hash_asset_r(h: &U) -> U {
    slot_map(&const_keccak256(b"details_hash_asset_r"), h)
}

// Get the second asset of the hash. This is used by orders to store the desired asset.
pub fn get_details_hash_asset_r(h: &U) -> Address {
    storage_load(&hash_details_hash_asset_r(h)).into()
}

pub fn set_details_hash_asset_r(h: &U, v: &Address) {
    storage_store(&hash_details_hash_asset_r(h), &v.into())
}

pub fn set_details_hash_asset_r_hash(h: &[u8; 64], v: &Address) {
    let h: [u8; 32] = h[..32].try_into().unwrap();
    set_details_hash_asset_r(&U::from(h), v.into())
}

pub fn get_details_hash_asset_r_hash(h: &[u8; 64]) -> Address {
    let h: [u8; 32] = h[..32].try_into().unwrap();
    get_details_hash_asset_r(&U::from(h))
}

fn hash_details_hash_order_desired_amt(h: &U) -> U {
    slot_map(&const_keccak256(b"details_hash_order_desired_amt"), h)
}

// Get the desired asset amount by the order at this hash.
pub fn get_details_hash_order_desired_amt(h: &U) -> u128 {
    storage_load(&hash_details_hash_order_desired_amt(h)).into()
}

// Get the desired asset amount by the order at this hash.
pub fn get_details_hash_order_desired_amt_hash(h: &[u8; 64]) -> u128 {
    let h: [u8; 32] = h[..32].try_into().unwrap();
    get_details_hash_order_desired_amt(&U::from(h))
}

fn err_checked_add(c: ApplyContext, x: U, y: U) -> Error {
    Error::from(ErrorDiscriminant::CheckedAdd).ctx(c).x(x).y(y)
}

fn err_checked_sub(c: ApplyContext, x: U, y: U) -> Error {
    Error::from(ErrorDiscriminant::CheckedSub).ctx(c).x(x).y(y)
}

fn err_hash_already_onchain(h: &[u8; 64]) -> Error {
    Error::from(ErrorDiscriminant::HashAlreadyOnchain).hash(*h)
}

pub fn add_details_hash_order_desired_amt(ctx: ApplyContext, h: &U, extra: &U) -> Result<U, Error> {
    storage_checked_add_res(&hash_details_hash_order_desired_amt(h), extra)
        .map_err(|(x, y)| err_checked_add(ctx, x, y))
}

// Get the owner of this contract.
pub fn get_owner() -> Address {
    storage_load(&const_keccak256(b"owner")).into()
}

pub fn set_hash_details_owner_l(h: &[u8; 64], owner: Address) {
    let k: [u8; 32] = h[..32].try_into().unwrap();
    storage_store(&hash_details_hash_owner_l(&U(k)), &U::from(owner)).into()
}

pub fn get_hash_owner_l(h: &[u8; 64]) -> Address {
    let k: [u8; 32] = h[..32].try_into().unwrap();
    storage_load(&hash_details_hash_owner_l(&U(k))).into()
}

pub fn get_hash_owner_r(h: &[u8; 64]) -> Address {
    let k: [u8; 32] = h[..32].try_into().unwrap();
    storage_load(&hash_details_hash_owner_r(&U(k))).into()
}

pub fn set_hash_owner_r(h: &[u8; 64], v: &Address) {
    let k: [u8; 32] = h[..32].try_into().unwrap();
    storage_store(&hash_details_hash_owner_r(&U(k)), &U::from(v))
}

pub fn find_ed25519_key(i: &U) -> Result<VerifyingKey, Error> {
    let v = ed25519_keys::get(i);
    if v.is_zero() {
        Err(Error::from(ErrorDiscriminant::AccountIdNotFound))
    } else {
        VerifyingKey::from_bytes(&v.0).map_err(|_| Error::from(ErrorDiscriminant::BadVerifyingKey))
    }
}

pub fn find_ed25519_addr(i: &U) -> Result<Address, Error> {
    let addr = ed25519_owners::get(i);
    if addr == [0u8; 20] {
        Err(Error::from(ErrorDiscriminant::AccountIdNotFound))
    } else {
        Ok(addr)
    }
}

pub fn ensure_hash_unseen(hash: &[u8; 64]) -> Result<(), Error> {
    if get_hash_owner_l(hash) != [0u8; 20] {
        return Err(err_hash_already_onchain(hash));
    }
    Ok(())
}

pub struct Storage;
