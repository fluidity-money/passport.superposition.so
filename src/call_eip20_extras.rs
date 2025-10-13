use crate::error::*;

use bobcat_sdk::{
    call::{call_unit_opt, safe_call_bool_opt},
    interfaces::{
        eip20::{make_fn_transfer, make_fn_transfer_from},
        eip2612::make_fn_permit,
    },
    maths::U,
};

type Address = [u8; 20];

#[cfg(all(target_arch = "wasm32", not(feature = "dryrun")))]
#[allow(unused)]
mod implem {
    use super::*;

    pub fn transfer(addr: [u8; 20], recipient: [u8; 20], amt: U) -> Result<(), Error> {
        safe_call_bool_opt(addr, &make_fn_transfer(recipient, &amt), &U::ZERO, u64::MAX)
            .ok_or(Error::from(ErrorDiscriminant::Erc20Invoke))
    }

    pub fn transfer_from(addr: Address, from: Address, to: Address, amt: U) -> Result<(), Error> {
        safe_call_bool_opt(
            addr,
            &make_fn_transfer_from(from, to, &amt),
            &U::ZERO,
            u64::MAX,
        )
        .ok_or(Error::from(ErrorDiscriminant::Erc20Invoke))
    }

    pub fn permit(
        addr: Address,
        owner: Address,
        spender: Address,
        value: U,
        deadline: U,
        v: u8,
        r: U,
        s: U,
    ) -> Result<(), Error> {
        call_unit_opt(
            addr,
            &make_fn_permit(owner, spender, &value, &deadline, v, &r, &s),
            &U::ZERO,
            u64::MAX,
        )
        .ok_or(Error::from(ErrorDiscriminant::Erc20Invoke))
    }
}

#[cfg(feature = "dryrun")]
#[allow(unused)]
mod implem {
    use super::*;

    pub fn transfer(addr: [u8; 20], recipient: [u8; 20], amt: U) -> Result<(), Error> {
        todo!()
    }

    pub fn transfer_from(addr: Address, from: Address, to: Address, amt: U) -> Result<(), Error> {
        todo!()
    }

    pub fn permit(
        addr: Address,
        owner: Address,
        spender: Address,
        value: U,
        deadline: U,
        v: u8,
        r: U,
        s: U,
    ) -> Result<(), Error> {
        todo!()
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "dryrun")))]
#[allow(unused)]
mod implem {
    use super::*;

    pub fn transfer(addr: [u8; 20], recipient: [u8; 20], amt: U) -> Result<(), Error> {
        todo!()
    }

    pub fn transfer_from(addr: Address, from: Address, to: Address, amt: U) -> Result<(), Error> {
        todo!()
    }

    pub fn permit(
        addr: Address,
        owner: Address,
        spender: Address,
        value: U,
        deadline: U,
        v: u8,
        r: U,
        s: U,
    ) -> Result<(), Error> {
        todo!()
    }
}

pub use implem::*;
