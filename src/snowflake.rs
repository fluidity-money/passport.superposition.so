use stylus_sdk::{
    alloy_primitives::{Address, U256},
    crypto::keccak,
};

pub fn snowflake(owner: Address, ms_ts: U256) -> U256 {
    let mut b = [0u8; 64];
    b.copy_from_slice(owner.as_slice());
    b.copy_from_slice(&ms_ts.to_be_bytes::<32>());
    U256::try_from(keccak(b)).unwrap()
}
