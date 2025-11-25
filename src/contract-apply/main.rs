#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::entry_apply;

#[global_allocator]
#[cfg(target_arch = "wasm32")]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry_apply(len)
}

#[allow(unused)]
fn main() {}
