use ed25519_dalek::VerifyingKey;

pub const SOLVER_KEY_TESTNET: VerifyingKey = OnceCell::match VerifyingKey::from_bytes(&[0u8; 32]) {
    Ok(v) => v,
    Err(_) => panic!()
};
