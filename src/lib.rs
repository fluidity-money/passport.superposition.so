#![cfg_attr(target_arch = "wasm32", no_std)]
// We don't instantiate the VM context so this is needed.
#![allow(deprecated)]

extern crate alloc;

pub mod error;
pub mod result;

pub mod apply;
pub mod conversion;

pub mod solver_context;
pub mod user_context;

pub mod immutables;

pub mod applicative;
pub mod state_machine;

pub mod ops;
pub mod storage;

pub mod utils;

mod call_erc20;

use stylus_sdk::alloy_sol_types::sol;

sol!("src/IErrors.sol");

pub type OurLzss = lzss::Lzss<12, 11, 0, { 1 << 12 }, { 2 << 12 }>;

use borsh::BorshDeserialize;

use stylus_sdk::{alloy_sol_types::SolError, prelude::HostAccess};

#[cfg(target_arch = "wasm32")]
use stylus_sdk::prelude::CalldataAccess;

pub use crate::{error::DONE_UNIT, ops::Op, storage::Storage};

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "vm_hooks")]
extern "C" {
    fn pay_for_memory_grow(pages: u16);
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe fn mark_used() {
    pay_for_memory_grow(0);
    panic!();
}

#[no_mangle]
pub extern "C" fn user_entrypoint(len: usize) -> usize {
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
    let args = OurLzss::decompress_stack(
        lzss::SliceReader::new(&args),
        lzss::VecWriter::with_capacity(1024 * 10),
    )
    .unwrap();
    let mut s = unsafe {
        <Storage as stylus_sdk::storage::StorageType>::new(
            stylus_sdk::alloy_primitives::U256::ZERO,
            0,
            vm,
        )
    };
    let r = match Op::deserialize(&mut (&args as &[u8])).unwrap() {
        Op::Dummy => DONE_UNIT,
        Op::Solve(accounts, args) => s
            .validate(&accounts, args)
            .and_then(|x| s.apply(x))
            .and_then(|_| DONE_UNIT),
    };
    let rd = match r {
        Ok(_) => 0,
        Err(_) => 1,
    };
    s.vm().write_result(&match r {
        Ok(v) => borsh::to_vec(&v).unwrap(),
        Err(v) => PassportError(borsh::to_vec(&v).unwrap().into()).abi_encode(),
    });
    s.vm().flush_cache(true);
    rd
}
