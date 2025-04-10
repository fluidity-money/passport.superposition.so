use crate::error::*;

use stylus_sdk::{
    alloy_primitives::{address, Address},
    alloy_sol_types::{sol, SolCall},
    prelude::calls::{context::Call, CallAccess},
};

pub const ECRECOVER_ADDR: Address = address!("0000000000000000000000000000000000000001");

sol! {
    function ecrecover(bytes32 hash, bytes32 r, bytes32 s, uint8 v) returns (address addr);
}

fn pack_ecrecover(hash: &[u8; 32], r: &[u8; 32], s: &[u8; 32], v: u8) -> [u8; 32 * 4] {
    let mut b = [0u8; 32 * 4];
    b[..32].copy_from_slice(hash);
    b[64 - 4..64].copy_from_slice(&v.to_be_bytes());
    b[64..96].copy_from_slice(r);
    b[96..128].copy_from_slice(s);
    b
}

pub fn ecrecover(
    vm: &dyn CallAccess,
    hash: &[u8; 32],
    r: &[u8; 32],
    s: &[u8; 32],
    v: u8,
) -> Result<Address, Error> {
    let cd = pack_ecrecover(hash, r, s, v);
    Ok(ecrecoverCall::abi_decode_returns(
        &vm.static_call(&Call::new(), ECRECOVER_ADDR, &pack_ecrecover(hash, r, s, v))?,
        true,
    )?
    .addr)
}
