use crate::network::Network;

use ed25519_dalek::{SigningKey, VerifyingKey};

pub fn pick_solver_key(n: Network) -> VerifyingKey {
    match n {
        Network::CUSTOM => VerifyingKey::from_bytes(&[
            0x5c, 0xdf, 0xa3, 0xae, 0x86, 0x21, 0x29, 0xa3, 0xc9, 0x56, 0x32, 0x94, 0x0e, 0xe7,
            0x10, 0x42, 0x51, 0x55, 0x6e, 0x86, 0xfe, 0xc0, 0x98, 0xa2, 0x46, 0xa2, 0xa1, 0x17,
            0x8d, 0x20, 0x6b, 0xc9,
        ])
        .unwrap(),
        Network::TESTNET => VerifyingKey::from_bytes(&[0u8; 32]).unwrap(),
        Network::MAINNET => VerifyingKey::from_bytes(&[1u8; 32]).unwrap(),
    }
}

pub fn solver_key_offline() -> SigningKey {
    SigningKey::from_bytes(
        &const_hex::decode_to_array(
            "9e879ec92a3949f810e4ac37df4c4488bcf5945df1e18491b9f478b5a5034849",
        )
        .unwrap(),
    )
}
