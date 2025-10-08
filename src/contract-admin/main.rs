#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{entry_non_reentrant, wasm_vm_harness, ops::OpAdmin};

#[cfg(not(target_arch = "wasm32"))]
use libpassport::host_vm_harenss;

use stylus_sdk::host::VM;

use borsh::BorshDeserialize;

pub fn entry(vm: VM, len: usize) -> usize {
    entry_non_reentrant(vm, len, |_, args| match OpAdmin::deserialize(args).unwrap() {
        OpAdmin::Upgrade(_, _, _, _) => todo!(),
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(wasm_vm_harness(), len)
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let (vm, len) = host_vm_harness();
    entry(vm, len)
}
