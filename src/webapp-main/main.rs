use libpassport::ops::OpSolver;

#[unsafe(no_mangle)]
pub unsafe fn encode_dummy() -> Vec<u8> {
    borsh::to_vec(&OpSolver::Dummy).unwrap()
}

fn main() {}
