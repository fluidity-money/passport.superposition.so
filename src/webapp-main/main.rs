
#![cfg_attr(target_arch = "wasm32", no_main)]

use libpassport::*;

#[unsafe(no_mangle)]
pub unsafe fn encode_dummy() -> Vec<u8> {
    borsh::to_vec(&Op::Dummy).unwrap()
}

fn main() {}
