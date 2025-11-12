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

pub mod add_liq;
pub mod onboard;

pub mod call_eip20_extras;

pub type OurLzss = lzss::Lzss<12, 11, 0, { 1 << 12 }, { 2 << 12 }>;

use bobcat_sdk::{storage::const_keccak256, entry::read_args_vec};

#[cfg(feature = "std")]
use clap::Parser as ClapParser;

use core::str::FromStr;

pub use crate::error::{done_u64, DONE_UNIT, NOOP, R};

#[allow(unused_imports)]
use alloc::boxed::Box;

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

pub const REENTRANCY_CANARY: [u8; 32] = const_keccak256(b"superposition.passport.reentrancy-canary").0;

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

#[derive(Debug, Clone, PartialEq)]
pub struct ArgsAddr(pub [u8; 20]);

impl FromStr for ArgsAddr {
    type Err = const_hex::FromHexError;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        const_hex::decode_to_array::<_, 20>(x).map(|x| ArgsAddr(x))
    }
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
    pub sender: ArgsAddr,
    #[arg(short, long, default_value = "98985")]
    pub chain_id: u64,
    #[arg(
        short,
        long,
        default_value = "0x0000000000000000000000000000000000000000"
    )]
    pub addr: ArgsAddr,
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
    c
}
