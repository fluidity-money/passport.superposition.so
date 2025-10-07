use crate::state_machine::StateMachine;

use stylus_sdk::prelude::{HostAccess, TopLevelStorage};

#[cfg(target_arch = "wasm32")]
use stylus_sdk::{
    alloy_primitives::Address,
    prelude::{delegate_call, errors::Error},
    stylus_core::Call,
};

use alloc::vec::Vec;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "vm_hooks")]
extern "C" {
    fn storage_load_bytes32(key: *const u8, out: *mut u8);
}

pub const SLOT_APPLY: [u8; 32] = match const_hex::const_decode_to_array::<32>(
    b"b1071da564e73d02ae6815965358af289c23a15d2fe16d58dc4a81c3101b4d79",
) {
    Ok(v) => v,
    Err(_) => panic!(),
};

#[cfg(target_arch = "wasm32")]
pub fn begin_apply(
    env: &mut (impl TopLevelStorage + HostAccess),
    s: StateMachine,
) -> (usize, Vec<u8>) {
    let mut b = [0u8; 32];
    unsafe {
        storage_load_bytes32(SLOT_APPLY.as_ptr(), b.as_mut_ptr());
    }
    let addr: [u8; 20] = b[32 - 20..].try_into().unwrap();
    let addr = Address::from(addr);
    let c = Call::new_mutating(env);
    // The cache should already be flushed before the delegate child reverts!
    unsafe {
        match delegate_call(env.vm(), c, addr, &borsh::to_vec(&s).unwrap()) {
            Ok(b) => (0, b),
            Err(Error::Revert(b)) => (1, b),
            _ => unimplemented!(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn begin_apply(
    _env: &mut (impl TopLevelStorage + HostAccess),
    _s: StateMachine,
) -> (usize, Vec<u8>) {
    todo!()
}
