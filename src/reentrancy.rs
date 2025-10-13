use crate::state_machine::StateMachine;

use bobcat_sdk::{storage::{flush_cache, const_slot_off_curve}, maths::U};

use alloc::vec::Vec;

pub const SLOT_APPLY: U = const_slot_off_curve(b"passport.superposition.impl.apply");

#[cfg(target_arch = "wasm32")]
pub fn begin_apply(
    _s: StateMachine,
) -> (usize, Vec<u8>) {
/*
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
    } */
    flush_cache();
    todo!()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn begin_apply(
    _s: StateMachine,
) -> (usize, Vec<u8>) {
    todo!()
}
