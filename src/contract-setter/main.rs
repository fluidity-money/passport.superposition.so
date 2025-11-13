#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use bobcat_sdk::{entry::write_result_slice, storage::flush_cache};

use libpassport::{
    add_liq::add_liq, entry_non_reentrant, onboard::onboard, ops::OpSetter, DONE_UNIT,
};

use borsh::BorshDeserialize;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

extern crate alloc;

pub fn entry(len: usize) -> usize {
    entry_non_reentrant(len, |args| {
        let r = match OpSetter::deserialize(args).unwrap() {
            OpSetter::Dummy => DONE_UNIT,
            OpSetter::Onboard(
                key,
                sig,
                contract,
                nonce,
                chain,
                token,
                value,
                deadline,
                permit_v,
                permit_r,
                permit_s,
            ) => onboard(
                *key, sig, contract, nonce, chain, token, value, deadline, permit_v, permit_r,
                permit_s,
            ),
            OpSetter::AddLiquidity(token, recipient, value, deadline, v, r, s_) => {
                add_liq(token, recipient, value, deadline, v, r, s_)
            }
        };
        let rd = match r {
            Ok(_) => 0,
            Err(ref _reason) => {
                #[cfg(feature = "harness-stylus-interpreter")]
                panic!("reverted: {_reason:?}");
                #[allow(unreachable_code)]
                1
            }
        };
        match r {
            Ok(v) => write_result_slice(&borsh::to_vec(&v).unwrap()),
            Err(v) => write_result_slice(&{
                #[cfg(feature = "errors-extra-context")]
                {
                    borsh::to_vec(&v).unwrap()
                }
                #[cfg(not(feature = "errors-extra-context"))]
                {
                    [v.dis_u8().into()]
                }
            }),
        }
        flush_cache();
        rd
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(len)
}

#[allow(unused)]
fn main() {}

