use crate::error::*;

use bobcat_sdk::maths::U;

type Address = [u8; 20];

#[cfg(all(target_arch = "wasm32", not(feature = "dryrun")))]
#[allow(unused)]
mod implem {
    use super::*;

    use bobcat_sdk::{
        call::{call_unit_opt, safe_call_bool_opt},
        interfaces::{
            eip20::{make_fn_transfer, make_fn_transfer_from},
            eip2612::make_fn_permit,
        },
    };

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

    use std::{cell::RefCell, collections::HashMap};

    use bobcat_sdk::entry::contract_address;

    thread_local! {
        static BALANCES: RefCell<HashMap<Address, U>> = RefCell::default();
    }

    pub fn balance_of(spender: Address) -> U {
        BALANCES
            .with(|b| b.borrow().get(&spender).map(|x| *x))
            .unwrap_or_default()
    }

    pub fn transfer_from(_: Address, from: Address, to: Address, amt: &U) -> Option<()> {
        let amt = *amt;
        BALANCES.with(|b| {
            let mut b = b.borrow_mut();
            let from_bal = b.get_mut(&from)?;
            if *from_bal < amt {
                return None;
            }
            *from_bal -= amt;
            b.entry(to).and_modify(|v| *v += amt).or_insert(amt);
            Some(())
        })
    }

    pub fn transfer(addr: Address, recipient: Address, amt: &U) -> Option<()> {
        transfer_from(addr, contract_address(), recipient, amt)
    }

    pub fn clear() {
        BALANCES.with(|b| b.borrow_mut().clear())
    }

    pub fn give(recipient: Address, amt: U) {
        BALANCES.with(|b| {
            *b.borrow_mut()
                .entry(recipient)
                .and_modify(|v| *v += amt)
                .or_insert(amt)
        });
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
        Ok(())
    }
}

pub use implem::*;
