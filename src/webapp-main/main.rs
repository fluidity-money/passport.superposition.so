use libpassport::ops::Op;

#[unsafe(no_mangle)]
pub unsafe fn encode_dummy() -> Vec<u8> {
    borsh::to_vec(&Op::Dummy).unwrap()
}

fn main() {}
