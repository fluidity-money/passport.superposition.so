use ed25519_dalek::VerifyingKey;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref SOLVER_KEY_TESTNET: VerifyingKey =
        VerifyingKey::from_bytes(&[0u8; 32]).expect("bad testnet verifyingkey");

    pub static ref SOLVER_KEY_MAINNET: VerifyingKey =
        VerifyingKey::from_bytes(&[1u8; 32]).expect("bad mainnet verifyingkey");
}
