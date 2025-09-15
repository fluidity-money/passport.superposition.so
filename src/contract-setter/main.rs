#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{entry, Op, DONE_UNIT};

#[cfg(target_arch = "wasm32")]
#[mutants::skip]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(len, |_, op| match op {
        Op::Dummy => DONE_UNIT,
        _ => panic!()
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
