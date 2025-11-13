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

use bobcat_sdk::{entry::read_args_vec, storage::reentrancy_guard_const_keccak};

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

#[derive(Debug, Clone, PartialEq)]
pub struct ArgsAddr(pub [u8; 20]);

#[derive(Debug, Clone, PartialEq)]
pub struct FromStrErr;

impl core::fmt::Display for FromStrErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for FromStrErr {}

impl FromStr for ArgsAddr {
    type Err = FromStrErr;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        const_hex::decode_to_array::<_, 20>(x)
            .map(|x| ArgsAddr(x))
            .map_err(|_| FromStrErr)
    }
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, ClapParser)]
#[command(version, about)]
pub struct VmArgs {
    #[arg(
        short,
        long,
        default_value = "0xfeb6034fc7df27df18a3a6bad5fb94c0d3dcb6d5",
        value_parser = ArgsAddr::from_str
    )]
    pub sender: ArgsAddr,
    #[arg(short, long, default_value = "98985")]
    pub chain_id: u64,
    #[arg(
        short,
        long,
        default_value = "0x0000000000000000000000000000000000000000",
        value_parser = ArgsAddr::from_str
    )]
    pub addr: ArgsAddr,
}

pub fn entry_non_reentrant(len: usize, entry: impl FnOnce(&mut &[u8]) -> usize) -> usize {
    let args = read_args_vec(len);
    reentrancy_guard_const_keccak(b"superposition.passport.reentrancy-canary", || {
        let c = entry(
            &mut OurLzss::decompress_stack(
                lzss::SliceReader::new(&args[1..]),
                lzss::VecWriter::with_capacity(1024 * 10),
            )
            .unwrap()
            .as_slice(),
        );
        c
    })
}
