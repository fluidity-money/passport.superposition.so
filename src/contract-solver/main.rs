#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{entry, Op, DONE_UNIT};

#[cfg(target_arch = "wasm32")]
#[mutants::skip]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(len, |s, op| match op {
        Op::Dummy => DONE_UNIT,
        Op::Solve(accounts, args) => s
            .validate(&accounts, args)
            .and_then(|x| s.apply(x))
            .and_then(|_| DONE_UNIT),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
