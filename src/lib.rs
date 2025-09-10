#![cfg_attr(target_arch = "wasm32", no_std)]
// We don't instantiate the VM context so this is needed.
#![allow(deprecated)]

extern crate alloc;

pub mod entry;
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

#[cfg(target_arch = "wasm32")]
mod implem {
    use borsh::BorshDeserialize;

    use stylus_sdk::{
        alloy_sol_types::SolError,
        prelude::{CalldataAccess, HostAccess},
    };

    pub use crate::{ops::Op, storage::Storage, PassportError};

    use super::OurLzss;

    #[link(wasm_import_module = "vm_hooks")]
    extern "C" {
        fn pay_for_memory_grow(pages: u16);
    }

    #[no_mangle]
    pub unsafe fn mark_used() {
        pay_for_memory_grow(0);
        panic!();
    }

    #[no_mangle]
    #[cfg(target_arch = "wasm32")]
    pub extern "C" fn user_entrypoint(len: usize) -> usize {
        let vm = stylus_sdk::host::VM(stylus_sdk::host::WasmVM {});
        let args = OurLzss::decompress_stack(
            lzss::SliceReader::new(&vm.read_args(len)),
            lzss::VecWriter::with_capacity(1024 * 10),
        )
        .unwrap();
        let mut store = unsafe {
            <Storage as stylus_sdk::storage::StorageType>::new(
                stylus_sdk::alloy_primitives::U256::ZERO,
                0,
                vm,
            )
        };
        let r = match Op::deserialize(&mut (&args as &[u8])).unwrap() {
            Op::Dummy => store.dummy(),
            Op::Solve(accounts, args) => store.solve(accounts, args),
        };
        let rd = match r {
            Ok(_) => 0,
            Err(_) => 1,
        };
        store.vm().write_result(&match r {
            Ok(v) => borsh::to_vec(&v).unwrap(),
            Err(v) => PassportError(borsh::to_vec(&v).unwrap().into()).abi_encode(),
        });
        store.vm().flush_cache(true);
        rd
    }
}

#[cfg(target_arch = "wasm32")]
pub use implem::*;
