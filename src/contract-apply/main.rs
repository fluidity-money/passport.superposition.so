#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use bobcat_sdk::{
    entry::{read_args_vec, write_result_slice},
    storage::flush_cache,
};

use libpassport::{apply::apply, state_machine::StateMachine};

use borsh::de::BorshDeserialize;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    let args = read_args_vec(len);
    let r = apply(StateMachine::deserialize(&mut args.as_slice()).unwrap());
    match r {
        Ok(v) => write_result_slice(&borsh::to_vec(&v).unwrap()),
        Err(v) => write_result_slice(&[v.dis_u8()]),
    }
    flush_cache();
    1
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
