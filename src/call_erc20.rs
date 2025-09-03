use crate::error::*;

use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::sol,
    stylus_core::Host,
};

#[cfg(target_arch = "wasm32")]
use {
    alloc::vec::Vec,
    stylus_sdk::{alloy_sol_types::SolCall, prelude::calls::context::Call},
};

sol! {
    function balanceOf(address) external view returns (uint256);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
    function transfer(address recipient, uint256 amount) external returns (bool);
}

#[cfg(target_arch = "wasm32")]
#[allow(unused)]
mod implem {
    use super::*;

    pub fn balance_of(host: &dyn Host, addr: Address, owner: Address) -> Result<U256, Error> {
        let rd = host
            .static_call(
                &Call::new(),
                addr,
                &balanceOfCall { _0: owner }.abi_encode(),
            )
            .map_err(|cd| Error {
                typ: ErrorDiscriminant::Erc20BalanceOfCall,
                cd: cd.into(),
            })?;
        let balanceOfReturn { _0 } =
            balanceOfCall::abi_decode_returns(&rd, true).map_err(|_| Error {
                typ: ErrorDiscriminant::Erc20BalanceOfDecode,
                cd: rd,
            })?;
        Ok(_0)
    }

    pub fn transfer_from(
        host: &dyn Host,
        addr: Address,
        from: Address,
        to: Address,
        amt: U256,
    ) -> Result<(), Error> {
        let rd = host
            .call(
                &Call::new(),
                addr,
                &transferFromCall {
                    from,
                    to,
                    amount: amt,
                }
                .abi_encode(),
            )
            .map_err(|cd| Error {
                typ: ErrorDiscriminant::Erc20TransferFromCall,
                cd: cd.into(),
            })?;
        let transferFromReturn { _0 } =
            transferFromCall::abi_decode_returns(&rd, true).map_err(|_| Error {
                typ: ErrorDiscriminant::Erc20TransferFromDecode,
                cd: rd,
            })?;
        if !_0 {
            return Err(Error {
                typ: ErrorDiscriminant::Erc20TransferFromFalse,
                cd: Vec::new(),
            });
        }
        Ok(())
    }

    pub fn transfer(
        host: &dyn Host,
        addr: Address,
        recipient: Address,
        amt: U256,
    ) -> Result<(), Error> {
        let rd = host
            .call(
                &Call::new(),
                addr,
                &transferCall {
                    recipient,
                    amount: amt,
                }
                .abi_encode(),
            )
            .map_err(|cd| Error {
                typ: ErrorDiscriminant::Erc20TransferCall,
                cd: cd.into(),
            })?;
        if rd.len() == 0 {
            return Ok(());
        }
        let transferReturn { _0 } =
            transferCall::abi_decode_returns(&rd, true).map_err(|_| Error {
                typ: ErrorDiscriminant::Erc20TransferDecode,
                cd: Vec::new(),
            })?;
        if !_0 {
            return Err(Error {
                typ: ErrorDiscriminant::Erc20TransferFalse,
                cd: Vec::new(),
            });
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused)]
mod implem {
    use super::*;

    pub fn balance_of(_vm: &dyn Host, _addr: Address, _owner: Address) -> Result<U256, Error> {
        Ok(U256::ZERO)
    }

    pub fn transfer_from(
        _host: &dyn Host,
        _addr: Address,
        _from: Address,
        _to: Address,
        _amt: U256,
    ) -> Result<(), Error> {
        Ok(())
    }

    pub fn transfer(
        _host: &dyn Host,
        _addr: Address,
        _recipient: Address,
        _amt: U256,
    ) -> Result<(), Error> {
        Ok(())
    }
}

pub use implem::*;
