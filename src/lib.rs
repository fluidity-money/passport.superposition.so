#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

pub mod error;
pub mod result;

pub mod storage;

pub mod network;

pub mod apply;
pub mod conversion;

pub mod solver_context;
pub mod user_context;

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
#[allow(unused)]
extern "C" {
    fn pay_for_memory_grow(pages: u16);
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

#[cfg(all(target_arch = "wasm32", feature = "harness-stylus-interpreter"))]
#[link(wasm_import_module = "stylus_interpreter")]
extern "C" {
    fn die(ptr: i32, len: i32, code: i32);
}

#[cfg(all(not(feature = "std"), target_arch = "wasm32"))]
#[mutants::skip]
#[panic_handler]
fn panic(_msg: &core::panic::PanicInfo) -> ! {
    #[cfg(feature = "harness-stylus-interpreter")]
    {
        let msg = alloc::format!("{_msg}");
        unsafe {
            die(msg.as_ptr() as i32, msg.len() as i32, 1)
        }
    }
    core::arch::wasm32::unreachable();
    #[allow(unreachable_code)]
    loop {}
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
fn is_reentrancy() -> bool {
    let mut b = [0u8; 32];
    unsafe {
        transient_load_bytes32(REENTRANCY_CANARY.as_ptr(), b.as_mut_ptr());
    }
    b[31] == 1
}

#[cfg(any(not(target_arch = "wasm32"), feature = "dryrun"))]
fn is_reentrancy() -> bool {
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

fn is_reentrant_facet(x: &Facet) -> bool {
    match x {
        Facet::UserSolver | Facet::UserSetter | Facet::UserAdmin => false,
        Facet::ReentrantVault => true,
    }
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
    let f = Facet::try_from(args[0]).unwrap();
    let are_we_reentrant = is_reentrancy();
    // Check the reentrancy canary to see if we're inside a facet that can be
    // used this way.
    if are_we_reentrant && !is_reentrant_facet(&f) {
        // Roll back the state, we shouldn't be here!
        return 1;
    }
    set_reentrancy_flag();
    let mut args = if is_reentrant_facet(&f) {
        // The reentrant calldata should be a single byte for the complex type
        // here, so we can avoid lots of overhead. The reentrant code should load
        // its parameters using transient storage.
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
