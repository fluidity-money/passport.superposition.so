#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

pub use libpassport::*;

extern crate alloc;

#[cfg(target_arch = "wasm32")]
#[mutants::skip]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let msg = alloc::format!("{}", info);
    stylus_sdk::console!(msg);
    core::arch::wasm32::unreachable()
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
