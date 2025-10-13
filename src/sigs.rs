pub fn make_onboarding_sig(
    owner: &[u8; 20],
    contract: &[u8; 20],
    nonce: u16,
    chain: u64,
) -> [u8; (20 * 2) + 2 + 8] {
    let mut b = [0u8; (20 * 2) + 2 + 8];
    b[..20].copy_from_slice(owner);
    b[20..20 + 20].copy_from_slice(contract);
    b[20 + 20..(20 * 2) + 2].copy_from_slice(&nonce.to_be_bytes());
    b[(20 * 2) + 2..].copy_from_slice(&chain.to_be_bytes());
    b
}
