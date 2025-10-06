use crate::state_machine::StateMachine;

use stylus_sdk::{
    alloy_primitives::Address,
    prelude::{delegate_call, errors::Error, HostAccess, TopLevelStorage},
    stylus_core::Call,
};

use alloc::vec::Vec;

#[cfg(target_arch = "wasm32")]
pub fn begin_apply(
    env: &mut (impl TopLevelStorage + HostAccess),
    addr: Address,
    s: StateMachine,
) -> (i32, Vec<u8>) {
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
