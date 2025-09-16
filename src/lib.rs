#![cfg_attr(target_arch = "wasm32", no_std)]
// We don't instantiate the VM context so this is needed.

extern crate alloc;

pub mod error;
pub mod result;

pub mod storage;

pub mod apply;
pub mod conversion;

pub mod solver_context;
pub mod user_context;

pub mod immutables;

pub mod applicative;
pub mod state_machine;

pub mod facet;
pub mod ops;

pub mod utils;

pub mod add_liq;
pub mod onboard;

mod call_eip20_extras;

use stylus_sdk::alloy_sol_types::sol;

sol!("src/IErrors.sol");

pub type OurLzss = lzss::Lzss<12, 11, 0, { 1 << 12 }, { 2 << 12 }>;

use stylus_sdk::{alloy_sol_types::SolError, prelude::HostAccess};

#[cfg(target_arch = "wasm32")]
use stylus_sdk::prelude::CalldataAccess;

use crate::facet::Facet;

pub use crate::{
    error::{done_u64, DONE_UNIT, NOOP, R},
    storage::Storage,
};

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

#[cfg(all(not(feature = "std"), target_arch = "wasm32"))]
#[mutants::skip]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

pub fn entry(len: usize, simulate: impl FnOnce(&mut Storage, &mut &[u8]) -> R) -> usize {
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
    // Make sure we skip the first byte, which we assume is the magic byte
    // that was used to do the contract indirection using the proxy contract.
    // Note that everything after the admin facet is considered a reentrant facet.
    let mut args = if args[0] > Facet::UserAdmin as u8 {
        &args[1..]
    } else {
        &OurLzss::decompress_stack(
            lzss::SliceReader::new(&args[1..]),
            lzss::VecWriter::with_capacity(1024 * 10),
        )
        .unwrap()
    };
    #[allow(unused_mut)]
    let mut s = unsafe {
        <Storage as stylus_sdk::storage::StorageType>::new(
            stylus_sdk::alloy_primitives::U256::ZERO,
            0,
            vm,
        )
    };
    let r = simulate(&mut s, &mut args);
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
