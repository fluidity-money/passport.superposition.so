use crate::error::*;

use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256},
    prelude::HostAccess,
};

use stylus_sdk::alloy_sol_types::sol;

#[cfg(target_arch = "wasm32")]
use stylus_sdk::alloy_sol_types::SolCall;

// Some of the code here was handwritten to reduce codesize for the
// solver path.

sol! {
    function transferFrom(address spender, address recipient, uint256 amount) external returns (bool);
    function permit(address owner, address spender, uint value, uint deadline, uint8 v, bytes32 r, bytes32 s) external;
}

#[cfg(target_arch = "wasm32")]
#[allow(unused)]
mod implem {
    use super::*;

    use stylus_sdk::{call::call, prelude::TopLevelStorage, stylus_core::Call};

    pub fn transfer(
        env: &mut (impl TopLevelStorage + HostAccess),
        addr: Address,
        recipient: Address,
        amt: U256,
    ) -> Result<(), Error> {
        if env.vm().code_size(addr) == 0 {
            return Err(Error {
                typ: ErrorDiscriminant::TokenNoCode,
            });
        }
        let sel = [0xa9, 0x05, 0x9c, 0xbb];
        let mut b = [0u8; 32 * 2 + 4];
        b[..4].copy_from_slice(&sel);
        b[4 + 12..4 + 12 + 20].copy_from_slice(recipient.as_slice());
        let c = Call::new_mutating(env);
        b[4 + 32..].copy_from_slice(&amt.to_be_bytes() as &[u8; 32]);
        let rd = call(env.vm(), c, addr, &b).map_err(|_| Error {
            typ: ErrorDiscriminant::Erc20TransferCall,
        })?;
        if rd.len() == 0 {
            return Ok(());
        }
        Ok(())
    }

    pub fn transfer_from(
        env: &mut (impl TopLevelStorage + HostAccess),
        addr: Address,
        spender: Address,
        recipient: Address,
        amt: U256,
    ) -> Result<(), Error> {
        if env.vm().code_size(addr) == 0 {
            return Err(Error {
                typ: ErrorDiscriminant::TokenNoCode,
            });
        }
        let c = Call::new_mutating(env);
        let rd = call(
            env.vm(),
            c,
            addr,
            &transferFromCall {
                spender,
                recipient,
                amount: amt,
            }
            .abi_encode(),
        )?;
        if rd.len() == 0 {
            return Ok(());
        }
        if rd.len() != 32 {
            return Err(Error {
                typ: ErrorDiscriminant::Erc20TransferFromDecode,
            });
        }
        if rd[31] != 1 {
            return Err(Error {
                typ: ErrorDiscriminant::Erc20TransferFromFalse,
            });
        }
        Ok(())
    }

    pub fn permit(
        env: &mut (impl TopLevelStorage + HostAccess),
        addr: Address,
        owner: Address,
        spender: Address,
        value: U256,
        deadline: U256,
        v: u8,
        r: FixedBytes<32>,
        s: FixedBytes<32>,
    ) -> Result<(), Error> {
        // We don't check the code on this one since the transferFrom would fail
        // if there isn't anything here.
        let c = Call::new_mutating(env);
        let _ = call(
            env.vm(),
            c,
            addr,
            &permitCall {
                owner,
                spender,
                value,
                deadline,
                v,
                r,
                s,
            }
            .abi_encode(),
        )?;
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused)]
mod implem {
    use crate::storage::StorageApplicationV1;

    use super::*;

    pub fn give(env: &mut StorageApplicationV1, addr: Address, owner: Address, amt: U256) {
        env.test_eip20
            .balances
            .setter(addr)
            .setter(owner)
            .update_check_add(amt)
            .unwrap();
    }

    fn _transfer(
        env: &mut StorageApplicationV1,
        addr: Address,
        from: Address,
        recipient: Address,
        amt: U256,
    ) -> Result<(), Error> {
        env.test_eip20
            .balances
            .setter(addr)
            .setter(from)
            .update_check_sub(amt)
            .expect("Not enough sending balance");
        env.test_eip20
            .balances
            .setter(addr)
            .setter(recipient)
            .update_check_add(amt)
            .expect("Too much sent");
        Ok(())
    }

    pub fn transfer_from(
        env: &mut StorageApplicationV1,
        addr: Address,
        from: Address,
        to: Address,
        amt: U256,
    ) -> Result<(), Error> {
        if env
            .test_eip20
            .allowances
            .getter(addr)
            .getter(from)
            .get(env.vm().contract_address())
            < amt
        {
            panic!("Not enough allowance for the spend");
        }
        _transfer(env, addr, env.vm().contract_address(), to, amt)
    }

    pub fn transfer(
        env: &mut StorageApplicationV1,
        addr: Address,
        recipient: Address,
        amt: U256,
    ) -> Result<(), Error> {
        _transfer(env, addr, env.vm().contract_address(), recipient, amt)
    }

    pub fn permit(
        _env: &mut StorageApplicationV1,
        _addr: Address,
        _owner: Address,
        _spender: Address,
        _value: U256,
        _deadline: U256,
        _v: u8,
        _r: FixedBytes<32>,
        _s: FixedBytes<32>,
    ) -> Result<(), Error> {
        todo!()
    }
}

pub use implem::*;
