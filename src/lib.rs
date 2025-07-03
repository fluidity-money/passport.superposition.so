#![cfg_attr(target_arch = "wasm32", no_std)]

// We don't instantiate the VM context so this is needed.
#![allow(deprecated)]

extern crate alloc;

pub mod entry;
pub mod error;
pub mod result;

pub mod crypto;

pub mod snowflake;
pub mod accounts;
pub mod applicative;
pub mod state_machine;
pub mod emissions;

pub mod encoding;
pub mod ops;
pub mod storage;

pub mod utils;

#[allow(unused)]
use {
    borsh::BorshDeserialize,
    stylus_sdk::alloy_sol_types::{sol, SolError},
};

#[cfg(target_arch = "wasm32")]
use crate::{ops::Op, storage::StoragePassport};

sol!("src/IErrors.sol");

#[no_mangle]
pub unsafe fn mark_used() {
    stylus_sdk::evm::pay_for_memory_grow(0);
    panic!();
}

#[no_mangle]
#[cfg(target_arch = "wasm32")]
pub extern "C" fn user_entrypoint(len: usize) -> usize {
    type OurLzss = lzss::Lzss<12, 11, 0, { 1 << 12 }, { 2 << 12 }>;
    let vm = stylus_sdk::host::VM(stylus_sdk::host::WasmVM {});
    let args = OurLzss::decompress_stack(
        lzss::SliceReader::new(&stylus_sdk::contract::args(len)),
        lzss::VecWriter::with_capacity(1024 * 10),
    )
    .unwrap();
    let mut store = unsafe {
        <StoragePassport as stylus_sdk::storage::StorageType>::new(
            stylus_sdk::alloy_primitives::U256::ZERO,
            0,
            vm,
        )
    };
    let r = match Op::deserialize(&mut (&args as &[u8])).unwrap() {
        Op::Dummy => store.dummy(),
        Op::QueryUnusedLiquidity(addr) => store.query_unused_liq(addr),
        Op::DepositUnusedLiquidity(l) => store.deposit_unused_liq(l),
        Op::Solve(args) => store.solve(args)
    };
    stylus_sdk::storage::StorageCache::flush();
    let rd = match r {
        Ok(_) => 0,
        Err(_) => 1,
    };
    stylus_sdk::contract::output(&match r {
        Ok(v) => borsh::to_vec(&v).unwrap(),
        Err(v) => PassportError {
            _0: borsh::to_vec(&v).unwrap().into(),
        }
        .abi_encode(),
    });
    rd
}

// We need this function due to a bug in the SDK if this is compiled for
// the native host.
#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub unsafe extern "C" fn native_keccak256(bytes: *const u8, len: usize, output: *mut u8) {
    use core::slice;
    use tiny_keccak::{Hasher, Keccak};
    let mut hasher = Keccak::v256();
    let data = unsafe { slice::from_raw_parts(bytes, len) };
    hasher.update(data);
    let output = unsafe { slice::from_raw_parts_mut(output, 32) };
    hasher.finalize(output);
}
