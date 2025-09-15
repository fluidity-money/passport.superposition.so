#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{
    entry,
    ops::{Op, PermitBlob},
    DONE_UNIT,
};

#[cfg(target_arch = "wasm32")]
#[mutants::skip]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(len, |s, op| match op {
        Op::Dummy => DONE_UNIT,
        Op::Onboard(
            key,
            owner,
            PermitBlob {
                value,
                deadline,
                v,
                r,
                s: s_,
            },
        ) => s.onboard(key, owner, value, deadline, v, r, s_),
        _ => panic!(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
