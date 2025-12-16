use bobcat_sdk::maths::*;

use crate::error::{Error, ErrorDiscriminant};

pub type Address = [u8; 20];

macro_rules! storage {
    ($($name:ident($($param:ident),*)),* $(,)?) => {
        storage! {
            @internal
            counter: 0,
            items: [$($name($($param),*)),*]
        }
    };
    (@internal
        counter: $counter:expr,
        items: []
    ) => {};
    (@internal
        counter: $counter:expr,
        items: [$name:ident($($param:ident),*) $(, $($rest:tt)*)?]
    ) => {
        pub mod $name {
            pub(crate) use bobcat_sdk::storage::*;
            use bobcat_sdk::maths::{u, U};

            const SLOT: U = u!($counter);

            storage!(@impl [$($param),*]);
        }
        $(
            storage! {
                @internal
                counter: $counter + 1,
                items: [$($rest)*]
            }
        )?
    };
    (@impl []) => {
        pub fn get() -> U {
            storage_load(&SLOT)
        }
        pub fn set(x: &U) {
            storage_store(&SLOT, x)
        }
        pub fn add(x: &U) -> Option<()> {
            storage_checked_add(&SLOT, x)
        }
        pub fn sub(x: &U) -> Option<()> {
            storage_checked_sub(&SLOT, x)
        }
        pub fn clear() {
            set(&U::ZERO)
        }
    };
    (@impl [$param1:ident]) => {
        pub fn get($param1: &U) -> U {
            storage_load(&slot_map(&SLOT, $param1))
        }
        pub fn set($param1: &U, x: &U) {
            #[cfg(feature = "tracing")]
            dbg!(module_path!(), "setting", $param1, x);
            storage_store(&slot_map(&SLOT, $param1), x)
        }
        pub fn add($param1: &U, x: &U) -> Option<()> {
            #[cfg(feature = "tracing")]
            dbg!(module_path!(), "adding", $param1, x);
            storage_checked_add(&slot_map(&SLOT, $param1), x)
        }
        pub fn sub($param1: &U, x: &U) -> Option<()> {
            #[cfg(feature = "tracing")]
            dbg!(module_path!(), "subbing", $param1, x);
            storage_checked_sub(&slot_map(&SLOT, $param1), x)
        }
        pub fn get_hash($param1: &[u8; 64]) -> U {
            let x: [u8; 32] = $param1[..32].try_into().unwrap();
            get(&U(x))
        }
    };
    (@impl [$param1:ident, $param2:ident]) => {
        pub fn get($param1: &U, $param2: &U) -> U {
            storage_load(&slot_map(&slot_map(&SLOT, $param1), $param2))
        }
        pub fn set($param1: &U, $param2: &U, x: &U) {
            #[cfg(feature = "tracing")]
            dbg!(module_path!(), "setting", $param1, $param2, x);
            storage_store(&slot_map(&slot_map(&SLOT, $param1), $param2), x)
        }
        pub fn add($param1: &U, $param2: &U, x: &U) -> Option<()> {
            #[cfg(feature = "tracing")]
            dbg!(module_path!(), "adding", $param1, $param2, x);
            storage_checked_add(&slot_map(&slot_map(&SLOT, $param1), $param2), x)
        }
        pub fn sub($param1: &U, $param2: &U, x: &U) -> Option<()> {
            #[cfg(feature = "tracing")]
            dbg!(module_path!(), "subbing", $param1, $param2, x);
            storage_checked_sub(&slot_map(&slot_map(&SLOT, $param1), $param2), x)
        }
    };
    (@impl [$param1:ident, $param2:ident, $param3:ident]) => {
        pub fn get($param1: &U, $param2: &U, $param3: &U) -> U {
            storage_load(&slot_map(&slot_map(&slot_map(&SLOT, $param1), $param2), $param3))
        }
        pub fn set($param1: &U, $param2: &U, $param3: &U, x: &U) {
            #[cfg(feature = "tracing")]
            dbg!(module_path!(), "setting", $param1, $param2, $param3, x);
            storage_store(&slot_map(&slot_map(&slot_map(&SLOT, $param1), $param2), $param3), x)
        }
        pub fn add($param1: &U, $param2: &U, $param3: &U, x: &U) -> Option<()> {
            #[cfg(feature = "tracing")]
            dbg!(module_path!(), "adding", $param1, $param2, $param3, x);
            storage_checked_add(&slot_map(&slot_map(&slot_map(&SLOT, $param1), $param2), $param3), x)
        }
        pub fn sub($param1: &U, $param2: &U, $param3: &U, x: &U) -> Option<()> {
            #[cfg(feature = "tracing")]
            dbg!(module_path!(), "subbing", $param1, $param2, $param3, x);
            storage_checked_sub(&slot_map(&slot_map(&slot_map(&SLOT, $param1), $param2), $param3), x)
        }
        pub fn get_hash($param1: &U, $param2: &U, $param3: &[u8; 64]) -> U {
            let x: [u8; 32] = $param3[..32].try_into().unwrap();
            get($param1, $param2, &U(x))
        }
    };
}

storage! {
    contract_owner(),
    ed25519_count(),
    ed25519_keys(id),
    ed25519_owners(id),
    hash_owner_l(hash),
    hash_owner_r(hash),
    interim_amt(owner, asset, hash),
    order_amt(owner, asset, hash),
    withdrawable(owner, asset),
    hash_asset_l(hash),
    hash_asset_r(hash),
    hash_desired_amt(hash)
}

pub fn find_ed25519_key(i: &U) -> Result<U, Error> {
    let v = ed25519_keys::get(i);
    if v.is_zero() {
        Err(Error::from(ErrorDiscriminant::AccountIdNotFound))
    } else {
        // NOTE this doesn't ensure the key is valid
        Ok(v)
    }
}

pub fn find_ed25519_addr(i: &U) -> Result<Address, Error> {
    let addr: Address = ed25519_owners::get(i).into();
    if addr == [0u8; 20] {
        Err(Error::from(ErrorDiscriminant::AccountIdNotFound))
    } else {
        Ok(addr)
    }
}

pub fn ensure_hash_unseen(hash: &[u8; 64]) -> Option<()> {
    let hash: [u8; 32] = hash[..32].try_into().unwrap();
    if hash_owner_l::get(&U(hash)).is_some() {
        return None;
    }
    Some(())
}

pub struct Storage;
