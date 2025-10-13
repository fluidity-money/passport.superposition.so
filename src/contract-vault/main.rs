#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{entry_non_reentrant, ops::OpVault};

#[cfg(not(target_arch = "wasm32"))]
use libpassport::host_vm_harness;

use borsh::BorshDeserialize;

pub fn entry(len: usize) -> usize {
    entry_non_reentrant(len, |args| {
        match OpVault::deserialize(args).unwrap() {
            OpVault::MoveLiquidity => todo!(),
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(len)
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let (vm, len) = host_vm_harness();
    std::process::exit(entry(vm, len).try_into().unwrap())
}
