#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{entry_non_reentrant, ops::OpSetter, DONE_UNIT};

use stylus_sdk::prelude::HostAccess;

use borsh::BorshDeserialize;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry_non_reentrant(len, |s, args| {
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
            ) => s.onboard(
                key, sig, contract, nonce, chain, token, value, deadline, permit_v, permit_r,
                permit_s,
            ),
            OpSetter::AddLiquidity(token, recipient, value, deadline, v, r, s_) => {
                s.app.add_liq(token, recipient, value, deadline, v, r, s_)
            }
        };

        let rd = match r {
            Ok(ref _result) =>
            {
                #[allow(unreachable_code)]
                0
            }
            Err(ref _reason) => {
                #[cfg(feature = "harness-stylus-interpreter")]
                panic!("reverted: {_reason:?}");
                #[allow(unreachable_code)]
                1
            }
        };
        match r {
            Ok(v) => s.vm().write_result(&borsh::to_vec(&v).unwrap()),
            Err(v) => {
                s.vm().write_result(&[v.dis_u8().into()])
            }
        }
        s.vm().flush_cache(true);
        rd
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
