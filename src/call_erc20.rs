use crate::error::*;

use stylus_sdk::{
    alloy_primitives::{Address, U256},
    stylus_core::{Host, Call},
};

#[cfg(target_arch = "wasm32")]
#[allow(unused)]
mod implem {
    use super::*;

    pub fn transfer(
        host: &dyn Host,
        addr: Address,
        recipient: Address,
        amt: U256,
    ) -> Result<(), Error> {
        let sel = [0xa9, 0x05, 0x9c, 0xbb];
        let mut b = [0u8; 32 * 2 + 4];
        b[..4].copy_from_slice(&sel);
        b[4 + 12..4 + 12 + 20].copy_from_slice(recipient.as_slice());
        b[4 + 32..].copy_from_slice(&amt.to_be_bytes() as &[u8; 32]);
        let rd = host.call(&Call::new(), addr, &b).map_err(|_| Error {
            typ: ErrorDiscriminant::Erc20TransferCall,
        })?;
        if rd.len() == 0 {
            return Ok(());
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused)]
mod implem {
    use super::*;

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
