#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod error;
pub mod result;

pub mod storage;

pub mod network;

pub mod apply;
pub mod conversion;

pub mod solver_context;
pub mod user_context;

pub mod sigs;

pub mod immutables;

pub mod reentrancy;

pub mod applicative;
pub mod state_machine;

pub mod facet;
pub mod ops;

pub mod utils;

pub mod add_liq;
pub mod onboard;

pub mod call_eip20_extras;

pub type OurLzss = lzss::Lzss<12, 11, 0, { 1 << 12 }, { 2 << 12 }>;

pub use stylus_panic;

#[cfg(target_arch = "wasm32")]
use stylus_sdk::prelude::CalldataAccess;

use stylus_sdk::host::VM;

#[cfg(feature = "std")]
use clap::Parser as ClapParser;

#[cfg(feature = "std")]
use std::io::Read;

#[cfg(feature = "std")]
use stylus_sdk::alloy_primitives::Address;

pub use crate::{
    error::{done_u64, DONE_UNIT, NOOP, R},
    storage::Storage,
};

#[allow(unused_imports)]
use alloc::boxed::Box;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "vm_hooks")]
#[allow(unused)]
extern "C" {
    pub(crate) fn pay_for_memory_grow(pages: u16);
}

#[cfg(all(target_arch = "wasm32", not(feature = "dryrun")))]
#[link(wasm_import_module = "vm_hooks")]
extern "C" {
    fn transient_load_bytes32(key: *const u8, dest: *const u8);
    fn transient_store_bytes32(key: *const u8, value: *const u8);
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe fn mark_used() {
    pay_for_memory_grow(0);
    panic!();
}

//uint256(keccak256(abi.encodePacked("superposition.passport.reentrancy-canary"))) - 1
//50613784340086862354587376166245909682637321245089807187397334771443192077707
pub const REENTRANCY_CANARY: [u8; 32] = match const_hex::const_decode_to_array(
    b"6fe66301d6923adbef88700a33e306a66ebe50db5cc5b0599eccfbcb3c5c9d8b",
) {
    Ok(v) => v,
    _ => panic!(),
};

#[cfg(all(target_arch = "wasm32", not(feature = "dryrun")))]
pub fn is_reentrancy() -> bool {
    let mut b = [0u8; 32];
    unsafe {
        transient_load_bytes32(REENTRANCY_CANARY.as_ptr(), b.as_mut_ptr());
    }
    b[31] == 1
}

#[cfg(any(not(target_arch = "wasm32"), feature = "dryrun"))]
pub fn is_reentrancy() -> bool {
    false
}

#[cfg(all(target_arch = "wasm32", not(feature = "dryrun")))]
fn set_reentrancy_flag() {
    let b = [1u8; 32];
    unsafe {
        transient_store_bytes32(REENTRANCY_CANARY.as_ptr(), b.as_ptr());
    }
}

#[cfg(any(not(target_arch = "wasm32"), feature = "dryrun"))]
fn set_reentrancy_flag() {}

pub fn wasm_vm_harness() -> VM {
    #[cfg(target_arch = "wasm32")]
    return VM(stylus_sdk::host::WasmVM {});
    #[cfg(not(target_arch = "wasm32"))]
    return VM {
        host: Box::new(stylus_sdk::host::WasmVM {}),
    };
}

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
thread_local! {
    static VM_ARGS: std::cell::RefCell<Vec<u8>> =
        const { std::cell::RefCell::new(Vec::new()) }
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, ClapParser)]
#[command(version, about)]
pub struct VmArgs {
    pub sender: Address,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn host_vm_harness() -> (VM, usize) {
    #[allow(unused_mut)]
    let mut vm = VM {
        host: Box::new(stylus_sdk::testing::vm::TestVM::new()),
    };
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    let args_len = {
        let args = VmArgs::parse();
        let mut b = Vec::new();
        let s = std::io::stdin().read_to_end(&mut b).unwrap();
        let a = const_hex::decode(s).unwrap();
        let l = a.len();
        if !s.is_empty() {
            VM_ARGS.with(|x| *x.borrow_mut() = a)
        }
        l
    };
    #[cfg(not(all(feature = "std", not(target_arch = "wasm32"))))]
    let args_len = 0;
    (vm, args_len)
}

pub fn entry_non_reentrant(
    vm: VM,
    len: usize,
    entry: impl FnOnce(&mut Storage, &mut &[u8]) -> usize,
) -> usize {
    #[cfg(all(not(target_arch = "wasm32"), feature = "std"))]
    let args = VM_ARGS.with(|x| x);
    #[cfg(target_arch = "wasm32")]
    let args = vm.read_args(len);
    // Blow up if we're reentrant! If someone is using this, they should
    // not tolerate reentrancy.
    if is_reentrancy() {
        // Roll back the state, we shouldn't be here!
        return 1;
    }
    set_reentrancy_flag();
    #[allow(unused_mut)]
    let mut s = unsafe {
        <Storage as stylus_sdk::storage::StorageType>::new(
            stylus_sdk::alloy_primitives::U256::ZERO,
            0,
            vm,
        )
    };
    entry(
        &mut s,
        &mut OurLzss::decompress_stack(
            lzss::SliceReader::new(&args[1..]),
            lzss::VecWriter::with_capacity(1024 * 10),
        )
        .unwrap()
        .as_slice(),
    )
}
