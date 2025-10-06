#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{state_machine::StateMachine, Storage};

use borsh::de::BorshDeserialize;

#[cfg(target_arch = "wasm32")]
use stylus_sdk::prelude::CalldataAccess;

use stylus_sdk::prelude::HostAccess;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    #[cfg(target_arch = "wasm32")]
    let vm = stylus_sdk::host::VM(stylus_sdk::host::WasmVM {});
    #[cfg(not(target_arch = "wasm32"))]
    let vm = stylus_sdk::host::VM {
        host: Box::new(stylus_sdk::testing::vm::TestVM::new()),
    };
    #[cfg(target_arch = "wasm32")]
    let args = vm.read_args(len);
    #[cfg(not(target_arch = "wasm32"))]
    let args = vm.host.read_args(len);
    #[allow(unused_mut)]
    let mut s = unsafe {
        <Storage as stylus_sdk::storage::StorageType>::new(
            stylus_sdk::alloy_primitives::U256::ZERO,
            0,
            vm,
        )
    };
    let r = s
        .app
        .apply(StateMachine::deserialize(&mut args.as_slice()).unwrap());
    match r {
        Ok(v) => s.vm().write_result(&borsh::to_vec(&v).unwrap()),
        Err(v) => {
            s.vm().write_result(&[v.dis_u8()])
        }
    }
    s.vm().flush_cache(true);
    1
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
