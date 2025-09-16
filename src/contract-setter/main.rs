#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{entry, ops::OpSetter, DONE_UNIT};

use borsh::BorshDeserialize;

#[cfg(target_arch = "wasm32")]
#[mutants::skip]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(len, |s, args| match OpSetter::deserialize(args).unwrap() {
        OpSetter::Dummy => DONE_UNIT,
        OpSetter::Onboard(
            key,
            sig,
            nonce,
            token,
            value,
            deadline,
            permit_v,
            permit_r,
            permit_s,
        ) => s.onboard(
            key, sig, nonce, token, value, deadline, permit_v, permit_r, permit_s,
        ),
        OpSetter::AddLiquidity(token, recipient, value, deadline, v, r, s_) => {
            s.add_liq(token, recipient, value, deadline, v, r, s_)
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
