#![cfg_attr(target_arch = "wasm32", no_std)]
// We don't instantiate the VM context so this is needed.
#![allow(deprecated)]

extern crate alloc;

pub mod entry;
pub mod error;
pub mod result;

pub mod conversion;
pub mod apply;

pub mod solver_context;
pub mod user_context;

pub mod immutables;

pub mod accounts;

pub mod applicative;
pub mod state_machine;

pub mod ops;
pub mod storage;

pub mod utils;

mod call_erc20;

#[allow(unused)]
use {
    borsh::BorshDeserialize,
    stylus_sdk::alloy_sol_types::{SolError, sol},
};

pub use crate::{ops::Op, storage::StoragePassport};

sol!("src/IErrors.sol");

#[no_mangle]
pub unsafe fn mark_used() {
    stylus_sdk::evm::pay_for_memory_grow(0);
    panic!();
}

pub type OurLzss = lzss::Lzss<12, 11, 0, { 1 << 12 }, { 2 << 12 }>;

#[no_mangle]
#[cfg(target_arch = "wasm32")]
pub extern "C" fn user_entrypoint(len: usize) -> usize {
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
        Op::Solve(accounts, args) => store.solve(accounts, args),
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
