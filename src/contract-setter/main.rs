#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{entry_non_reentrant, ops::OpSetter, wasm_vm_harness, DONE_UNIT};

#[cfg(not(target_arch = "wasm32"))]
use libpassport::host_vm_harness;

use stylus_sdk::{host::VM, prelude::HostAccess};

use borsh::BorshDeserialize;

extern crate alloc;

pub fn entry(vm: VM, len: usize) -> usize {
    entry_non_reentrant(vm, len, |s, args| {
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
        stylus_panic::harness_dbg!("I MADE IT OUSIDET THIS FHTHTHRT");
        let rd = match r {
            Ok(_) => 0,
            Err(ref _reason) => {
                stylus_panic::harness_dbg!(_reason);
                #[cfg(feature = "harness-stylus-interpreter")]
                panic!("reverted: {_reason:?}");
                #[allow(unreachable_code)]
                1
            }
        };
        match r {
            Ok(v) => s.vm().write_result(&borsh::to_vec(&v).unwrap()),
            Err(v) => s.vm().write_result(&[v.dis_u8().into()]),
        }
        s.vm().flush_cache(true);
        rd
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(wasm_vm_harness(), len)
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let (vm, len) = host_vm_harness();
    std::process::exit(entry(vm, len).try_into().unwrap())
}
