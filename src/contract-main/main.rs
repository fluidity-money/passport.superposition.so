
#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

pub use libpassport::*;

#[cfg(all(feature = "harness-stylus-interpreter", target_arch = "wasm32"))]
#[mutants::skip]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let msg = format!("{}", info);
    unsafe { lib9lives::die(msg.as_ptr(), msg.len(), 1) }
    core::arch::wasm32::unreachable()
}

#[cfg(all(not(feature = "harness-stylus-interpreter"), target_arch = "wasm32"))]
#[mutants::skip]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
