#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::entry_solver;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

extern crate alloc;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry_solver(len)
}

#[allow(unused)]
fn main() {}
