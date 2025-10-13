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

use bobcat_sdk::entry::read_args_vec;

#[cfg(feature = "std")]
use clap::Parser as ClapParser;

#[cfg(feature = "std")]
use std::io::Read;

#[cfg(feature = "std")]
use stylus_sdk::alloy_primitives::Address;

pub use crate::error::{done_u64, DONE_UNIT, NOOP, R};

#[allow(unused_imports)]
use alloc::boxed::Box;

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "vm_hooks")]
#[allow(unused)]
unsafe extern "C" {
    pub(crate) fn pay_for_memory_grow(pages: u16);
}

#[cfg(all(target_arch = "wasm32", not(feature = "dryrun")))]
#[link(wasm_import_module = "vm_hooks")]
unsafe extern "C" {
    fn transient_load_bytes32(key: *const u8, dest: *const u8);
    fn transient_store_bytes32(key: *const u8, value: *const u8);
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

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
thread_local! {
    static VM_ARGS: std::cell::RefCell<Vec<u8>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static VM_RETURN: std::cell::RefCell<Vec<u8>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, ClapParser)]
#[command(version, about)]
pub struct VmArgs {
    #[arg(
        short,
        long,
        default_value = "0xfeb6034fc7df27df18a3a6bad5fb94c0d3dcb6d5"
    )]
    pub sender: Address,
    #[arg(short, long, default_value = "98985")]
    pub chain_id: u64,
    #[arg(
        short,
        long,
        default_value = "0x0000000000000000000000000000000000000000"
    )]
    pub addr: Address,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn host_vm_harness() -> (VM, usize) {
    #[allow(unused_mut)]
    let test_vm = stylus_sdk::testing::vm::TestVM::new();
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    let args_len = {
        let args = VmArgs::parse();
        test_vm.set_chain_id(args.chain_id);
        test_vm.set_sender(args.sender);
        test_vm.set_contract_address(args.addr);
        let mut b = String::new();
        std::io::stdin().read_to_string(&mut b).unwrap();
        let a = const_hex::decode(b.trim()).unwrap();
        let l = a.len();
        if !b.is_empty() {
            VM_ARGS.with(|x| *x.borrow_mut() = a)
        }
        l
    };
    let vm = VM {
        host: Box::new(test_vm),
    };
    #[cfg(not(all(feature = "std", not(target_arch = "wasm32"))))]
    let args_len = 0;
    (vm, args_len)
}

#[cfg(all(not(target_arch = "wasm32"), feature = "std"))]
pub fn return_data() -> Vec<u8> {
    VM_RETURN.with(|x| x.borrow().clone())
}

pub fn entry_non_reentrant(len: usize, entry: impl FnOnce(&mut &[u8]) -> usize) -> usize {
    let args = read_args_vec(len);
    // Blow up if we're reentrant!
    if is_reentrancy() {
        // Roll back the state, we shouldn't be here!
        return 1;
    }
    set_reentrancy_flag();
    let c = entry(
        &mut OurLzss::decompress_stack(
            lzss::SliceReader::new(&args[1..]),
            lzss::VecWriter::with_capacity(1024 * 10),
        )
        .unwrap()
        .as_slice(),
    );
    #[cfg(all(not(target_arch = "wasm32"), feature = "std"))]
    VM_RETURN.with(|x| *x.borrow_mut() = s.vm().read_return_data(0, None));
    c
}
