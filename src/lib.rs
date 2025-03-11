
#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;

pub mod entry;
pub mod error;
pub mod result;

pub mod encoding;
pub mod storage;

pub use encoding::*;
pub use entry::*;
pub use result::*;

use lzss::{Lzss, SliceReader, VecWriter};

use stylus_sdk::alloy_primitives::*;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Op {
    Spend { spendable: Vec<(BAddress, BU256)> },
    Delegate,
}

#[no_mangle]
pub unsafe fn mark_used() {
    stylus_sdk::evm::pay_for_memory_grow(0);
    panic!();
}

type OurLzss = Lzss<12, 11, 0, { 1 << 12 }, { 2 << 12 }>;

#[no_mangle]
pub extern "C" fn user_entrypoint(len: usize) -> usize {
    let args = OurLzss::decompress_stack(
        SliceReader::new(&stylus_sdk::contract::args(len)),
        VecWriter::with_capacity(1024 * 10),
    )
    .unwrap();
    let mut store =
        unsafe { <StoragePassport as stylus_sdk::storage::StorageType>::new(U256::ZERO, 0) };
    let r = match Op::deserialize(&mut (&args as &[u8])).unwrap() {
        Op::Spend { spendable } => store.spend(spendable),
        _ => unimplemented!(),
    };
    stylus_sdk::storage::StorageCache::flush();
    let rd = match r {
        Ok(_) => 0,
        Err(_) => 1,
    };
    stylus_sdk::contract::output(&match r {
        Ok(v) => borsh::to_vec(&v).unwrap(),
        Err(v) => borsh::to_vec(&v).unwrap(),
    });
    rd
}
