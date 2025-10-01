use crate::network::Network;

use ed25519_dalek::{SigningKey, VerifyingKey};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref SOLVER_KEY_CUSTOM: VerifyingKey =
        VerifyingKey::from_bytes(&match const_hex::decode_to_array(
            "5cdfa3ae862129a3c95632940ee7104251556e86fec098a246a2a1178d206bc9"
        ) {
            Ok(v) => v,
            _ => panic!(),
        })
        .expect("bad offline verifyingkey");
    pub static ref SOLVER_KEY_TESTNET: VerifyingKey =
        VerifyingKey::from_bytes(&[0u8; 32]).expect("bad testnet verifyingkey");
    pub static ref SOLVER_KEY_MAINNET: VerifyingKey =
        VerifyingKey::from_bytes(&[1u8; 32]).expect("bad mainnet verifyingkey");
}

pub fn pick_solver_key(n: Network) -> VerifyingKey {
    match n {
        Network::CUSTOM => *SOLVER_KEY_CUSTOM,
        Network::TESTNET => *SOLVER_KEY_TESTNET,
        Network::MAINNET => *SOLVER_KEY_MAINNET,
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
