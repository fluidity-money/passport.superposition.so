use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256},
    crypto::keccak,
};

// Used for creating the EIP712 hash signature for the Bonding operation.
//keccak256("Bonding(bytes32 edSig,address ethAddr,uint256 nonce,uint64 deadline)")
pub const BONDING_TYPEHASH: [u8; 32] = match const_hex::const_decode_to_array::<32>(
    b"01fc59c2dfb4eec49085b15b62820f424d220516ce7d159399ef2b932457da97",
) {
    Ok(x) => x,
    _ => panic!(),
};

fn pack(ed_addr: &[u8; 32], eth_addr: Address, nonce: U256, deadline: u64) -> [u8; 32 * 5] {
    let mut b = [0u8; 32 * 5];
    b[..32].copy_from_slice(&BONDING_TYPEHASH);
    b[32..64].copy_from_slice(ed_addr);
    b[64..96].copy_from_slice(&eth_addr.into_word().as_slice());
    b[96..128].copy_from_slice(&nonce.to_be_bytes::<32>());
    b[160 - 8..].copy_from_slice(&deadline.to_be_bytes());
    b
}

pub fn hash(ed_addr: &[u8; 32], eth_addr: Address, nonce: U256, deadline: u64) -> FixedBytes<32> {
    keccak(pack(ed_addr, eth_addr, nonce, deadline))
}

#[test]
fn test_hash() {
    use stylus_sdk::alloy_primitives::address;
    let ed_sig: [u8; 32] =
        const_hex::decode("90ab9b736d91ca761102da2339e9a715a454797a4bc6a4719a500a9e891abad0")
            .unwrap()
            .try_into()
            .unwrap();
    let eth_addr = address!("6221a9c005f6e47eb398fd867784cacfdcfff4e7");
    let nonce = U256::ZERO;
    let deadline = 100000;
    let data =
    assert_eq!(
	    "01fc59c2dfb4eec49085b15b62820f424d220516ce7d159399ef2b932457da9790ab9b736d91ca761102da2339e9a715a454797a4bc6a4719a500a9e891abad00000000000000000000000006221a9c005f6e47eb398fd867784cacfdcfff4e7000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000186a0".to_string(),
	    const_hex::encode(pack(&ed_sig, eth_addr, nonce, deadline)),
	);
    assert_eq!(
        FixedBytes::<32>::from(
            const_hex::decode_to_array::<_, 32>(
                "241a4f4e13aea8f5a3e066522a8f26bd35e9e258d49323bab79ab115816cb9ae",
            )
            .unwrap()
        ),
        hash(&ed_sig, eth_addr, nonce, deadline)
    );
}
