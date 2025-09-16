#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{entry, ops::OpVault};

use borsh::BorshDeserialize;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(len, |_, args| match OpVault::deserialize(args).unwrap() {
        OpVault::MoveLiquidity => todo!()
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
