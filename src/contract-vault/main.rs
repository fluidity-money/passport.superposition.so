#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{entry_non_reentrant, ops::OpVault};

use borsh::BorshDeserialize;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

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

#[allow(unused)]
fn main() {}
