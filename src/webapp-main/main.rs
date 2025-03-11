
#![cfg_attr(target_arch = "wasm32", no_main)]

use libpassport::*;

#[no_mangle]
pub fn encode_dummy() -> Vec<u8> {
    borsh::to_vec(&Op::Dummy).unwrap()
}

fn main() {}
