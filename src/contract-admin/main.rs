#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{entry, ops::OpAdmin};

use borsh::BorshDeserialize;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(len, |_, args| match OpAdmin::deserialize(args).unwrap() {
        OpAdmin::Upgrade(_, _, _, _) => todo!(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
