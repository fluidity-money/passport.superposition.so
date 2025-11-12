use crate::state_machine::StateMachine;

use bobcat_sdk::{maths::U, storage::const_slot_off_curve};

#[cfg(target_arch = "wasm32")]
use bobcat_sdk::{
    call::delegate_call_vec,
    storage::{flush_cache, storage_load},
};

use alloc::vec::Vec;

pub const SLOT_APPLY: U = const_slot_off_curve(b"passport.superposition.impl.apply");

#[cfg(target_arch = "wasm32")]
pub fn begin_apply(s: StateMachine) -> (usize, Vec<u8>) {
    flush_cache();
    let (rc, v) = delegate_call_vec(
        storage_load(&SLOT_APPLY).into(),
        &borsh::to_vec(&s).unwrap(),
        u64::MAX,
        0,
    );
    if !rc {
        (1, v)
    } else {
        (0, v)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn begin_apply(_s: StateMachine) -> (usize, Vec<u8>) {
    todo!()
}
