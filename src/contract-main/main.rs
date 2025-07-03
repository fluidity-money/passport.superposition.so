#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

pub use libpassport::*;

extern crate alloc;

/*
#[cfg(target_arch = "wasm32")]
#[mutants::skip]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    stylus_sdk::console!(alloc::format!("{}", info));
    core::arch::wasm32::unreachable()
}*/

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
